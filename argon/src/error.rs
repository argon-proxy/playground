use thiserror::Error;

use crate::{rotary::RotaryCanonError, slot::worker::SlotWorkerError};

#[derive(Error, Debug)]
pub enum TunRackError {
    #[error("TunError({0})")]
    TunError(#[from] tun::Error),

    #[error("IoError({0})")]
    IoError(std::io::Error),

    #[error("RotaryCanonError({0})")]
    RotaryCanonError(#[from] RotaryCanonError),

    #[error("SlotRunnerError({0})")]
    SlotWorkerError(#[from] SlotWorkerError),

    #[error("TokioJoinError({0})")]
    TokioJoinError(#[from] tokio::task::JoinError),
}
