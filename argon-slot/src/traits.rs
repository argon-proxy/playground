use argon::rotary::{RotaryCanon, RotaryTarget};

use super::{worker::SlotWorkerHandle, SlotConfig};
use crate::ArgonSlotError;

pub trait Slot: 'static {
    fn get_config(&self) -> SlotConfig;

    fn start_worker(
        &mut self,
        entry_rx: RotaryTarget,
        next_tx: RotaryCanon,
        exit_tx: RotaryCanon,
    ) -> Result<SlotWorkerHandle, ArgonSlotError>;
}
