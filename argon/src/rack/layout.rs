use std::collections::HashMap;

use thiserror::Error;

use crate::{
    config::{ArgonRackSlotConfig, ArgonSlotConfig},
    slot::{Slot, SyncSlot, SyncSlotProcessor},
};

pub struct DummySlot {}

impl SyncSlotProcessor for DummySlot {
    type Event = ();

    type Data = ();

    type Action = ();

    fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> Result<crate::slot::SlotPacket<Self::Event, Self::Data>, tun::TunPacket>
    {
        todo!()
    }

    fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action> {
        todo!()
    }

    fn serialize(&self, action: Self::Action) -> tun::TunPacket {
        todo!()
    }

    fn process(&self, data: Self::Data) -> crate::slot::SlotProcessResult {
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
        slots: HashMap<String, ArgonSlotConfig>,
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
