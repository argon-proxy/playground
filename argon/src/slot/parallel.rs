use async_trait::async_trait;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

use super::{SlotPacket, SlotProcessResult, SlotRunnerConfig};
use crate::runner::{ParallelSlotRunner, SlotRunner};

#[async_trait]
pub trait ParallelSlot: Send + Sync + 'static {
    type Event: Send + Sync;
    type Data: Send + Sync;
    type Action: Send + Sync;

    async fn deserialize<'p>(
        slot: &mut RwLockWriteGuard<'p, Self>,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket>;

    async fn handle_event<'p>(slot: &mut RwLockWriteGuard<'p, Self>, event: Self::Event) -> Vec<Self::Action>;

    async fn serialize<'p>(slot: &RwLockReadGuard<'p, Self>, action: Self::Action) -> tun::TunPacket;

    async fn process<'p>(slot: &RwLockReadGuard<'p, Self>, data: Self::Data) -> SlotProcessResult;
}

pub struct ParallelSlotRunnerConfig {}

impl<S> SlotRunnerConfig<S, ParallelSlotRunner<S>> for ParallelSlotRunnerConfig
where
    S: ParallelSlot,
{
    fn build(&mut self, slot: S) -> ParallelSlotRunner<S> {
        ParallelSlotRunner::new(slot)
    }
}
