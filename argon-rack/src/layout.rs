use std::collections::HashMap;

use argon::config::{ArgonRackSlotConfig, ArgonRuntimeType, ArgonSlotConfig};
use argon_plugin_registry::{ArgonPluginRegistry, ArgonPluginRegistryError};
use argon_slot::{Slot, SlotConfig, SyncSlot};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TunRackLayoutError {
    #[error("SlotMissing")]
    SlotMissing,
}

pub struct TunRackLayoutSlot<'pr> {
    slot_builder: Box<
        dyn FnOnce() -> Result<Box<dyn Slot>, ArgonPluginRegistryError> + 'pr,
    >,
    sink: bool,
    subslots: Vec<Self>,
}

pub struct TunRackLayoutSlotProperties<'pr> {
    pub sink: bool,
    pub subslots: Vec<TunRackLayoutSlot<'pr>>,
}

impl<'pr> TunRackLayoutSlot<'pr> {
    pub fn build(
        self,
    ) -> Result<
        (Box<dyn Slot>, TunRackLayoutSlotProperties<'pr>),
        ArgonPluginRegistryError,
    > {
        let slot = (self.slot_builder)()?;

        Ok((slot, TunRackLayoutSlotProperties {
            sink: self.sink,
            subslots: self.subslots,
        }))
    }
}

pub struct TunRackLayout<'pr> {
    pub slots: Vec<TunRackLayoutSlot<'pr>>,
}

impl<'pr> TunRackLayout<'pr> {
    pub fn build(
        layout: Vec<ArgonRackSlotConfig>,
        slotmap: HashMap<String, ArgonSlotConfig>,
        plugin_registry: &'pr ArgonPluginRegistry,
    ) -> Result<TunRackLayout, TunRackLayoutError> {
        Ok(TunRackLayout {
            slots: Self::build_slots(layout, &slotmap, plugin_registry)?,
        })
    }

    fn build_slots(
        layout: Vec<ArgonRackSlotConfig>,
        slotmap: &HashMap<String, ArgonSlotConfig>,
        plugin_registry: &'pr ArgonPluginRegistry,
    ) -> Result<Vec<TunRackLayoutSlot<'pr>>, TunRackLayoutError> {
        let mut slots = Vec::<TunRackLayoutSlot>::with_capacity(layout.len());

        for slot in layout {
            let slot_config = slotmap
                .get(&slot.slot)
                .ok_or(TunRackLayoutError::SlotMissing)?;

            slots.push(TunRackLayoutSlot {
                slot_builder: create_slot_builder(slot_config, plugin_registry),
                sink: slot.sink,
                subslots: Self::build_slots(
                    slot.subslots,
                    slotmap,
                    plugin_registry,
                )?,
            });
        }

        Ok(slots)
    }
}

fn create_slot_builder<'pr>(
    config: &ArgonSlotConfig,
    plugin_registry: &'pr ArgonPluginRegistry,
) -> Box<dyn FnOnce() -> Result<Box<dyn Slot>, ArgonPluginRegistryError> + 'pr>
{
    let plugin_name = config.plugin.clone();

    let slot_config = SlotConfig::from(config);

    match config.runtime {
        ArgonRuntimeType::Sync => Box::new(move || {
            Ok(plugin_registry
                .build_sync_slot(&plugin_name)
                .map(|p| Box::<SyncSlot<_>>::new((p, slot_config).into()))?)
        }),
        ArgonRuntimeType::Async => todo!(),
    }
}
