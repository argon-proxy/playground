use std::collections::HashMap;

use argon::config::{ArgonRackSlotConfig, ArgonRuntimeType, ArgonSlotConfig};
use argon_plugin_registry::{ArgonPluginRegistry, ArgonPluginRegistryError};
use argon_slot::{Slot, SyncSlot};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TunRackLayoutError {
    #[error("SlotMissing")]
    SlotMissing,
}

pub struct TunRackSlot<'pr> {
    pub slot_builder: Box<
        dyn FnOnce() -> Result<Box<dyn Slot>, ArgonPluginRegistryError> + 'pr,
    >,
    pub sink: bool,
}

impl<'pr> TunRackSlot<'pr> {
    pub fn build(
        layout: Vec<ArgonRackSlotConfig>,
        slots: HashMap<String, ArgonSlotConfig>,
        plugin_registry: &'pr ArgonPluginRegistry,
    ) -> Result<Vec<TunRackSlot<'pr>>, TunRackLayoutError> {
        let mut result = Vec::<TunRackSlot>::with_capacity(layout.len());

        for slot in layout {
            let slot_config = slots
                .get(&slot.slot)
                .ok_or(TunRackLayoutError::SlotMissing)?;

            result.push(TunRackSlot {
                slot_builder: create_slot_builder(
                    &slot_config.plugin,
                    &slot_config.runtime,
                    plugin_registry,
                ),
                sink: slot.sink,
            });
        }

        Ok(result)
    }
}

fn create_slot_builder<'pr>(
    plugin_name: &String,
    runtime: &ArgonRuntimeType,
    plugin_registry: &'pr ArgonPluginRegistry,
) -> Box<dyn FnOnce() -> Result<Box<dyn Slot>, ArgonPluginRegistryError> + 'pr>
{
    let plugin_name = plugin_name.to_owned();

    match runtime {
        ArgonRuntimeType::Sync => Box::new(move || {
            Ok(plugin_registry
                .build_sync_slot(&plugin_name)
                .map(|p| Box::<SyncSlot<_>>::new((p).into()))?)
        }),
        ArgonRuntimeType::Async => todo!(),
    }
}
