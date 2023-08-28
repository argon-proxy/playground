use argon_plugin::argon_plugin;
use argon_slot::processor::{
    r#async::AsyncSlotProcessor, sync::SyncSlotProcessor, SlotPacket,
    SlotProcessorResult,
};
use async_trait::async_trait;
use packet::{Builder, Packet};

type Event = ();
type Data = (
    packet::ip::v4::Packet<Vec<u8>>,
    packet::icmp::echo::Packet<Vec<u8>>,
);
type Action = ();

#[derive(Clone, Default)]
pub struct PingSlotProcessor {}

impl SyncSlotProcessor for PingSlotProcessor {
    type Event = Event;
    type Data = Data;
    type Action = Action;

    fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> SlotPacket<Self::Event, Self::Data> {
        if let Ok(packet::ip::Packet::V4(ipv4_packet)) =
            packet::ip::Packet::new(packet.get_bytes())
        {
            if let Ok(icmp_packet) =
                packet::icmp::Packet::new(ipv4_packet.payload())
            {
                if let Ok(icmp_echo_packet) = icmp_packet.echo() {
                    return SlotPacket::Data((
                        ipv4_packet.to_owned(),
                        icmp_echo_packet.to_owned(),
                    ));
                }
            }
        }

        SlotPacket::Next(packet)
    }

    fn handle_event(&mut self, _event: Self::Event) -> Vec<Self::Action> {
        unreachable!()
    }

    fn serialize(&self, _action: Self::Action) -> tun::TunPacket {
        unreachable!()
    }

    fn process(&self, data: Self::Data) -> SlotProcessorResult {
        SlotProcessorResult {
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

#[derive(Default)]
pub struct PingSlotPlugin {}

argon_plugin!(
    "argon/ping",
    PingSlotPlugin,
    PingSlotPlugin::default,
    PingSlotProcessor,
    Event,
    Data,
    Action
);

#[async_trait]
impl AsyncSlotProcessor for PingSlotProcessor {
    type Event = ();
    type Data = (
        packet::ip::v4::Packet<Vec<u8>>,
        packet::icmp::echo::Packet<Vec<u8>>,
    );
    type Action = ();

    async fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> SlotPacket<Self::Event, Self::Data> {
        <Self as SyncSlotProcessor>::deserialize(self, packet)
    }

    async fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action> {
        <Self as SyncSlotProcessor>::handle_event(self, event)
    }

    async fn serialize(&self, action: Self::Action) -> tun::TunPacket {
        <Self as SyncSlotProcessor>::serialize(self, action)
    }

    async fn process(&self, data: Self::Data) -> SlotProcessorResult {
        <Self as SyncSlotProcessor>::process(self, data)
    }
}
