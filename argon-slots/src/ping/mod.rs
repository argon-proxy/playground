use argon::slot::{
    AsyncSlotProcessor, SlotPacket, SlotProcessResult, SyncSlotProcessor,
};
use async_trait::async_trait;
use packet::{Builder, Packet};
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

#[derive(Default)]
pub struct PingSlotProcessor {}

impl SyncSlotProcessor for PingSlotProcessor {
    type Event = ();
    type Data = (
        packet::ip::v4::Packet<Vec<u8>>,
        packet::icmp::echo::Packet<Vec<u8>>,
    );
    type Action = ();

    fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket> {
        match packet::ip::Packet::new(packet.get_bytes()) {
            Ok(packet::ip::Packet::V4(ipv4_packet)) => {
                match packet::icmp::Packet::new(ipv4_packet.payload()) {
                    Ok(icmp_packet) => match icmp_packet.echo() {
                        Ok(icmp_echo_packet) => Ok(SlotPacket::Data((
                            ipv4_packet.to_owned(),
                            icmp_echo_packet.to_owned(),
                        ))),
                        _ => Err(packet),
                    },
                    _ => Err(packet),
                }
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

#[async_trait]
impl AsyncSlotProcessor for PingSlotProcessor {
    type Event = ();
    type Data = (
        packet::ip::v4::Packet<Vec<u8>>,
        packet::icmp::echo::Packet<Vec<u8>>,
    );
    type Action = ();

    async fn deserialize<'p>(
        slot: &RwLockReadGuard<'p, Self>,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket> {
        <Self as SyncSlotProcessor>::deserialize(slot, packet)
    }

    async fn handle_event<'p>(
        slot: &mut RwLockWriteGuard<'p, Self>,
        event: Self::Event,
    ) -> Vec<Self::Action> {
        <Self as SyncSlotProcessor>::handle_event(slot, event)
    }

    async fn serialize<'p>(
        slot: &RwLockReadGuard<'p, Self>,
        action: Self::Action,
    ) -> tun::TunPacket {
        <Self as SyncSlotProcessor>::serialize(slot, action)
    }

    async fn process<'p>(
        slot: &RwLockReadGuard<'p, Self>,
        data: Self::Data,
    ) -> SlotProcessResult {
        <Self as SyncSlotProcessor>::process(slot, data)
    }
}
