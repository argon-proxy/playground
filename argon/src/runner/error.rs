use thiserror::Error;
use tokio::sync::TryLockError;

use crate::rack::SlotSendError;

#[derive(Error, Debug)]
pub enum SlotRunnerError {
    #[error("TryLockError({0})")]
    TryLockError(#[from] TryLockError),

    #[error("SlotCrash")]
    SlotCrash,

    #[error("SlotChannelClosed")]
    SlotChannelClosed,

    #[error("SlotSendError")]
    SlotSendError(#[from] SlotSendError),
}
