use std::sync::Arc;

use argon::rotary::{RotaryCanon, RotaryTarget};
use tokio::sync::RwLock;

use super::{worker::SlotWorkerHandle, Slot, SlotConfig};
use crate::{processor::sync::SyncSlotProcessor, worker, ArgonSlotError};

pub type AbiSyncSlotProcessor = Box<
    dyn SyncSlotProcessor<
        Event = dyn Send + Sync,
        Data = dyn Send + Sync,
        Action = dyn Send + Sync,
    >,
>;

pub struct SyncSlot<SP>
where
    SP: SyncSlotProcessor,
{
    processor: Arc<RwLock<SP>>,
    config: SlotConfig,
}

impl<SP> From<SP> for SyncSlot<SP>
where
    SP: SyncSlotProcessor,
{
    fn from(processor: SP) -> Self {
        Self {
            processor: Arc::new(RwLock::new(processor)),
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
            processor: Arc::new(RwLock::new(pair.0)),
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
        next_tx: RotaryCanon,
        exit_tx: RotaryCanon,
    ) -> Result<SlotWorkerHandle, ArgonSlotError> {
        let processor = self.processor.clone();
        let config = self.config.clone();

        let handle = tokio::spawn(worker::sync::run_worker(
            processor, config, entry_rx, next_tx, exit_tx,
        ));

        Ok(SlotWorkerHandle { handle })
    }
}
