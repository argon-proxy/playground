use packet::{Builder, Packet};
use tun::TunPacket;

use crate::rack::{
    slot::TunRackSlot,
    slot_builder::TunRackSlotBuilder,
    slot_handle::TunRackSlotHandle,
    TunRackSlotReceiver,
    TunRackSlotSender,
};

pub struct PingSlotBuilder {}

impl PingSlotBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl TunRackSlotBuilder<PingSlot> for PingSlotBuilder {
    fn build(self, rx: TunRackSlotReceiver, tx: TunRackSlotSender, exit_tx: TunRackSlotSender) -> PingSlot {
        PingSlot::new(rx, tx, exit_tx)
    }
}

pub struct PingSlot {
    rx: TunRackSlotReceiver,
    tx: TunRackSlotSender,
    exit_tx: TunRackSlotSender,
}

impl PingSlot {
    pub fn new(rx: TunRackSlotReceiver, tx: TunRackSlotSender, exit_tx: TunRackSlotSender) -> Self {
        Self { rx, tx, exit_tx }
    }
}

impl TunRackSlot for PingSlot {
    fn run(self) -> TunRackSlotHandle {
        let mut rx = self.rx;
        let tx = self.tx;
        let exit_tx = self.exit_tx;

        let handle = tokio::spawn(async move {
            while let Some(tun_packet) = rx.recv().await {
                match packet::ip::Packet::new(tun_packet.get_bytes()) {
                    Ok(packet::ip::Packet::V4(ipv4_packet)) => match packet::icmp::Packet::new(ipv4_packet.payload()) {
                        Ok(icmp_packet) => match icmp_packet.echo() {
                            Ok(icmp_echo_packet) => {
                                let reply = packet::ip::v4::Builder::default()
                                    .id(0x42)
                                    .unwrap()
                                    .ttl(64)
                                    .unwrap()
                                    .source(ipv4_packet.destination())
                                    .unwrap()
                                    .destination(ipv4_packet.source())
                                    .unwrap()
                                    .icmp()
                                    .unwrap()
                                    .echo()
                                    .unwrap()
                                    .reply()
                                    .unwrap()
                                    .identifier(icmp_echo_packet.identifier())
                                    .unwrap()
                                    .sequence(icmp_echo_packet.sequence())
                                    .unwrap()
                                    .payload(icmp_echo_packet.payload())
                                    .unwrap()
                                    .build()
                                    .unwrap();

                                exit_tx.send(TunPacket::new(reply)).await?;
                                continue;
                            },
                            _ => {},
                        },
                        _ => {},
                    },
                    _ => {},
                }

                tx.send(tun_packet).await?;
            }

            Ok(())
        });

        TunRackSlotHandle::new(handle)
    }
}
