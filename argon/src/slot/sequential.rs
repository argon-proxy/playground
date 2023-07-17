use super::{SlotPacket, SlotProcessResult};

pub trait SequentialSlot: Send + Sync + 'static {
    type Event: Send + Sync;
    type Data: Send + Sync;
    type Action: Send + Sync;

    fn deserialize(&self, packet: tun::TunPacket) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket>;

    fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action>;

    fn serialize(&self, action: Self::Action) -> tun::TunPacket;

    fn process(&self, data: Self::Data) -> SlotProcessResult;
}
