use async_trait::async_trait;

use crate::rack::slot::{SlotPacket, TunRackSlot, TunRackSlotBuilder, TunRackSlotProcessResult};

pub struct LogSlotBuilder {}

impl Default for LogSlotBuilder {
    fn default() -> Self {
        Self {}
    }
}

impl TunRackSlotBuilder<LogSlot> for LogSlotBuilder {
    fn build(self) -> LogSlot {
        LogSlot {}
    }
}

pub struct LogSlot {}

#[async_trait]
impl TunRackSlot for LogSlot {
    type Event = ();
    type Data = tun::TunPacket;
    type Action = ();

    async fn deserialize(&self, packet: tun::TunPacket) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket> {
        Ok(SlotPacket::Data(packet))
    }

    async fn handle_event(&self, _event: Self::Event) -> Vec<Self::Action> {
        unreachable!()
    }

    async fn serialize(&self, _action: Self::Action) -> tun::TunPacket {
        unreachable!()
    }

    async fn process(&self, data: Self::Data) -> TunRackSlotProcessResult {
        println!("[logslot] {:?}", data);

        TunRackSlotProcessResult {
            forward: vec![data],
            exit: vec![],
        }
    }
}
