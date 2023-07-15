use super::{slot::SlotPacket, TunRackSlot, TunRackSlotHandle, TunRackSlotReceiver, TunRackSlotSender};

pub struct TunRackSlotRunner<S: TunRackSlot> {
    pub slot: S,
}

impl<S: TunRackSlot> TunRackSlotRunner<S> {
    pub fn run(
        self,
        mut rx: TunRackSlotReceiver,
        tx: TunRackSlotSender,
        exit_tx: TunRackSlotSender,
    ) -> TunRackSlotHandle {
        let slot = self.slot;

        let handle = tokio::spawn(async move {
            while let Some(tun_packet) = rx.recv().await {
                let packet = match slot.deserialize(tun_packet).await {
                    Ok(packet) => packet,
                    Err(tun_packet) => {
                        tx.send(tun_packet).await?;
                        continue;
                    },
                };

                match packet {
                    SlotPacket::Event(event) => {
                        let actions = slot.handle_event(event).await;

                        for action in actions {
                            exit_tx.send(slot.serialize(action).await).await?;
                        }
                    },
                    SlotPacket::Data(data) => {
                        let result = slot.process(data).await;

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
