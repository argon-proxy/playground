use argon_slot::processor::ffi::CSyncSlotProcessor;

pub trait ArgonPlugin: Send + Sync {
    fn name(&self) -> &'static str;

    fn build_sync_slot(&self) -> Option<CSyncSlotProcessor>;

    fn build_async_slot(&self) -> Option<()>;

    fn destroy(&self);
}

#[macro_export]
macro_rules! argon_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _argon_plugin_create(
        ) -> *mut dyn ::argon_plugin::ArgonPlugin {
            let plugin: $plugin_type = $constructor();

            let plugin_box: Box<dyn ::argon_plugin::ArgonPlugin> =
                Box::new(plugin);

            Box::into_raw(plugin_box)
        }
    };
}
