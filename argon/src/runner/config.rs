use crate::runner::SlotRunner;

pub trait SlotRunnerConfig<S, SR>: Default
where
    SR: SlotRunner<S>,
{
    fn build(&mut self, slot: S) -> SR;
}
