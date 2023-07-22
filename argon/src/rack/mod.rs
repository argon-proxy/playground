use futures::{FutureExt, Stream};
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

    pub fn build(
        self,
    ) -> Result<(RotaryCanon, TunRack, RotaryTarget), TunRackError> {
        let (entry_tx, entry_rx) =
            build_single_channel(INTRA_SLOT_CHANNEL_SIZE);

        let entry_tx = RotaryCanon::new(NonEmpty::new(entry_tx));
        let mut entry_rx = RotaryTarget::new(NonEmpty::new(entry_rx));

        let (exit_tx, exit_rx) = build_single_channel(INTRA_SLOT_CHANNEL_SIZE);

        let exit_tx = RotaryCanon::new(NonEmpty::new(exit_tx));
        let exit_rx = RotaryTarget::new(NonEmpty::new(exit_rx));

        let mut handles = Vec::new();

        for slot in self.slots {
            let (new_entry_rx, handle) =
                TunRackBuilder::build_slot(entry_rx, slot, exit_tx.clone())?;

            entry_rx = new_entry_rx;
            handles.push(handle);
        }

        Ok((entry_tx, TunRack::new(handles), exit_rx))
    }

    fn build_slot(
        entry_rx: RotaryTarget,
        mut slot: Box<dyn Slot>,
        exit_tx: RotaryCanon,
    ) -> Result<(RotaryTarget, SlotWorkerHandle), TunRackError> {
        let (next_tx, next_rx) = build_single_channel(INTRA_SLOT_CHANNEL_SIZE);

        let next_tx = RotaryCanon::new(NonEmpty::new(next_tx));
        let next_rx = RotaryTarget::new(NonEmpty::new(next_rx));

        let handle = slot.start_worker(entry_rx, next_tx, exit_tx)?;

        Ok((next_rx, handle))
    }
}

pub struct TunRack {
    handles: Vec<SlotWorkerHandle>,
}

impl TunRack {
    pub fn new(handles: Vec<SlotWorkerHandle>) -> Self {
        Self { handles }
    }
}

impl Stream for TunRack {
    type Item = Result<tun::TunPacket, TunRackError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
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
