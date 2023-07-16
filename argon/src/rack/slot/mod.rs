use async_trait::async_trait;

mod builder;
pub use builder::TunRackSlotBuilder;

mod config;
pub use config::{TunRackRunnerConfig, TunRackSequentialSlotRunnerConfig};

mod event;
pub use event::SlotPacket;

mod handle;
pub use handle::TunRackSlotHandle;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct TunRackSlotProcessResult {
    pub forward: Vec<tun::TunPacket>,
    pub exit: Vec<tun::TunPacket>,
}

pub trait TunRackSequentialSlot: Send + Sync + 'static {
    type Event: Send + Sync;
    type Data: Send + Sync;
    type Action: Send + Sync;

    fn deserialize(&self, packet: tun::TunPacket) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket>;

    fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action>;

    fn serialize(&self, action: Self::Action) -> tun::TunPacket;

    fn process(&self, data: Self::Data) -> TunRackSlotProcessResult;
}

pub struct TunRackParallelSlotContainer<S: TunRackParallelSlot> {
    pub slot: RwLock<S>,
}

#[async_trait]
pub trait TunRackParallelSlot: Send + Sync + 'static {
    type Event: Send + Sync;
    type Data: Send + Sync;
    type Action: Send + Sync;

    async fn deserialize<'p>(
        slot: RwLockReadGuard<'p, Self>,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket>;

    async fn handle_event<'p>(slot: RwLockWriteGuard<'p, Self>, event: Self::Event) -> Vec<Self::Action>;

    async fn serialize<'p>(slot: RwLockReadGuard<'p, Self>, action: Self::Action) -> tun::TunPacket;

    async fn process<'p>(slot: RwLockReadGuard<'p, Self>, data: Self::Data) -> TunRackSlotProcessResult;
}
