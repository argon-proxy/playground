use futures::{FutureExt, Stream, StreamExt};
use gcd::euclid_usize;
use itertools::{multizip, Itertools};
use nonempty::NonEmpty;

use crate::{
    constants::INTRA_SLOT_CHANNEL_SIZE,
    error::TunRackError,
    rotary::{build_single_channel, RotaryCanon, RotaryTarget},
    slot::{
        worker::{SlotWorkerError, SlotWorkerHandle},
        Slot,
    },
};

#[derive(Default)]
pub struct TunRackBuilder {
    slots: Vec<Box<dyn Slot>>,
}

impl TunRackBuilder {
    pub fn add_slot<S>(mut self, slot: impl Into<S>) -> Self
    where
        S: Slot,
    {
        self.slots.push(Box::new(slot.into()));

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
            return std::task::Poll::Ready(Some(option.ok_or(
                TunRackError::SlotWorkerError(
                    SlotWorkerError::SlotChannelClosed,
                ),
            )));
        }

        for slot in &mut self.handles {
            if let std::task::Poll::Ready(result) = slot.handle.poll_unpin(cx) {
                return std::task::Poll::Ready(Some(Err(match result {
                    Ok(item) => {
                        item.err().unwrap_or(SlotWorkerError::SlotCrash).into()
                    },
                    Err(e) => TunRackError::TokioJoinError(e),
                })));
            }
        }

        std::task::Poll::Pending
    }
}
