use std::sync::atomic::{AtomicUsize, Ordering};

use argon::{
    error::TunRackError,
    rack::TunRack,
    runner::{ParallelSlotRunnerConfig, SequentialSlotRunnerConfig},
    Tun,
};
use argon_slots::{log::LogSlotBuilder, ping::PingParallelSlotBuilder};
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
            format!("argon-{}", id)
        })
        .build()
        .unwrap()
        .block_on(async move { run(cli).await });

    if let Err(e) = result {
        println!("error: {:?}", e);
    }
}

async fn run(cli: Cli) -> Result<(), TunRackError> {
    let mut tun = Tun::new(cli.mtu)?;

    let (mut rack, mut rack_exit_rx) = TunRack::new(cli.channel_size);

    rack.add_slot(PingParallelSlotBuilder::default(), ParallelSlotRunnerConfig::default());
    rack.add_slot(LogSlotBuilder::default(), SequentialSlotRunnerConfig::default());

    loop {
        tokio::select! {
            Some(result) = tun.next() => {
                let packet = result.map_err(TunRackError::IoError)?;

                rack.send(packet).await?;
            }

            option = rack_exit_rx.recv() => {
                if let Some(tun_packet) = option {
                    tun.send(tun_packet).await.map_err(TunRackError::IoError)?;
                } else {
                    return Err(TunRackError::SlotChannelClosed);
                }
            }

            result = rack.next() => {
                let result = if let Some(packet) = result {
                    packet
                } else {
                    return Err(TunRackError::SlotChannelClosed);
                };

                // Consume any packet that goes through the tun_rack and does not get forwarded through the exit_tx
                drop(result?);
            }
        }
    }
}