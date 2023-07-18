use argon::slot::{AsyncSlot, SlotBuilder, SlotPacket, SlotProcessResult, SyncSlot};
use async_trait::async_trait;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

use super::PingSyncSlot;

#[derive(Default)]
pub struct PingAsyncSlotBuilder {}

impl SlotBuilder<PingAsyncSlot> for PingAsyncSlotBuilder {
    fn build(self) -> PingAsyncSlot {
        PingAsyncSlot { sync: PingSyncSlot {} }
    }
}

pub struct PingAsyncSlot {
    sync: PingSyncSlot,
}

#[async_trait]
impl AsyncSlot for PingAsyncSlot {
    type Event = ();
    type Data = (packet::ip::v4::Packet<Vec<u8>>, packet::icmp::echo::Packet<Vec<u8>>);
    type Action = ();

    async fn deserialize<'p>(
        slot: &RwLockReadGuard<'p, Self>,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket> {
        <PingSyncSlot as SyncSlot>::deserialize(&slot.sync, packet)
    }

    async fn handle_event<'p>(slot: &mut RwLockWriteGuard<'p, Self>, event: Self::Event) -> Vec<Self::Action> {
        <PingSyncSlot as SyncSlot>::handle_event(&mut slot.sync, event)
    }

    async fn serialize<'p>(slot: &RwLockReadGuard<'p, Self>, action: Self::Action) -> tun::TunPacket {
        <PingSyncSlot as SyncSlot>::serialize(&slot.sync, action)
    }

    async fn process<'p>(slot: &RwLockReadGuard<'p, Self>, data: Self::Data) -> SlotProcessResult {
        <PingSyncSlot as SyncSlot>::process(&slot.sync, data)
    }
}
