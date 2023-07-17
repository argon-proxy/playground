use std::sync::Arc;

use futures::{stream::FuturesUnordered, StreamExt};
use tokio::sync::RwLock;

use super::{SlotRunner, SlotRunnerConfig, SlotRunnerHandle};
use crate::{
    error::TunRackError,
    rack::{SlotReceiver, SlotSender},
    slot::{ParallelSlot, SlotPacket},
};

pub struct ParallelSlotRunnerConfig {}

impl<S> SlotRunnerConfig<S, ParallelSlotRunner<S>> for ParallelSlotRunnerConfig
where
    S: ParallelSlot,
{
    fn build(&mut self, slot: S) -> ParallelSlotRunner<S> {
        ParallelSlotRunner::new(slot)
    }
}

pub struct ParallelSlotContainer<S: ParallelSlot> {
    pub slot: Arc<RwLock<S>>,
}

pub struct ParallelSlotRunner<S: ParallelSlot> {
    pub container: ParallelSlotContainer<S>,
}

impl<S: ParallelSlot> SlotRunner<S> for ParallelSlotRunner<S> {
    fn new(slot: S) -> Self {
        Self {
            container: ParallelSlotContainer {
                slot: Arc::new(RwLock::new(slot)),
            },
        }
    }

    fn run(self, mut rx: SlotReceiver, tx: SlotSender, exit_tx: SlotSender) -> SlotRunnerHandle {
        let slot = self.container.slot;

        let handle = tokio::spawn(async move {
            let (worker_tx, worker_rx) = async_channel::bounded::<<S as ParallelSlot>::Data>(1024);

            let mut workers: FuturesUnordered<tokio::task::JoinHandle<Result<(), TunRackError>>> =
                FuturesUnordered::new();

            for _ in 0..8 {
                let slot = slot.clone();
                let tx = tx.clone();
                let exit_tx = exit_tx.clone();
                let worker_rx = worker_rx.clone();

                workers.push(tokio::spawn(async move {
                    loop {
                        let data = worker_rx.recv().await?;

                        let slotlock = slot.read().await;

                        let result = <S as ParallelSlot>::process(&slotlock, data).await;

                        for forward in result.forward {
                            tx.send(forward).await?;
                        }

                        for exit in result.exit {
                            exit_tx.send(exit).await?;
                        }
                    }
                }));
            }

            drop(worker_rx);

            loop {
                tokio::select! {
                    result = rx.recv() => {
                        let tun_packet = result.ok_or(TunRackError::SlotChannelClosed)?;

                        let mut slotlock = slot.write().await;

                        let packet = match <S as ParallelSlot>::deserialize(&mut slotlock, tun_packet).await {
                            Ok(packet) => packet,
                            Err(tun_packet) => {
                                tx.send(tun_packet).await?;
                                continue;
                            },
                        };

                        match packet {
                            SlotPacket::Event(event) => {
                                let actions = <S as ParallelSlot>::handle_event(&mut slotlock, event).await;

                                let slotlock = slotlock.downgrade();

                                for action in actions {
                                    exit_tx.send(<S as ParallelSlot>::serialize(&slotlock, action).await).await?;
                                }
                            },
                            SlotPacket::Data(data) => {
                                worker_tx.send(data).await.map_err(|_| TunRackError::SlotAsyncChannelSendError)?;
                            }
                        }
                    }

                    result = workers.next() => {
                        return if let Some(result) = result {
                            result?
                        } else {
                            Err(TunRackError::SlotChannelClosed)
                        };
                    }
                }
            }
        });

        SlotRunnerHandle { handle }
    }
}
