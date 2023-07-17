use argon::slot::{ParallelSlot, SequentialSlot, SlotBuilder, SlotPacket, SlotProcessResult};
use async_trait::async_trait;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

use super::PingSequentialSlot;

pub struct PingParallelSlotBuilder {}

impl Default for PingParallelSlotBuilder {
    fn default() -> Self {
        Self {}
    }
}

impl SlotBuilder<PingParallelSlot> for PingParallelSlotBuilder {
    fn build(self) -> PingParallelSlot {
        PingParallelSlot {
            sequential: PingSequentialSlot {},
        }
    }
}

pub struct PingParallelSlot {
    sequential: PingSequentialSlot,
}

#[async_trait]
impl ParallelSlot for PingParallelSlot {
    type Event = ();
    type Data = (packet::ip::v4::Packet<Vec<u8>>, packet::icmp::echo::Packet<Vec<u8>>);
    type Action = ();

    async fn deserialize<'p>(
        slot: &mut RwLockWriteGuard<'p, Self>,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket> {
        <PingSequentialSlot as SequentialSlot>::deserialize(&slot.sequential, packet)
    }

    async fn handle_event<'p>(slot: &mut RwLockWriteGuard<'p, Self>, event: Self::Event) -> Vec<Self::Action> {
        <PingSequentialSlot as SequentialSlot>::handle_event(&mut slot.sequential, event)
    }

    async fn serialize<'p>(slot: &RwLockReadGuard<'p, Self>, action: Self::Action) -> tun::TunPacket {
        <PingSequentialSlot as SequentialSlot>::serialize(&slot.sequential, action)
    }

    async fn process<'p>(slot: &RwLockReadGuard<'p, Self>, data: Self::Data) -> SlotProcessResult {
        <PingSequentialSlot as SequentialSlot>::process(&slot.sequential, data)
    }
}
