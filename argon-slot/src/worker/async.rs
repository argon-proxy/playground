use std::sync::Arc;

use argon::rotary::{RotaryCanon, RotaryTarget};
use futures::StreamExt;
use tokio::sync::RwLock;

use super::SlotWorkerError;
use crate::{
    processor::{r#async::AsyncSlotProcessor, SlotPacket},
    SlotConfig,
};

pub async fn run_worker<SP>(
    processor: Arc<RwLock<SP>>,
    config: SlotConfig,
    mut entry_rx: RotaryTarget,
    mut next_tx: RotaryCanon,
    mut exit_tx: RotaryCanon,
) -> Result<(), SlotWorkerError>
where
    SP: AsyncSlotProcessor,
{
    while let Some(tun_packet) = entry_rx.next().await {
        let processor_lock = processor.read().await;

        let packet = match <SP as AsyncSlotProcessor>::deserialize(
            &processor_lock,
            tun_packet,
        )
        .await
        {
            Ok(packet) => packet,
            Err(tun_packet) => {
                if !next_tx.fire(tun_packet)? {
                    println!("[{}][warn] dropped packet", config.name);
                }

                continue;
            },
        };

        match packet {
            SlotPacket::Event(event) => {
                drop(processor_lock);

                let mut processor_lock = processor.write().await;

                let actions = <SP as AsyncSlotProcessor>::handle_event(
                    &mut processor_lock,
                    event,
                )
                .await;

                let processor_lock = processor_lock.downgrade();

                for action in actions {
                    if !exit_tx.fire(
                        <SP as AsyncSlotProcessor>::serialize(
                            &processor_lock,
                            action,
                        )
                        .await,
                    )? {
                        println!("[{}][warn] dropped packet", config.name);
                    }
                }
            },
            SlotPacket::Data(data) => {
                let result =
                    <SP as AsyncSlotProcessor>::process(&processor_lock, data)
                        .await;

                for forward in result.forward {
                    if !next_tx.fire(forward)? {
                        println!("[{}][warn] dropped packet", config.name);
                    }
                }

                for exit in result.exit {
                    if !exit_tx.fire(exit)? {
                        println!("[{}][warn] dropped packet", config.name);
                    }
                }
            },
        }
    }

    Ok(())
}
