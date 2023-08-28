use argon_slot::processor::ffi::CSyncSlotProcessor;

pub trait ArgonPlugin: Send + Sync {
    fn name(&self) -> &'static str;

    fn build_sync_slot(&self) -> Option<CSyncSlotProcessor>;

    fn build_async_slot(&self) -> Option<()>;

    fn destroy(&self);
}

#[macro_export]
macro_rules! argon_plugin {
    (
        $name:expr,
        $type:ty,
        $constructor:path,
        $processor_sync_type:ty,
        $processor_event_type:ty,
        $processor_data_type:ty,
        $processor_action_type:ty
    ) => {
        use argon_plugin::ArgonPlugin;
        use argon_slot::processor::ffi::{
            CAction, CActions, CData, CEvent, CSyncSlotProcessor,
        };

        #[no_mangle]
        pub extern "C" fn _argon_plugin_create(
        ) -> *mut dyn ::argon_plugin::ArgonPlugin {
            let plugin: $type = $constructor();

            let plugin_box: Box<dyn ::argon_plugin::ArgonPlugin> =
                Box::new(plugin);

            Box::into_raw(plugin_box)
        }

        impl ArgonPlugin for $type {
            fn name(&self) -> &'static str {
                $name
            }

            fn build_sync_slot(&self) -> Option<CSyncSlotProcessor> {
                let p = Box::<$processor_sync_type>::default();

                Some(CSyncSlotProcessor {
                    processor: Box::into_raw(p).cast::<std::ffi::c_void>(),
                    deserialize: sync_deserialize,
                    handle_event: sync_handle_event,
                    serialize: sync_serialize,
                    process: sync_process,
                })
            }

            fn build_async_slot(&self) -> Option<()> {
                None
            }

            fn destroy(&self) {}
        }

        extern "C" fn sync_deserialize(
            p: *const std::ffi::c_void,
            packet: tun::TunPacket,
        ) -> SlotPacket<CEvent, CData> {
            let p = unsafe { &*(p.cast::<$processor_sync_type>()) };

            let result = SyncSlotProcessor::deserialize(p, packet);

            match result {
                SlotPacket::Event(event) => {
                    SlotPacket::Event(event_to_cevent(event))
                },
                SlotPacket::Data(data) => SlotPacket::Data(data_to_cdata(data)),
                SlotPacket::Next(tun_packet) => SlotPacket::Next(tun_packet),
            }
        }

        extern "C" fn sync_handle_event(
            p: *mut std::ffi::c_void,
            cevent: CEvent,
        ) -> CActions {
            let p = unsafe { &mut *(p.cast::<$processor_sync_type>()) };

            let result =
                SyncSlotProcessor::handle_event(p, cevent_to_event(cevent));

            result
                .into_iter()
                .map(action_to_caction)
                .collect::<Vec<CAction>>()
                .into()
        }

        extern "C" fn sync_serialize(
            p: *const std::ffi::c_void,
            caction: CAction,
        ) -> tun::TunPacket {
            let p = unsafe { &*(p.cast::<$processor_sync_type>()) };

            SyncSlotProcessor::serialize(p, caction_to_action(caction))
        }

        extern "C" fn sync_process(
            p: *const std::ffi::c_void,
            cdata: CData,
        ) -> SlotProcessorResult {
            let p = unsafe { &*(p.cast::<$processor_sync_type>()) };

            SyncSlotProcessor::process(p, cdata_to_data(cdata))
        }

        fn cevent_to_event(cevent: CEvent) -> $processor_event_type {
            unsafe { *Box::from_raw(cevent.data.cast()) }
        }

        fn event_to_cevent(event: $processor_event_type) -> CEvent {
            CEvent {
                data: Box::into_raw(Box::new(event)).cast(),
            }
        }

        fn cdata_to_data(cdata: CData) -> $processor_data_type {
            unsafe { *Box::from_raw(cdata.data.cast()) }
        }

        fn data_to_cdata(data: $processor_data_type) -> CData {
            CData {
                data: Box::into_raw(Box::new(data)).cast(),
            }
        }

        fn caction_to_action(caction: CAction) -> $processor_action_type {
            unsafe { *Box::from_raw(caction.data.cast()) }
        }

        fn action_to_caction(action: $processor_action_type) -> CAction {
            CAction {
                data: Box::into_raw(Box::new(action)).cast(),
            }
        }
    };
}
