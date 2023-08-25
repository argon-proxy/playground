use std::{
    collections::{hash_map::Entry, HashMap},
    ffi::OsStr,
};

use argon_plugin::ArgonPlugin;
use argon_slot::processor::ffi::CSyncSlotProcessor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArgonPluginRegistryError {
    #[error("PluginAlreadyLoaded")]
    PluginAlreadyLoaded,

    #[error("PluginNotFound")]
    PluginNotFound,

    #[error("PluginCannotBuildSyncSlot")]
    PluginCannotBuildSyncSlot,

    #[error("PluginCannotBuildAsyncSlot")]
    PluginCannotBuildAsyncSlot,

    #[error("InvalidPlugin")]
    InvalidPlugin(libloading::Error),

    #[error("PluginCouldNotBeLoaded")]
    PluginCouldNotBeLoaded(libloading::Error),
}

#[derive(Default)]
pub struct ArgonPluginRegistry {
    plugins: HashMap<String, Box<dyn ArgonPlugin>>,
    libs: Vec<libloading::Library>,
}

impl ArgonPluginRegistry {
    pub fn load_plugin<P: AsRef<OsStr>>(
        &mut self,
        filename: P,
    ) -> Result<(), ArgonPluginRegistryError> {
        type ArgonPluginCreate = unsafe fn() -> *mut dyn ArgonPlugin;

        let lib = unsafe { libloading::Library::new(filename.as_ref()) }
            .map_err(ArgonPluginRegistryError::PluginCouldNotBeLoaded)?;

        let plugin_constructor: libloading::Symbol<ArgonPluginCreate> =
            unsafe { lib.get(b"_argon_plugin_create") }
                .map_err(ArgonPluginRegistryError::InvalidPlugin)?;

        let plugin = unsafe { Box::from_raw(plugin_constructor()) };
        let plugin_name = plugin.name();

        match self.plugins.entry(plugin_name.to_owned()) {
            Entry::Occupied(_) => {
                Err(ArgonPluginRegistryError::PluginAlreadyLoaded)
            },
            Entry::Vacant(v) => {
                self.libs.push(lib);
                v.insert(plugin);

                Ok(())
            },
        }
    }

    pub fn build_sync_slot(
        &self,
        plugin_name: &str,
    ) -> Result<CSyncSlotProcessor, ArgonPluginRegistryError> {
        match self.plugins.get(plugin_name) {
            Some(plugin) => plugin
                .build_sync_slot()
                .ok_or(ArgonPluginRegistryError::PluginCannotBuildSyncSlot),
            None => Err(ArgonPluginRegistryError::PluginNotFound),
        }
    }

    pub fn build_async_slot(
        &self,
        plugin_name: &str,
    ) -> Result<(), ArgonPluginRegistryError> {
        match self.plugins.get(plugin_name) {
            Some(plugin) => plugin
                .build_async_slot()
                .ok_or(ArgonPluginRegistryError::PluginCannotBuildAsyncSlot),
            None => Err(ArgonPluginRegistryError::PluginNotFound),
        }
    }
}

impl Drop for ArgonPluginRegistry {
    fn drop(&mut self) {
        for (_, plugin) in self.plugins.drain() {
            plugin.destroy();
            let _ = Box::into_raw(plugin);
        }

        for lib in self.libs.drain(..) {
            drop(lib);
        }
    }
}
