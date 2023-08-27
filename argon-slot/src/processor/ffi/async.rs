use std::ffi::c_void;

use async_trait::async_trait;

use super::{CAction, CData, CEvent};
use crate::processor::{
    r#async::AsyncSlotProcessor, SlotPacket, SlotProcessorResult,
};

#[repr(C)]
#[derive(Clone)]
pub struct CAsyncSlotProcessor {
    pub processor: *mut c_void,
}

unsafe impl Send for CAsyncSlotProcessor {}
unsafe impl Sync for CAsyncSlotProcessor {}

#[async_trait]
impl AsyncSlotProcessor for CAsyncSlotProcessor {
    type Event = CEvent;
    type Data = CData;
    type Action = CAction;

    async fn deserialize(
        &self,
        _packet: tun::TunPacket,
    ) -> SlotPacket<Self::Event, Self::Data> {
        todo!()
    }

    async fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action> {
        todo!()
    }

    async fn serialize(&self, action: Self::Action) -> tun::TunPacket {
        todo!()
    }

    async fn process(&self, data: Self::Data) -> SlotProcessorResult {
        todo!()
    }
}
