use super::{worker::SlotWorkerHandle, SlotConfig};
use crate::{
    error::TunRackError,
    rotary::{RotaryCanon, RotaryTarget},
};

pub trait Slot: 'static {
    fn get_config(&self) -> SlotConfig;

    fn start_worker(
        &mut self,
        entry_rx: RotaryTarget,
        next_tx: RotaryCanon,
        exit_tx: RotaryCanon,
    ) -> Result<SlotWorkerHandle, TunRackError>;
}
