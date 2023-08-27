use std::ffi::c_void;

use super::{CAction, CActions, CData, CEvent};
use crate::processor::{
    sync::SyncSlotProcessor, SlotPacket, SlotProcessorResult,
};

#[repr(C)]
#[derive(Clone)]
pub struct CSyncSlotProcessor {
    pub processor: *mut c_void,

    pub deserialize: extern "C" fn(
        *const c_void,
        tun::TunPacket,
    ) -> SlotPacket<CEvent, CData>,

    pub handle_event: extern "C" fn(*mut c_void, CEvent) -> CActions,

    pub serialize: extern "C" fn(*const c_void, CAction) -> tun::TunPacket,

    pub process: extern "C" fn(*const c_void, CData) -> SlotProcessorResult,
}

unsafe impl Send for CSyncSlotProcessor {}
unsafe impl Sync for CSyncSlotProcessor {}

impl SyncSlotProcessor for CSyncSlotProcessor {
    type Event = CEvent;
    type Data = CData;
    type Action = CAction;

    fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> SlotPacket<Self::Event, Self::Data> {
        (self.deserialize)(self.processor, packet)
    }

    fn handle_event(&mut self, event: Self::Event) -> Vec<Self::Action> {
        let cactions = (self.handle_event)(self.processor, event);

        unsafe { Vec::from_raw_parts(cactions.0, cactions.1, cactions.2) }
    }

    fn serialize(&self, action: Self::Action) -> tun::TunPacket {
        (self.serialize)(self.processor, action)
    }

    fn process(&self, data: Self::Data) -> SlotProcessorResult {
        (self.process)(self.processor, data)
    }
}
