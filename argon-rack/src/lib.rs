use argon::rotary::{build_single_channel, RotaryCanon, RotaryTarget};
use argon_slot::{
    processor::{r#async::AsyncSlotProcessor, sync::SyncSlotProcessor},
    worker::SlotWorkerHandle,
    AsyncSlot, Slot, SyncSlot,
};
use futures::{FutureExt, Stream, StreamExt};
use gcd::euclid_usize;
use itertools::{multizip, Itertools};
use nonempty::NonEmpty;

mod constants;
use constants::INTRA_SLOT_CHANNEL_SIZE;

mod error;
pub use error::TunRackError;

mod layout;
pub use layout::{TunRackLayoutError, TunRackSlot};

#[derive(Default)]
pub struct TunRackBuilder {
    slots: Vec<Box<dyn Slot>>,
}

impl TunRackBuilder {
    pub fn add_slot(&mut self, slot: Box<dyn Slot>) {
        self.slots.push(slot)
    }

    pub fn add_sync_slot<SP>(
        mut self,
        slot: impl Into<Box<SyncSlot<SP>>>,
    ) -> Self
    where
        SP: SyncSlotProcessor,
    {
        self.slots.push(slot.into());

        self
    }

    pub fn add_async_slot<SP>(
        mut self,
        slot: impl Into<Box<AsyncSlot<SP>>>,
    ) -> Self
    where
        SP: AsyncSlotProcessor,
    {
        self.slots.push(slot.into());

        self
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
            outs.into_iter()
                .map(|rx| RotaryTarget::new(NonEmpty::new(rx)))
                .collect(),
        )
    }

    pub fn build(
        self,
    ) -> Result<(RotaryCanon, TunRack, RotaryTarget), TunRackError> {
        let (canons, targets): (Vec<Vec<RotaryCanon>>, Vec<Vec<RotaryTarget>>) =
            [1].into_iter()
                .chain(self.slots.iter().map(|slot| slot.get_config().workers))
                .chain([1])
                .tuple_windows()
                .map(|(prev, curr)| {
                    let groups = euclid_usize(prev, curr);

                    (0..groups)
                        .map(|_| {
                            Self::build_channels(prev / groups, curr / groups)
                        })
                        .fold(
                            (Vec::default(), Vec::default()),
                            |mut acc, (txs, rxs)| {
                                acc.0.extend(txs);
                                acc.1.extend(rxs);
                                acc
                            },
                        )
                })
                .unzip();

        let mut canons = canons.into_iter();
        let canon_first = canons
            .next()
            .and_then(|c| c.into_iter().next())
            .ok_or(TunRackError::InternalError)?;

        let mut targets = targets.into_iter();

        let mut handles = Vec::default();

        let (exit_tx, exit_rx) = build_single_channel(INTRA_SLOT_CHANNEL_SIZE);
        let exit_tx = RotaryCanon::new(NonEmpty::new(exit_tx));
        let exit_rx = RotaryTarget::new(NonEmpty::new(exit_rx));

        for (mut slot, slot_canons, slot_targets) in
            multizip((self.slots, &mut canons, &mut targets))
        {
            let workers = slot.get_config().workers;

            debug_assert!(workers == slot_canons.len());
            debug_assert!(workers == slot_targets.len());

            for (slot_canon, slot_target) in
                multizip((slot_canons, slot_targets))
            {
                let handle = slot.start_worker(
                    slot_target,
                    slot_canon,
                    exit_tx.clone(),
                )?;

                handles.push(handle);
            }
        }

        debug_assert!(canons.next().is_none());

        let target_last = targets
            .next()
            .and_then(|c| c.into_iter().next())
            .ok_or(TunRackError::InternalError)?;
        debug_assert!(targets.next().is_none());

        Ok((canon_first, TunRack::new(handles, target_last), exit_rx))
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
