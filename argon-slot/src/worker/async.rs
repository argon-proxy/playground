use argon::rotary::{RotaryCanon, RotaryTarget};
use futures::StreamExt;

use super::SlotWorkerError;
use crate::{
    processor::{r#async::AsyncSlotProcessor, SlotPacket},
    SlotConfig,
};

pub async fn run_worker<SP>(
    mut processor: SP,
    config: SlotConfig,
    mut entry_rx: RotaryTarget,
    mut next_tx: RotaryCanon,
    mut exit_tx: RotaryCanon,
) -> Result<(), SlotWorkerError>
where
    SP: AsyncSlotProcessor,
{
    while let Some(tun_packet) = entry_rx.next().await {
        let packet = processor.deserialize(tun_packet).await;

        match packet {
            SlotPacket::Event(event) => {
                let actions = processor.handle_event(event).await;

                for action in actions {
                    if !exit_tx.fire(processor.serialize(action).await)? {
                        println!("[{}][warn] dropped packet", config.name);
                    }
                }
            },
            SlotPacket::Data(data) => {
                let result = processor.process(data).await;

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
            SlotPacket::Forward(tun_packet) => {
                if !next_tx.fire(tun_packet)? {
                    println!("[{}][warn] dropped packet", config.name);
                }
            },
        }
    }

    Ok(())
}
