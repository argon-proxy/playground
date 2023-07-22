use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::RwLock;

use super::{worker::SlotWorkerHandle, Slot, SlotPacket, SlotProcessResult};
use crate::{
    error::TunRackError,
    rotary::{RotaryCanon, RotaryTarget},
};

pub trait SyncSlotProcessor: Send + Sync + 'static {
    type Event: Send + Sync;
    type Data: Send + Sync;
    type Action: Send + Sync;

    fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket>;

    fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action>;

    fn serialize(&self, action: Self::Action) -> tun::TunPacket;

    fn process(&self, data: Self::Data) -> SlotProcessResult;
}

pub struct SyncSlot<SP>
where
    SP: SyncSlotProcessor,
{
    processor: Arc<RwLock<SP>>,
}

impl<SP> From<SP> for SyncSlot<SP>
where
    SP: SyncSlotProcessor,
{
    fn from(processor: SP) -> Self {
        Self {
            processor: Arc::new(RwLock::new(processor)),
        }
    }
}

impl<SP> Slot for SyncSlot<SP>
where
    SP: SyncSlotProcessor,
{
    fn start_worker(
        &mut self,
        mut entry_rx: RotaryTarget,
        mut next_tx: RotaryCanon,
        mut exit_tx: RotaryCanon,
    ) -> Result<SlotWorkerHandle, TunRackError> {
        let processor = self.processor.clone();

        let handle = tokio::spawn(async move {
            while let Some(tun_packet) = entry_rx.next().await {
                let processor_lock = processor.read().await;

                let packet = match <SP as SyncSlotProcessor>::deserialize(
                    &processor_lock,
                    tun_packet,
                ) {
                    Ok(packet) => packet,
                    Err(tun_packet) => {
                        if !next_tx.fire(tun_packet)? {
                            println!("[warn] dropped packet");
                        }

                        continue;
                    },
                };

                match packet {
                    SlotPacket::Event(event) => {
                        drop(processor_lock);

                        let mut processor_lock = processor.write().await;

                        let actions = <SP as SyncSlotProcessor>::handle_event(
                            &mut processor_lock,
                            event,
                        );

                        let processor_lock = processor_lock.downgrade();

                        for action in actions {
                            if !exit_tx.fire(
                                <SP as SyncSlotProcessor>::serialize(
                                    &processor_lock,
                                    action,
                                ),
                            )? {
                                println!("[warn] dropped packet");
                            }
                        }
                    },
                    SlotPacket::Data(data) => {
                        let result = <SP as SyncSlotProcessor>::process(
                            &processor_lock,
                            data,
                        );

                        for forward in result.forward {
                            if !next_tx.fire(forward)? {
                                println!("[warn] dropped packet");
                            }
                        }

                        for exit in result.exit {
                            if !exit_tx.fire(exit)? {
                                println!("[warn] dropped packet");
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
