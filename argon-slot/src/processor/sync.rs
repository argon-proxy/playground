use super::{SlotPacket, SlotProcessorResult};

pub trait SyncSlotProcessor: Send + Sync + 'static {
    type Event: Send;
    type Data: Send;
    type Action: Send;

    fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket>;

    fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action>;

    fn serialize(&self, action: Self::Action) -> tun::TunPacket;

    fn process(&self, data: Self::Data) -> SlotProcessorResult;
}
