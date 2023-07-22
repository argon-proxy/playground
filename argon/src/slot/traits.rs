use super::worker::SlotWorkerHandle;
use crate::{
    error::TunRackError,
    rotary::{RotaryCanon, RotaryTarget},
};

pub trait Slot: 'static {
    fn start_worker(
        &mut self,
        entry_rx: RotaryTarget,
        next_tx: RotaryCanon,
        exit_tx: RotaryCanon,
    ) -> Result<SlotWorkerHandle, TunRackError>;
}
