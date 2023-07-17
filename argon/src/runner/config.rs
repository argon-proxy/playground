use crate::runner::SlotRunner;

pub trait SlotRunnerConfig<S, SR>
where
    SR: SlotRunner<S>,
{
    fn build(&mut self, slot: S) -> SR;
}
