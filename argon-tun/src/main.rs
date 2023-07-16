use std::sync::atomic::{AtomicUsize, Ordering};

use argon::{
    error::TunRackError,
    rack::{slot::TunRackSequentialSlotRunnerConfig, TunRack},
    Tun,
};
use argon_slots::{log::LogSlotBuilder, ping::PingSlotBuilder};
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

    rack.add_sequential_slot(PingSlotBuilder::default(), TunRackSequentialSlotRunnerConfig {});
    rack.add_sequential_slot(LogSlotBuilder::default(), TunRackSequentialSlotRunnerConfig {});

    loop {
        tokio::select! {
            Some(result) = tun.next() => {
                let packet = result.map_err(TunRackError::TunIoError)?;

                rack.send(packet).await?;
            }

            option = rack_exit_rx.recv() => {
                if let Some(tun_packet) = option {
                    tun.send(tun_packet).await.map_err(TunRackError::TunIoError)?;
                } else {
                    return Err(TunRackError::TunRackChannelClosed);
                }
            }

            Some(result) = rack.next() => {
                // Consume any packet that goes through the tun_rack and does not get forwarded through the exit_tx
                drop(result?);
            }
        }
    }
}
