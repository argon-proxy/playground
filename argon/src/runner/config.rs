use crate::runner::SlotRunner;
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait SlotRunnerConfig<S, SR>: Default
where
    SR: SlotRunner<S>,
{
    fn build(&mut self, slot: S) -> SR;
}

pub struct SlotContainer<S> {
    pub slot: Arc<RwLock<S>>,
}
