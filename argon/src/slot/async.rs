use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::{
    worker::SlotWorkerHandle, Slot, SlotConfig, SlotPacket, SlotProcessResult,
};
use crate::{
    error::TunRackError,
    rotary::{RotaryCanon, RotaryTarget},
};

#[async_trait]
pub trait AsyncSlotProcessor: Send + Sync + 'static {
    type Event: Send + Sync;
    type Data: Send + Sync;
    type Action: Send + Sync;

    async fn deserialize<'p>(
        slot: &RwLockReadGuard<'p, Self>,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket>;

    async fn handle_event<'p>(
        slot: &mut RwLockWriteGuard<'p, Self>,
        event: Self::Event,
    ) -> Vec<Self::Action>;

    async fn serialize<'p>(
        slot: &RwLockReadGuard<'p, Self>,
        action: Self::Action,
    ) -> tun::TunPacket;

    async fn process<'p>(
        slot: &RwLockReadGuard<'p, Self>,
        data: Self::Data,
    ) -> SlotProcessResult;
}

pub struct AsyncSlot<SP>
where
    SP: AsyncSlotProcessor,
{
    processor: Arc<RwLock<SP>>,
    config: SlotConfig,
}

impl<SP> From<SP> for AsyncSlot<SP>
where
    SP: AsyncSlotProcessor,
{
    fn from(processor: SP) -> Self {
        Self {
            processor: Arc::new(RwLock::new(processor)),
            config: SlotConfig::default(),
        }
    }
}

impl<SP> From<(SP, SlotConfig)> for AsyncSlot<SP>
where
    SP: AsyncSlotProcessor,
{
    fn from(pair: (SP, SlotConfig)) -> Self {
        Self {
            processor: Arc::new(RwLock::new(pair.0)),
            config: pair.1,
        }
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
        mut entry_rx: RotaryTarget,
        mut next_tx: RotaryCanon,
        mut exit_tx: RotaryCanon,
    ) -> Result<SlotWorkerHandle, TunRackError> {
        let processor = self.processor.clone();
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            while let Some(tun_packet) = entry_rx.next().await {
                let processor_lock = processor.read().await;

                let packet = match <SP as AsyncSlotProcessor>::deserialize(
                    &processor_lock,
                    tun_packet,
                )
                .await
                {
                    Ok(packet) => packet,
                    Err(tun_packet) => {
                        if !next_tx.fire(tun_packet)? {
                            println!("[{}][warn] dropped packet", config.name);
                        }

                        continue;
                    },
                };

                match packet {
                    SlotPacket::Event(event) => {
                        drop(processor_lock);

                        let mut processor_lock = processor.write().await;

                        let actions = <SP as AsyncSlotProcessor>::handle_event(
                            &mut processor_lock,
                            event,
                        )
                        .await;

                        let processor_lock = processor_lock.downgrade();

                        for action in actions {
                            if !exit_tx.fire(
                                <SP as AsyncSlotProcessor>::serialize(
                                    &processor_lock,
                                    action,
                                )
                                .await,
                            )? {
                                println!(
                                    "[{}][warn] dropped packet",
                                    config.name
                                );
                            }
                        }
                    },
                    SlotPacket::Data(data) => {
                        let result = <SP as AsyncSlotProcessor>::process(
                            &processor_lock,
                            data,
                        )
                        .await;

                        for forward in result.forward {
                            if !next_tx.fire(forward)? {
                                println!(
                                    "[{}][warn] dropped packet",
                                    config.name
                                );
                            }
                        }

                        for exit in result.exit {
                            if !exit_tx.fire(exit)? {
                                println!(
                                    "[{}][warn] dropped packet",
                                    config.name
                                );
                            }
                        }
                    },
                }
            }

            Ok(())
        });

        Ok(SlotWorkerHandle { handle })
    }
}
