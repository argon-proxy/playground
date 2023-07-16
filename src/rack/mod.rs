use futures::{FutureExt, Stream};

use self::{
    runner::{SlotRunnerSequential, TunRackSlotRunner},
    slot::{TunRackSlot, TunRackSlotBuilder, TunRackSlotConfig, TunRackSlotHandle},
    util::build_tunrack_channel,
};
use crate::error::TunRackError;

pub mod slot;

pub mod runner;
mod util;

pub type TunRackSlotSender = tokio::sync::mpsc::Sender<tun::TunPacket>;
pub type TunRackSlotReceiver = tokio::sync::mpsc::Receiver<tun::TunPacket>;
pub type TunRackSendError = tokio::sync::mpsc::error::SendError<tun::TunPacket>;

pub struct TunRack {
    racks: Vec<TunRackSlotHandle>,

    channel_size: usize,

    first_tx: TunRackSlotSender,
    last_rx: TunRackSlotReceiver,

    exit_tx: TunRackSlotSender,
}

impl TunRack {
    pub fn new(channel_size: usize) -> (Self, TunRackSlotReceiver) {
        let (first_tx, last_rx) = build_tunrack_channel(channel_size);
        let (exit_tx, exit_rx) = build_tunrack_channel(channel_size);

        (
            Self {
                channel_size,
                racks: Vec::new(),
                first_tx,
                last_rx,

                exit_tx,
            },
            exit_rx,
        )
    }

    pub fn add_slot<ST, SB, SR>(&mut self, slot_builder: SB, mut slot_config: TunRackSlotConfig<ST, SR>)
    where
        ST: TunRackSlot,
        SB: TunRackSlotBuilder<ST>,
        SR: TunRackSlotRunner<ST>,
    {
        let (slot_tx, mut slot_rx) = build_tunrack_channel(self.channel_size);

        std::mem::swap(&mut self.last_rx, &mut slot_rx);

        let runner = slot_config.configure(slot_builder.build());

        self.racks.push(runner.run(slot_rx, slot_tx, self.exit_tx.clone()));
    }

    pub async fn send(&mut self, packet: tun::TunPacket) -> Result<(), TunRackSendError> {
        self.first_tx.send(packet).await
    }
}

impl Stream for TunRack {
    type Item = Result<tun::TunPacket, TunRackError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        for slot in &mut self.racks {
            if let std::task::Poll::Ready(result) = slot.handle.poll_unpin(cx) {
                return std::task::Poll::Ready(Some(Err(match result {
                    Ok(item) => item.err().unwrap_or(TunRackError::TunRackSlotCrash),
                    Err(e) => TunRackError::TokioJoinError(e),
                })));
            }
        }

        if let std::task::Poll::Ready(result) = self.last_rx.poll_recv(cx) {
            std::task::Poll::Ready(Some(result.ok_or(TunRackError::TunRackSlotCrash)))
        } else {
            std::task::Poll::Pending
        }
    }
}
