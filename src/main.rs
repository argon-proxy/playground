use std::sync::atomic::{AtomicUsize, Ordering};

use clap::Parser;

mod cli;
use cli::Cli;

mod device;
use device::Tun;

mod error;
use error::TunRackError;
use futures::StreamExt;
use rack::TunRack;

mod rack;

mod slots;
use slots::ping::PingSlotBuilder;

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

    let mut rack = TunRack::new(cli.channel_size);

    rack.add_slot(Box::new(PingSlotBuilder::new()));

    while let Some(packet) = tun.next().await {
        if let Ok(packet) = packet {
            rack.process(packet).await?
        }
    }

    Ok(())
}
