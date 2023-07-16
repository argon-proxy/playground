use super::{TunRackSlotReceiver, TunRackSlotSender};

pub fn build_tunrack_channel(channel_size: usize) -> (TunRackSlotSender, TunRackSlotReceiver) {
    tokio::sync::mpsc::channel(channel_size)
}
