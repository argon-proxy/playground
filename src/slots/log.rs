use crate::rack::{TunRackSlot, TunRackSlotBuilder, TunRackSlotHandle, TunRackSlotReceiver, TunRackSlotSender};

pub struct LogSlotBuilder {}

impl LogSlotBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl TunRackSlotBuilder for LogSlotBuilder {
    fn build(
        self: Box<Self>,
        rx: TunRackSlotReceiver,
        tx: TunRackSlotSender,
        exit_tx: TunRackSlotSender,
    ) -> Box<dyn TunRackSlot> {
        Box::new(LogSlot::new(rx, tx, exit_tx))
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
    fn run(self: Box<Self>) -> TunRackSlotHandle {
        let mut rx = self.rx;
        let tx = self.tx;
        let exit_tx = self.exit_tx;

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
