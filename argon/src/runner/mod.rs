use crate::{
    rack::{TunRackSlotReceiver, TunRackSlotSender},
    slot::{SlotPacket, TunRackSequentialSlot, TunRackSlotHandle},
};

pub trait TunRackSlotRunner<S> {
    fn new(slot: S) -> Self;

    fn run(self, rx: TunRackSlotReceiver, tx: TunRackSlotSender, exit_tx: TunRackSlotSender) -> TunRackSlotHandle;
}

pub struct TunRackSequentialSlotRunner<S: TunRackSequentialSlot> {
    pub slot: S,
}

impl<S: TunRackSequentialSlot> TunRackSlotRunner<S> for TunRackSequentialSlotRunner<S> {
    fn new(slot: S) -> TunRackSequentialSlotRunner<S> {
        Self { slot }
    }
    fn run(self, mut rx: TunRackSlotReceiver, tx: TunRackSlotSender, exit_tx: TunRackSlotSender) -> TunRackSlotHandle {
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

        TunRackSlotHandle { handle }
    }
}
