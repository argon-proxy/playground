use std::sync::atomic::{AtomicUsize, Ordering};

use argon::{config::ArgonConfig, ArgonTun};
use argon_plugin_registry::ArgonPluginRegistry;
use argon_rack::{TunRackBuilder, TunRackSlot};
use argon_slot::SlotConfig;
use argon_slot_log::LogSlotProcessor;
use argon_slot_ping::PingSlotProcessor;
use clap::Parser;
use futures::{SinkExt, StreamExt};

mod cli;
use cli::Cli;

mod error;
use error::ArgonDriverError;

fn main() {
    let cli = Cli::parse();

    let config = if let Some(config_file) = cli.config {
        let file = std::fs::read(config_file).unwrap();

        serde_json::from_slice(&file).unwrap()
    } else {
        ArgonConfig::default()
    };

    println!("config: {}", serde_json::to_string_pretty(&config).unwrap());

    let mut plugin_registry = ArgonPluginRegistry::default();

    if let Some(plugin_path) = cli.plugin_path {
        for path in std::fs::read_dir(&plugin_path).unwrap() {
            let path = path.unwrap();

            if path.file_name().to_str().unwrap().ends_with(".so") {
                plugin_registry.load_plugin(path.path()).unwrap();
            }
        }
    }

    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("argon-{id}")
        })
        .build()
        .unwrap()
        .block_on(async move { run(config, &mut plugin_registry).await });

    if let Err(e) = result {
        println!("error: {e:?}");
    }
}

async fn run(
    config: ArgonConfig,
    plugin_registry: &mut ArgonPluginRegistry,
) -> Result<(), ArgonDriverError> {
    let mut tun = ArgonTun::new(config.tun)?;

    let rack_layout =
        TunRackSlot::build(config.rack.layout, config.slots, plugin_registry)?;

    let mut rack_builder = TunRackBuilder::default();

    for slot in rack_layout {
        rack_builder.add_slot((slot.slot_builder)()?)
    }

    let (mut entry_tx, mut rack, mut exit_rx) = rack_builder.build()?;

    loop {
        tokio::select! {
            Some(result) = tun.next() => {
                let packet = result.map_err(ArgonDriverError::IoError)?;

                if !entry_tx.fire(packet).map_err(|_| ArgonDriverError::RackChannelClosed)? {
                    println!("[warn] packet dropped");
                }
            }

            result = rack.next() => {
                let Some(result) = result else {
                    return Err(ArgonDriverError::RackChannelClosed);
                };

                // Consume any packet that goes through the tun_rack and does
                // not get forwarded through the exit_tx
                drop(result?);
            }

            option = exit_rx.next() => {
                if let Some(tun_packet) = option {
                    tun.send(tun_packet).await.map_err(ArgonDriverError::IoError)?;
                } else {
                    return Err(ArgonDriverError::RackChannelClosed);
                }
            }
        }
    }
}
