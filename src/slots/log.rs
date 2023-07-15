use crate::rack::{
    slot::TunRackSlot,
    slot_builder::TunRackSlotBuilder,
    slot_handle::TunRackSlotHandle,
    TunRackSlotReceiver,
    TunRackSlotSender,
};

pub struct LogSlotBuilder {}

impl LogSlotBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl TunRackSlotBuilder<LogSlot> for LogSlotBuilder {
    fn build(self, rx: TunRackSlotReceiver, tx: TunRackSlotSender, exit_tx: TunRackSlotSender) -> LogSlot {
        LogSlot::new(rx, tx, exit_tx)
    }
}

pub struct LogSlot {
    rx: TunRackSlotReceiver,
    tx: TunRackSlotSender,
    exit_tx: TunRackSlotSender,
}

impl LogSlot {
    pub fn new(rx: TunRackSlotReceiver, tx: TunRackSlotSender, exit_tx: TunRackSlotSender) -> Self {
        Self { rx, tx, exit_tx }
    }
}

impl TunRackSlot for LogSlot {
    fn run(self) -> TunRackSlotHandle {
        let mut rx = self.rx;
        let tx = self.tx;

        let handle = tokio::spawn(async move {
            while let Some(packet) = rx.recv().await {
                println!("LogSlot got packet {:?}", packet);

                tx.send(packet).await?;
            }

            Ok(())
        });

        TunRackSlotHandle::new(handle)
    }
}
