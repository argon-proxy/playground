use async_trait::async_trait;

mod builder;
pub use builder::TunRackSlotBuilder;

mod config;
pub use config::TunRackSlotConfig;

mod event;
pub use event::SlotPacket;

mod handle;
pub use handle::TunRackSlotHandle;

pub struct TunRackSlotProcessResult {
    pub forward: Vec<tun::TunPacket>,
    pub exit: Vec<tun::TunPacket>,
}

#[async_trait]
pub trait TunRackSlot: Send + Sync + 'static {
    type Event: Send + Sync;
    type Data: Send + Sync;
    type Action: Send + Sync;

    async fn deserialize(&self, packet: tun::TunPacket) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket>;

    async fn handle_event(&self, event: Self::Event) -> Vec<Self::Action>;

    async fn serialize(&self, action: Self::Action) -> tun::TunPacket;

    async fn process(&self, data: Self::Data) -> TunRackSlotProcessResult;
}
