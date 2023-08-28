use argon::rotary::{RotaryCanon, RotaryTarget};

use super::{worker::SlotWorkerHandle, Slot, SlotConfig};
use crate::{processor::sync::SyncSlotProcessor, worker, ArgonSlotError};

pub struct SyncSlot<SP>
where
    SP: SyncSlotProcessor,
{
    processor: SP,
    config: SlotConfig,
}

impl<SP> From<SP> for SyncSlot<SP>
where
    SP: SyncSlotProcessor,
{
    fn from(processor: SP) -> Self {
        Self {
            processor,
            config: SlotConfig::default(),
        }
    }
}

impl<SP> From<SP> for Box<SyncSlot<SP>>
where
    SP: SyncSlotProcessor,
{
    fn from(processor: SP) -> Self {
        Box::new(Into::<SyncSlot<SP>>::into(processor))
    }
}

impl<SP> From<(SP, SlotConfig)> for SyncSlot<SP>
where
    SP: SyncSlotProcessor,
{
    fn from(pair: (SP, SlotConfig)) -> Self {
        Self {
            processor: pair.0,
            config: pair.1,
        }
    }
}

impl<SP> From<(SP, SlotConfig)> for Box<SyncSlot<SP>>
where
    SP: SyncSlotProcessor,
{
    fn from(pair: (SP, SlotConfig)) -> Self {
        Box::new(Into::<SyncSlot<SP>>::into(pair))
    }
}

impl<SP> Slot for SyncSlot<SP>
where
    SP: SyncSlotProcessor,
{
    fn get_config(&self) -> SlotConfig {
        self.config.clone()
    }

    fn start_worker(
        &mut self,
        entry_rx: RotaryTarget,
        exit_tx: RotaryCanon,
        next_tx: RotaryCanon,
        forward_tx: Option<RotaryCanon>,
    ) -> Result<SlotWorkerHandle, ArgonSlotError> {
        let processor = self.processor.clone();
        let config = self.config.clone();

        let forward_tx = forward_tx.unwrap_or(next_tx.clone());

        let handle = tokio::spawn(worker::sync::run_worker(
            processor, config, entry_rx, exit_tx, next_tx, forward_tx,
        ));

        Ok(SlotWorkerHandle { handle })
    }
}
