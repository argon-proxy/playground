use argon::rotary::{build_single_channel, RotaryCanon, RotaryTarget};
use argon_slot::worker::SlotWorkerHandle;
use futures::{FutureExt, Stream, StreamExt};
use gcd::euclid_usize;
use layout::TunRackLayoutSlot;
use nonempty::NonEmpty;

mod constants;
use constants::INTRA_SLOT_CHANNEL_SIZE;

mod error;
pub use error::TunRackError;

mod layout;
pub use layout::{TunRackLayout, TunRackLayoutError};

#[derive(Default)]
pub struct TunRackBuilder {}

impl TunRackBuilder {
    pub fn build_from_layout(
        self,
        layout: TunRackLayout,
    ) -> Result<(RotaryCanon, TunRack, RotaryTarget), TunRackError> {
        let mut handles = Vec::default();

        let (exit_tx, exit_rx) = build_single_channel(INTRA_SLOT_CHANNEL_SIZE);
        let exit_tx = RotaryCanon::from(exit_tx);
        let exit_rx = RotaryTarget::from(exit_rx);

        let (entry_tx, next_rx) =
            Self::build_layout_slots(layout.slots, &mut handles, &exit_tx)?;

        Ok((entry_tx, TunRack::new(handles, next_rx), exit_rx))
    }

    fn build_layout_slots(
        slots: Vec<TunRackLayoutSlot>,
        handles: &mut Vec<SlotWorkerHandle>,
        exit_tx: &RotaryCanon,
    ) -> Result<(RotaryCanon, RotaryTarget), TunRackError> {
        let (slot_entry_tx, exit_rx) =
            build_single_channel(INTRA_SLOT_CHANNEL_SIZE);

        let mut slot_rev_iter = slots.into_iter().rev();

        let Some(slot) = slot_rev_iter.next() else {
            return Ok((RotaryCanon::from(slot_entry_tx), RotaryTarget::from(exit_rx)));
        };

        let (mut slot, mut slot_props) = slot.build()?;
        let mut slot_entry_txs = (0..slot.get_config().workers)
            .map(|_| RotaryCanon::from(slot_entry_tx.clone()))
            .collect::<Vec<RotaryCanon>>();

        drop(slot_entry_tx);

        for prev_slot in slot_rev_iter {
            let (prev_slot, prev_slot_props) = prev_slot.build()?;

            let prev_slot_config = prev_slot.get_config();

            let channel_groups =
                euclid_usize(prev_slot_config.workers, slot_entry_txs.len());

            let (prev_slot_entry_txs, prev_slot_next_rxs) = (0..channel_groups)
                .map(|_| {
                    Self::build_channels(
                        prev_slot_config.workers / channel_groups,
                        slot_entry_txs.len() / channel_groups,
                    )
                })
                .fold(
                    (
                        Vec::with_capacity(prev_slot_config.workers),
                        Vec::with_capacity(slot_entry_txs.len()),
                    ),
                    |mut acc, (txs, rxs)| {
                        acc.0.extend(txs);
                        acc.1.extend(rxs);
                        acc
                    },
                );

            let mut prev_slot_next_rxs_iter = prev_slot_next_rxs.into_iter();
            let mut slot_entry_txs_iter = slot_entry_txs.into_iter();

            // TODO: Handle subslot routing

            for (entry_rx, next_tx) in
                (&mut prev_slot_next_rxs_iter).zip(&mut slot_entry_txs_iter)
            {
                handles.push(slot.start_worker(
                    entry_rx,
                    exit_tx.clone(),
                    next_tx,
                    None,
                )?);
            }

            slot = prev_slot;
            slot_props = prev_slot_props;
            slot_entry_txs = prev_slot_entry_txs;
        }

        let mut entry_txs = Vec::new();

        for next_tx in slot_entry_txs {
            let (entry_tx, entry_rx) =
                build_single_channel(INTRA_SLOT_CHANNEL_SIZE);

            entry_txs.push(entry_tx);

            handles.push(slot.start_worker(
                RotaryTarget::from(entry_rx),
                exit_tx.clone(),
                next_tx,
                None,
            )?);
        }

        Ok((
            RotaryCanon::new(
                NonEmpty::from_vec(entry_txs)
                    .ok_or(TunRackError::InternalError)?,
            ),
            RotaryTarget::from(exit_rx),
        ))
    }

    fn build_channels(
        txs: usize,
        rxs: usize,
    ) -> (Vec<RotaryCanon>, Vec<RotaryTarget>) {
        let (ins, outs): (Vec<_>, Vec<_>) = (0..rxs)
            .map(|_| build_single_channel(INTRA_SLOT_CHANNEL_SIZE))
            .unzip();

        (
            (0..txs)
                .map(|_| {
                    RotaryCanon::new(NonEmpty::from_vec(ins.clone()).unwrap())
                })
                .collect(),
            outs.into_iter().map(RotaryTarget::from).collect(),
        )
    }
}

pub struct TunRack {
    handles: Vec<SlotWorkerHandle>,
    target: RotaryTarget,
}

impl TunRack {
    pub fn new(handles: Vec<SlotWorkerHandle>, target: RotaryTarget) -> Self {
        Self { handles, target }
    }
}

impl Stream for TunRack {
    type Item = Result<tun::TunPacket, TunRackError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if let std::task::Poll::Ready(option) = self.target.poll_next_unpin(cx)
        {
            return std::task::Poll::Ready(Some(
                option.ok_or(TunRackError::SlotChannelClosed),
            ));
        }

        for slot in &mut self.handles {
            if let std::task::Poll::Ready(result) = slot.handle.poll_unpin(cx) {
                return std::task::Poll::Ready(Some(Err(match result {
                    Ok(item) => item.err().map_or(
                        TunRackError::SlotCrash,
                        TunRackError::SlotWorkerError,
                    ),
                    Err(e) => TunRackError::TokioJoinError(e),
                })));
            }
        }

        std::task::Poll::Pending
    }
}
