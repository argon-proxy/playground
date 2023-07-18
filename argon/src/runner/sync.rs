use super::{SlotRunner, SlotRunnerConfig, SlotRunnerHandle, SlotContainer};
use crate::{
    rack::{SlotReceiver, SlotSender},
    slot::{SyncSlot, SlotPacket},
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SyncSlotRunnerConfig {}

impl Default for SyncSlotRunnerConfig {
    fn default() -> Self {
        Self {}
    }
}

impl<S> SlotRunnerConfig<S, SyncSlotRunner<S>> for SyncSlotRunnerConfig
where
    S: SyncSlot,
{
    fn build(&mut self, slot: S) -> SyncSlotRunner<S> {
        let container = SlotContainer { slot: Arc::new(RwLock::new(slot)) };
        SyncSlotRunner { container }
    }
}

pub struct SyncSlotRunner<S: SyncSlot> {
    pub container: SlotContainer<S>,
}

impl<S: SyncSlot> SlotRunner<S> for SyncSlotRunner<S> {
    fn run(self, mut rx: SlotReceiver, tx: SlotSender, exit_tx: SlotSender) -> SlotRunnerHandle {
        let mut slot = match self.container.slot.try_write_owned() {
            Ok(s) => s,
            _ => panic!()
        };

        let handle = tokio::spawn(async move {
            while let Some(tun_packet) = rx.recv().await {
                let packet = match slot.deserialize(tun_packet) {
                    Ok(packet) => packet,
                    Err(tun_packet) => {
                        tx.send(tun_packet).await?;
                        continue;
                    },
                };

                match packet {
                    SlotPacket::Event(event) => {
                        let actions = slot.handle_event(event);

                        for action in actions {
                            exit_tx.send(slot.serialize(action)).await?;
                        }
                    },
                    SlotPacket::Data(data) => {
                        let result = slot.process(data);

                        for forward in result.forward {
                            tx.send(forward).await?;
                        }

                        for exit in result.exit {
                            exit_tx.send(exit).await?;
                        }
                    },
                }
            }

            Ok(())
        });

        SlotRunnerHandle { handle }
    }
}
