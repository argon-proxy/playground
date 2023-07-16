use argon::slot::{SlotPacket, TunRackSequentialSlot, TunRackSlotBuilder, TunRackSlotProcessResult};

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

impl TunRackSequentialSlot for LogSlot {
    type Event = ();
    type Data = tun::TunPacket;
    type Action = ();

    fn deserialize(&self, packet: tun::TunPacket) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket> {
        Ok(SlotPacket::Data(packet))
    }

    fn handle_event(&mut self, _event: Self::Event) -> Vec<Self::Action> {
        unreachable!()
    }

    fn serialize(&self, _action: Self::Action) -> tun::TunPacket {
        unreachable!()
    }

    fn process(&self, data: Self::Data) -> TunRackSlotProcessResult {
        println!("[logslot] {:?}", data);

        TunRackSlotProcessResult {
            forward: vec![data],
            exit: vec![],
        }
    }
}
