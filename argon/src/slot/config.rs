use super::SequentialSlot;
use crate::runner::{SequentialSlotRunner, SlotRunner};

pub trait SlotRunnerConfig<S, SR>
where
    SR: SlotRunner<S>,
{
    fn build(&mut self, slot: S) -> SR;
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
