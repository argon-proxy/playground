use argon::rotary::{RotaryCanon, RotaryTarget};

use super::{worker::SlotWorkerHandle, Slot, SlotConfig};
use crate::{processor::r#async::AsyncSlotProcessor, worker, ArgonSlotError};

pub struct AsyncSlot<SP>
where
    SP: AsyncSlotProcessor,
{
    processor: SP,
    config: SlotConfig,
}

impl<SP> From<SP> for AsyncSlot<SP>
where
    SP: AsyncSlotProcessor,
{
    fn from(processor: SP) -> Self {
        Self {
            processor,
            config: SlotConfig::default(),
        }
    }
}

impl<SP> From<SP> for Box<AsyncSlot<SP>>
where
    SP: AsyncSlotProcessor,
{
    fn from(processor: SP) -> Self {
        Box::new(Into::<AsyncSlot<SP>>::into(processor))
    }
}

impl<SP> From<(SP, SlotConfig)> for AsyncSlot<SP>
where
    SP: AsyncSlotProcessor,
{
    fn from(pair: (SP, SlotConfig)) -> Self {
        Self {
            processor: pair.0,
            config: pair.1,
        }
    }
}

impl<SP> From<(SP, SlotConfig)> for Box<AsyncSlot<SP>>
where
    SP: AsyncSlotProcessor,
{
    fn from(pair: (SP, SlotConfig)) -> Self {
        Box::new(Into::<AsyncSlot<SP>>::into(pair))
    }
}

impl<SP> Slot for AsyncSlot<SP>
where
    SP: AsyncSlotProcessor,
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

        let handle = tokio::spawn(worker::r#async::run_worker(
            processor, config, entry_rx, exit_tx, next_tx, forward_tx,
        ));

        Ok(SlotWorkerHandle { handle })
    }
}
