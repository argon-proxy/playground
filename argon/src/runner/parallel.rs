use std::sync::Arc;

use futures::{stream::FuturesUnordered, StreamExt};
use tokio::sync::RwLock;

use super::{SlotRunner, SlotRunnerConfig, SlotRunnerHandle};
use crate::{
    error::TunRackError,
    rack::{SlotReceiver, SlotSender},
    slot::{ParallelSlot, SlotPacket},
};

use flume::{Sender, Receiver};

type WorkerTx<S> = flume::Sender<<S as ParallelSlot>::Data>;
type WorkerRx<S> = flume::Receiver<<S as ParallelSlot>::Data>;

const WORKER_DEFAULT_CHANNEL_SIZE: usize = 2048;

pub struct ParallelSlotRunnerConfig {
    pub workers: usize,
    pub worker_channel_size: usize,
}

impl Default for ParallelSlotRunnerConfig {
    fn default() -> Self {
        Self {
            workers: num_cpus::get(),
            worker_channel_size: WORKER_DEFAULT_CHANNEL_SIZE,
        }
    }
}

impl<S> SlotRunnerConfig<S, ParallelSlotRunner<S>> for ParallelSlotRunnerConfig
where
    S: ParallelSlot,
{
    fn build(&mut self, slot: S) -> ParallelSlotRunner<S> {
        let (worker_tx, worker_rx) = flume::bounded(self.worker_channel_size);

        let worker_rxs = (0..self.workers).map(|_| worker_rx.clone()).collect::<Vec<_>>();

        ParallelSlotRunner::new(slot, worker_tx, worker_rxs)
    }
}

pub struct ParallelSlotContainer<S: ParallelSlot> {
    pub slot: Arc<RwLock<S>>,
}

pub struct ParallelSlotRunner<S: ParallelSlot> {
    pub container: ParallelSlotContainer<S>,
    pub worker_tx: WorkerTx<S>,
    pub worker_rxs: Vec<WorkerRx<S>>,
}

impl<S> ParallelSlotRunner<S>
where
    S: ParallelSlot,
{
    pub fn new(slot: S, worker_tx: WorkerTx<S>, worker_rxs: Vec<WorkerRx<S>>) -> Self {
        Self {
            container: ParallelSlotContainer {
                slot: Arc::new(RwLock::new(slot)),
            },
            worker_tx,
            worker_rxs,
        }
    }
}

impl<S: ParallelSlot> SlotRunner<S> for ParallelSlotRunner<S> {
    fn run(self, mut rx: SlotReceiver, tx: SlotSender, exit_tx: SlotSender) -> SlotRunnerHandle {
        let slot = self.container.slot;
        let worker_tx = self.worker_tx;
        let worker_rxs = self.worker_rxs;

        let handle = tokio::spawn(async move {
            let mut workers: FuturesUnordered<tokio::task::JoinHandle<Result<(), TunRackError>>> =
                FuturesUnordered::new();

            for worker_rx in worker_rxs {
                let slot = slot.clone();
                let tx = tx.clone();
                let exit_tx = exit_tx.clone();

                workers.push(tokio::spawn(async move {
                    loop {
                        let data = worker_rx.recv_async().await?;

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
                                worker_tx.send_async(data).await.map_err(|_| TunRackError::SlotAsyncChannelSendError)?;
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
