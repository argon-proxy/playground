use super::{SlotPacket, SlotProcessResult, SlotRunnerConfig};
use crate::runner::SequentialSlotRunner;

pub trait SequentialSlot: Send + Sync + 'static {
    type Event: Send + Sync;
    type Data: Send + Sync;
    type Action: Send + Sync;

    fn deserialize(&self, packet: tun::TunPacket) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket>;

    fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action>;

    fn serialize(&self, action: Self::Action) -> tun::TunPacket;

    fn process(&self, data: Self::Data) -> SlotProcessResult;
}

pub struct SequentialSlotRunnerConfig {}

impl<S> SlotRunnerConfig<S, SequentialSlotRunner<S>> for SequentialSlotRunnerConfig
where
    S: SequentialSlot,
{
    fn build(&mut self, slot: S) -> SequentialSlotRunner<S> {
        SequentialSlotRunner { slot }
    }
}
