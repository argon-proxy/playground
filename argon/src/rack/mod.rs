use futures::{FutureExt, Stream};

use crate::{
    error::TunRackError,
    runner::{SlotRunner, SlotRunnerConfig, SlotRunnerError, SlotRunnerHandle},
    slot::SlotBuilder,
};

mod types;
pub use types::*;

mod util;
use util::build_tunrack_channel;

pub struct TunRack {
    racks: Vec<SlotRunnerHandle>,

    channel_size: usize,

    first_tx: SlotSender,
    last_rx: SlotReceiver,

    exit_tx: SlotSender,
}

impl TunRack {
    pub fn new(channel_size: usize) -> (Self, SlotReceiver) {
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

    pub fn add_slot<S, SB, SR, SRC>(&mut self, slot_builder: SB, mut runner_config: SRC) -> Result<(), TunRackError>
    where
        SB: SlotBuilder<S>,
        SR: SlotRunner<S>,
        SRC: SlotRunnerConfig<S, SR>,
    {
        let (slot_tx, mut slot_rx) = build_tunrack_channel(self.channel_size);

        std::mem::swap(&mut self.last_rx, &mut slot_rx);

        let runner = runner_config.build(slot_builder.build());

        self.racks.push(runner.run(slot_rx, slot_tx, self.exit_tx.clone())?);

        Ok(())
    }

    pub async fn send(&mut self, packet: tun::TunPacket) -> Result<(), SlotSendError> {
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
                    Ok(item) => item.err().unwrap_or(SlotRunnerError::SlotCrash).into(),
                    Err(e) => TunRackError::TokioJoinError(e),
                })));
            }
        }

        if let std::task::Poll::Ready(result) = self.last_rx.poll_recv(cx) {
            std::task::Poll::Ready(Some(
                result
                    .ok_or(SlotRunnerError::SlotCrash)
                    .map_err(Into::<TunRackError>::into),
            ))
        } else {
            std::task::Poll::Pending
        }
    }
}
