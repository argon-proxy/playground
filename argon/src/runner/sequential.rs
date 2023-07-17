use super::{SlotRunner, SlotRunnerConfig, SlotRunnerHandle};
use crate::{
    rack::{SlotReceiver, SlotSender},
    slot::{SequentialSlot, SlotPacket},
};

pub struct SequentialSlotRunnerConfig {}

impl Default for SequentialSlotRunnerConfig {
    fn default() -> Self {
        Self {}
    }
}

impl<S> SlotRunnerConfig<S, SequentialSlotRunner<S>> for SequentialSlotRunnerConfig
where
    S: SequentialSlot,
{
    fn build(&mut self, slot: S) -> SequentialSlotRunner<S> {
        SequentialSlotRunner { slot }
    }
}

pub struct SequentialSlotRunner<S: SequentialSlot> {
    pub slot: S,
}

impl<S: SequentialSlot> SlotRunner<S> for SequentialSlotRunner<S> {
    fn run(self, mut rx: SlotReceiver, tx: SlotSender, exit_tx: SlotSender) -> SlotRunnerHandle {
        let mut slot = self.slot;

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
