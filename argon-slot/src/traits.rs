use argon::rotary::{RotaryCanon, RotaryTarget};

use super::{worker::SlotWorkerHandle, SlotConfig};
use crate::ArgonSlotError;

pub trait Slot {
    fn get_config(&self) -> SlotConfig;

    fn start_worker(
        &mut self,
        entry_rx: RotaryTarget,
        exit_tx: RotaryCanon,
        next_tx: RotaryCanon,
        forward_tx: Option<RotaryCanon>,
    ) -> Result<SlotWorkerHandle, ArgonSlotError>;
}
