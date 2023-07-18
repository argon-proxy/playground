use argon::slot::{SlotBuilder, SlotPacket, SlotProcessResult, SyncSlot};
use packet::{Builder, Packet};

pub struct PingSequentialSlotBuilder {}

impl Default for PingSequentialSlotBuilder {
    fn default() -> Self {
        Self {}
    }
}

impl SlotBuilder<PingSequentialSlot> for PingSequentialSlotBuilder {
    fn build(self) -> PingSequentialSlot {
        PingSequentialSlot {}
    }
}

pub struct PingSequentialSlot {}

impl SyncSlot for PingSequentialSlot {
    type Event = ();
    type Data = (packet::ip::v4::Packet<Vec<u8>>, packet::icmp::echo::Packet<Vec<u8>>);
    type Action = ();

    fn deserialize(&self, packet: tun::TunPacket) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket> {
        match packet::ip::Packet::new(packet.get_bytes()) {
            Ok(packet::ip::Packet::V4(ipv4_packet)) => match packet::icmp::Packet::new(ipv4_packet.payload()) {
                Ok(icmp_packet) => match icmp_packet.echo() {
                    Ok(icmp_echo_packet) => Ok(SlotPacket::Data((ipv4_packet.to_owned(), icmp_echo_packet.to_owned()))),
                    _ => Err(packet),
                },
                _ => Err(packet),
            },
            _ => Err(packet),
        }
    }

    fn handle_event(&mut self, _event: Self::Event) -> Vec<Self::Action> {
        unreachable!()
    }

    fn serialize(&self, _action: Self::Action) -> tun::TunPacket {
        unreachable!()
    }

    fn process(&self, data: Self::Data) -> SlotProcessResult {
        SlotProcessResult {
            forward: vec![],
            exit: vec![tun::TunPacket::new(
                packet::ip::v4::Builder::default()
                    .id(0x42)
                    .unwrap()
                    .ttl(64)
                    .unwrap()
                    .source(data.0.destination())
                    .unwrap()
                    .destination(data.0.source())
                    .unwrap()
                    .icmp()
                    .unwrap()
                    .echo()
                    .unwrap()
                    .reply()
                    .unwrap()
                    .identifier(data.1.identifier())
                    .unwrap()
                    .sequence(data.1.sequence())
                    .unwrap()
                    .payload(data.1.payload())
                    .unwrap()
                    .build()
                    .unwrap(),
            )],
        }
    }
}
