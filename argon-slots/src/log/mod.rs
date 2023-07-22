use argon::slot::{SlotPacket, SlotProcessResult, SyncSlotProcessor};

#[derive(Default)]
pub struct LogSlotProcessor {}

impl SyncSlotProcessor for LogSlotProcessor {
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

    fn process(&self, data: Self::Data) -> SlotProcessResult {
        println!("[logslot] {data:?}");

        SlotProcessResult {
            forward: vec![data],
            exit: vec![],
        }
    }
}
