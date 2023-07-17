use super::{SlotReceiver, SlotSender};

pub fn build_tunrack_channel(channel_size: usize) -> (SlotSender, SlotReceiver) {
    tokio::sync::mpsc::channel(channel_size)
}
