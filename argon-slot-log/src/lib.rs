use argon_plugin::argon_plugin;
use argon_slot::processor::{
    sync::SyncSlotProcessor, SlotPacket, SlotProcessorResult,
};

type Event = ();
type Data = tun::TunPacket;
type Action = ();

#[derive(Default)]
pub struct LogSlotProcessor {}

impl SyncSlotProcessor for LogSlotProcessor {
    type Event = Event;
    type Data = Data;
    type Action = Action;

    fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket> {
        Ok(SlotPacket::Data(packet))
    }

    fn handle_event(&mut self, _event: Self::Event) -> Vec<Self::Action> {
        unreachable!()
    }

    fn serialize(&self, _action: Self::Action) -> tun::TunPacket {
        unreachable!()
    }

    fn process(&self, data: Self::Data) -> SlotProcessorResult {
        println!("[logslot] {data:?}");

        SlotProcessorResult {
            forward: vec![data],
            exit: vec![],
        }
    }
}

#[derive(Default)]
pub struct LogSlotPlugin {}

argon_plugin!(
    "argon/log",
    LogSlotPlugin,
    LogSlotPlugin::default,
    LogSlotProcessor,
    Event,
    Data,
    Action
);
