use std::{
    collections::{hash_map::Entry, HashMap},
    ffi::OsStr,
};

use argon::slot::{AbiAsyncSlotProcessor, AbiSyncSlotProcessor};
use argon_plugin::ArgonPlugin;
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
}

pub struct ArgonPluginRegistry {
    plugins: HashMap<String, Box<dyn ArgonPlugin>>,
    libs: Vec<libloading::Library>,
}

impl ArgonPluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            libs: Vec::new(),
        }
    }

    pub unsafe fn load_plugin<P: AsRef<OsStr>>(
        &mut self,
        filename: P,
    ) -> Result<(), ArgonPluginRegistryError> {
        type ArgonPluginCreate = unsafe fn() -> *mut dyn ArgonPlugin;

        let lib = libloading::Library::new(filename.as_ref()).unwrap();

        let plugin_constructor: libloading::Symbol<ArgonPluginCreate> =
            lib.get(b"_argon_plugin_create").unwrap();

        let plugin = Box::from_raw(plugin_constructor());
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
    ) -> Result<AbiSyncSlotProcessor, ArgonPluginRegistryError> {
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
    ) -> Result<AbiAsyncSlotProcessor, ArgonPluginRegistryError> {
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
