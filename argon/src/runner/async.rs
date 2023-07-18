use std::sync::Arc;

use tokio::sync::RwLock;

use super::{SlotContainer, SlotRunner, SlotRunnerConfig, SlotRunnerError, SlotRunnerHandle};
use crate::{
    rack::{SlotReceiver, SlotSender},
    slot::{AsyncSlot, SlotPacket},
};

pub struct AsyncSlotRunnerConfig {}

impl Default for AsyncSlotRunnerConfig {
    fn default() -> Self {
        Self {}
    }
}

impl<S> SlotRunnerConfig<S, AsyncSlotRunner<S>> for AsyncSlotRunnerConfig
where
    S: AsyncSlot,
{
    fn build(&mut self, slot: S) -> AsyncSlotRunner<S> {
        AsyncSlotRunner::new(slot)
    }
}

pub struct AsyncSlotRunner<S: AsyncSlot> {
    pub container: SlotContainer<S>,
}

impl<S> AsyncSlotRunner<S>
where
    S: AsyncSlot,
{
    pub fn new(slot: S) -> Self {
        Self {
            container: SlotContainer {
                slot: Arc::new(RwLock::new(slot)),
            },
        }
    }
}

impl<S: AsyncSlot> SlotRunner<S> for AsyncSlotRunner<S> {
    fn run(
        self,
        mut rx: SlotReceiver,
        tx: SlotSender,
        exit_tx: SlotSender,
    ) -> Result<SlotRunnerHandle, SlotRunnerError> {
        let slot = self.container.slot;

        let handle = tokio::spawn(async move {
            while let Some(tun_packet) = rx.recv().await {
                let slotlock = slot.read().await;

                let packet = match <S as AsyncSlot>::deserialize(&slotlock, tun_packet).await {
                    Ok(packet) => packet,
                    Err(tun_packet) => {
                        tx.send(tun_packet).await?;
                        continue;
                    },
                };

                match packet {
                    SlotPacket::Event(event) => {
                        drop(slotlock);

                        let mut slotlock = slot.write().await;

                        let actions = <S as AsyncSlot>::handle_event(&mut slotlock, event).await;

                        let slotlock = slotlock.downgrade();

                        for action in actions {
                            exit_tx
                                .send(<S as AsyncSlot>::serialize(&slotlock, action).await)
                                .await?;
                        }
                    },
                    SlotPacket::Data(data) => {
                        let result = <S as AsyncSlot>::process(&slotlock, data).await;

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

        Ok(SlotRunnerHandle { handle })
    }
}
