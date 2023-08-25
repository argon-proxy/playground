use std::collections::HashMap;

use argon::config::{ArgonRackSlotConfig, ArgonSlotConfig};
use argon_slot::{
    processor::{sync::SyncSlotProcessor, SlotPacket, SlotProcessorResult},
    Slot, SyncSlot,
};
use thiserror::Error;

pub struct DummySlot {}

impl SyncSlotProcessor for DummySlot {
    type Event = ();

    type Data = ();

    type Action = ();

    fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket> {
        todo!()
    }

    fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action> {
        todo!()
    }

    fn serialize(&self, action: Self::Action) -> tun::TunPacket {
        todo!()
    }

    fn process(&self, data: Self::Data) -> SlotProcessorResult {
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum TunRackLayoutError {
    #[error("SlotMissing")]
    SlotMissing,
}

pub struct TunRackSlot {
    pub slot_builder: Box<dyn Fn() -> Box<dyn Slot>>,
    pub sink: bool,
}

impl TunRackSlot {
    pub fn build(
        layout: Vec<ArgonRackSlotConfig>,
    ) -> Result<Vec<TunRackSlot>, TunRackLayoutError> {
        let mut result = Vec::<TunRackSlot>::with_capacity(layout.len());

        for slot in layout {
            result.push(TunRackSlot {
                slot_builder: Box::new(|| {
                    Box::<SyncSlot<_>>::new((DummySlot {}).into())
                }),
                sink: slot.sink,
            });
        }

        Ok(result)
    }
}
