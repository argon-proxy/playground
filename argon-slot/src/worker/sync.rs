use argon::rotary::{RotaryCanon, RotaryTarget};
use futures::StreamExt;

use super::SlotWorkerError;
use crate::{
    processor::{sync::SyncSlotProcessor, SlotPacket},
    SlotConfig,
};

pub async fn run_worker<SP>(
    mut processor: SP,
    config: SlotConfig,
    mut entry_rx: RotaryTarget,
    mut exit_tx: RotaryCanon,
    mut next_tx: RotaryCanon,
    mut forward_tx: RotaryCanon,
) -> Result<(), SlotWorkerError>
where
    SP: SyncSlotProcessor,
{
    while let Some(tun_packet) = entry_rx.next().await {
        let packet = processor.deserialize(tun_packet);

        match packet {
            SlotPacket::Event(event) => {
                let actions = processor.handle_event(event);

                for action in actions {
                    if !exit_tx.fire(processor.serialize(action))? {
                        println!("[{}][warn] dropped packet", config.name);
                    }
                }
            },
            SlotPacket::Data(data) => {
                let result = processor.process(data);

                for forward in result.forward {
                    if !forward_tx.fire(forward)? {
                        println!("[{}][warn] dropped packet", config.name);
                    }
                }

                for exit in result.exit {
                    if !exit_tx.fire(exit)? {
                        println!("[{}][warn] dropped packet", config.name);
                    }
                }
            },
            SlotPacket::Next(tun_packet) => {
                if !next_tx.fire(tun_packet)? {
                    println!("[{}][warn] dropped packet", config.name);
                }
            },
        }
    }

    Ok(())
}
