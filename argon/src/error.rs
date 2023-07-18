use thiserror::Error;

use crate::{rack::SlotSendError, runner::SlotRunnerError};

#[derive(Error, Debug)]
pub enum TunRackError {
    #[error("TunError({0})")]
    TunError(#[from] tun::Error),

    #[error("IoError({0})")]
    IoError(std::io::Error),

    #[error("SlotSendError({0})")]
    SlotSendError(#[from] SlotSendError),

    #[error("SlotRunnerError({0})")]
    SlotRunnerError(#[from] SlotRunnerError),

    #[error("TokioJoinError({0})")]
    TokioJoinError(#[from] tokio::task::JoinError),
}
