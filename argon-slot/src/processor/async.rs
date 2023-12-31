use async_trait::async_trait;

use super::{SlotPacket, SlotProcessorResult};

#[async_trait]
pub trait AsyncSlotProcessor: Clone + Send + Sync + 'static {
    type Event: Send + Sync;
    type Data: Send + Sync;
    type Action: Send + Sync;

    async fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> SlotPacket<Self::Event, Self::Data>;

    async fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action>;

    async fn serialize(&self, action: Self::Action) -> tun::TunPacket;

    async fn process(&self, data: Self::Data) -> SlotProcessorResult;
}

// TODO: CAsyncSlotProcessor
