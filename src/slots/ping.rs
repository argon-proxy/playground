use crate::rack::{TunRackSlot, TunRackSlotBuilder, TunRackSlotReceiver, TunRackSlotSender};

pub struct PingSlotBuilder {}

impl PingSlotBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl TunRackSlotBuilder for PingSlotBuilder {
    fn build(
        self: Box<Self>,
        rx: crate::rack::TunRackSlotReceiver,
        tx: crate::rack::TunRackSlotSender,
    ) -> Box<dyn TunRackSlot> {
        Box::new(PingSlot::new(rx, tx))
    }
}

pub struct PingSlot {
    rx: TunRackSlotReceiver,
    tx: TunRackSlotSender,
}

impl PingSlot {
    pub fn new(rx: TunRackSlotReceiver, tx: TunRackSlotSender) -> Self {
        Self { rx, tx }
    }
}

impl TunRackSlot for PingSlot {}
