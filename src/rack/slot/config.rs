use super::TunRackSequentialSlot;
use crate::rack::runner::{TunRackSequentialSlotRunner, TunRackSlotRunner};

pub trait TunRackRunnerConfig<S, SR>
where
    SR: TunRackSlotRunner<S>,
{
    fn build(&mut self, slot: S) -> SR;
}

pub struct TunRackSequentialSlotRunnerConfig {}

impl<S> TunRackRunnerConfig<S, TunRackSequentialSlotRunner<S>> for TunRackSequentialSlotRunnerConfig
where
    S: TunRackSequentialSlot,
{
    fn build(&mut self, slot: S) -> TunRackSequentialSlotRunner<S> {
        TunRackSequentialSlotRunner { slot }
    }
}
