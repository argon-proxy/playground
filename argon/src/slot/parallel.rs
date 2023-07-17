use async_trait::async_trait;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

use super::{SlotPacket, SlotProcessResult};

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
