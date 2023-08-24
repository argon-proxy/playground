use std::sync::atomic::{AtomicUsize, Ordering};

use argon::{
    config::ArgonConfig,
    error::TunRackError,
    rack::{TunRackBuilder, TunRackSlot},
    slot::{worker::SlotWorkerError, AsyncSlot, SlotConfig, SyncSlot},
    Tun,
};
use argon_slots::{log::LogSlotProcessor, ping::PingSlotProcessor};
use clap::Parser;
use futures::{SinkExt, StreamExt};

mod cli;
use cli::Cli;

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

async fn run(config: ArgonConfig) -> Result<(), TunRackError> {
    let mut tun = Tun::new(config.tun)?;

    let rack_layout = TunRackSlot::build(config.rack.layout, config.slots)?;

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
                let packet = result.map_err(TunRackError::IoError)?;

                if !entry_tx.fire(packet)? {
                    println!("[warn] packet dropped");
                }
            }

            result = rack.next() => {
                let Some(result) = result else {
                    return Err(TunRackError::SlotWorkerError(SlotWorkerError::SlotChannelClosed))
                };

                // Consume any packet that goes through the tun_rack and does
                // not get forwarded through the exit_tx
                drop(result?);
            }

            option = exit_rx.next() => {
                if let Some(tun_packet) = option {
                    tun.send(tun_packet).await.map_err(TunRackError::IoError)?;
                } else {
                    return Err(TunRackError::SlotWorkerError(SlotWorkerError::SlotChannelClosed));
                }
            }
        }
    }
}
