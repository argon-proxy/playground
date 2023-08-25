use std::sync::atomic::{AtomicUsize, Ordering};

use argon::{config::ArgonConfig, ArgonTun};
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

    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("argon-{id}")
        })
        .build()
        .unwrap()
        .block_on(async move { run(config).await });

    if let Err(e) = result {
        println!("error: {e:?}");
    }
}

async fn run(config: ArgonConfig) -> Result<(), ArgonDriverError> {
    let mut tun = ArgonTun::new(config.tun)?;

    let rack_layout = TunRackSlot::build(config.rack.layout)?;

    let (mut entry_tx, mut rack, mut exit_rx) = TunRackBuilder::default()
        .add_async_slot((
            PingSlotProcessor::default(),
            SlotConfig::default().set_name("pingslot".to_owned()),
        ))
        .add_async_slot((
            PingSlotProcessor::default(),
            SlotConfig::default().set_name("pingslot".to_owned()),
        ))
        .add_sync_slot(LogSlotProcessor::default())
        .build()?;

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
