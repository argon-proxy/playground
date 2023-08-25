use std::{ffi::c_void, ptr};

use argon_plugin::{argon_plugin, ArgonPlugin};
use argon_slot::processor::{
    ffi::{CAction, CActions, CData, CEvent, CSyncSlotProcessor},
    sync::SyncSlotProcessor,
    SlotPacket, SlotProcessorResult,
};

type Event = ();
type Data = tun::TunPacket;
type Action = ();

#[derive(Default)]
pub struct LogSlotProcessor {}

impl SyncSlotProcessor for LogSlotProcessor {
    type Event = Event;
    type Data = Data;
    type Action = Action;

    fn deserialize(
        &self,
        packet: tun::TunPacket,
    ) -> Result<SlotPacket<Self::Event, Self::Data>, tun::TunPacket> {
        Ok(SlotPacket::Data(packet))
    }

    fn handle_event(&mut self, _event: Self::Event) -> Vec<Self::Action> {
        unreachable!()
    }

    fn serialize(&self, _action: Self::Action) -> tun::TunPacket {
        unreachable!()
    }

    fn process(&self, data: Self::Data) -> SlotProcessorResult {
        println!("[logslot] {data:?}");

        SlotProcessorResult {
            forward: vec![data],
            exit: vec![],
        }
    }
}

fn cevent_to_event(cevent: CEvent) -> Event {
    ()
}

fn event_to_cevent(event: Event) -> CEvent {
    CEvent {
        data: ptr::null_mut(),
    }
}

fn cdata_to_data(cdata: CData) -> Data {
    unsafe { *Box::from_raw(cdata.data.cast::<Data>()) }
}

fn data_to_cdata(data: Data) -> CData {
    CData {
        data: Box::into_raw(Box::new(data)).cast::<c_void>(),
    }
}

fn caction_to_action(caction: CAction) -> Action {
    ()
}

fn action_to_caction(action: Action) -> CAction {
    CAction {
        data: ptr::null_mut(),
    }
}

extern "C" fn deserialize(
    p: *const c_void,
    packet: tun::TunPacket,
) -> Result<SlotPacket<CEvent, CData>, tun::TunPacket> {
    let p = unsafe { &*(p.cast::<LogSlotProcessor>()) };

    let result = p.deserialize(packet)?;

    Ok(match result {
        SlotPacket::Event(event) => SlotPacket::Event(event_to_cevent(event)),
        SlotPacket::Data(data) => SlotPacket::Data(data_to_cdata(data)),
    })
}

extern "C" fn handle_event(p: *mut c_void, cevent: CEvent) -> CActions {
    let p = unsafe { &mut *(p.cast::<LogSlotProcessor>()) };

    let result = p.handle_event(cevent_to_event(cevent));

    result
        .into_iter()
        .map(action_to_caction)
        .collect::<Vec<CAction>>()
        .into()
}

extern "C" fn serialize(p: *const c_void, caction: CAction) -> tun::TunPacket {
    let p = unsafe { &*(p.cast::<LogSlotProcessor>()) };

    p.serialize(caction_to_action(caction))
}

extern "C" fn process(p: *const c_void, cdata: CData) -> SlotProcessorResult {
    let p = unsafe { &*(p.cast::<LogSlotProcessor>()) };

    p.process(cdata_to_data(cdata))
}

#[derive(Default)]
pub struct LogSlotPlugin {}

impl ArgonPlugin for LogSlotPlugin {
    fn name(&self) -> &'static str {
        "argon/log"
    }

    fn build_sync_slot(&self) -> Option<CSyncSlotProcessor> {
        let p = Box::<LogSlotProcessor>::default();

        Some(CSyncSlotProcessor {
            processor: Box::into_raw(p).cast::<c_void>(),
            deserialize,
            handle_event,
            serialize,
            process,
        })
    }

    fn build_async_slot(&self) -> Option<()> {
        None
    }

    fn destroy(&self) {}
}

argon_plugin!(LogSlotPlugin, LogSlotPlugin::default);
