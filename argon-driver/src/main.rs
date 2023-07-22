use std::sync::atomic::{AtomicUsize, Ordering};

use argon::{
    error::TunRackError,
    rack::TunRackBuilder,
    slot::{worker::SlotWorkerError, AsyncSlot, SyncSlot},
    Tun,
};
use argon_slots::{log::LogSlotProcessor, ping::PingSlotProcessor};
use clap::Parser;
use futures::{SinkExt, StreamExt};

mod cli;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("argon-{id}")
        })
        .build()
        .unwrap()
        .block_on(async move { run(cli).await });

    if let Err(e) = result {
        println!("error: {e:?}");
    }
}

async fn run(cli: Cli) -> Result<(), TunRackError> {
    let mut tun = Tun::new(cli.mtu)?;

    let (mut entry_tx, mut rack, mut exit_rx) = TunRackBuilder::default()
        .add_slot::<AsyncSlot<_>>(PingSlotProcessor::default())
        .add_slot::<SyncSlot<_>>(LogSlotProcessor::default())
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
