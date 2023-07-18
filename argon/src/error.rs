use thiserror::Error;

use crate::rack::SlotSendError;

#[derive(Error, Debug)]
pub enum TunRackError {
    #[error("TunError({0})")]
    TunError(#[from] tun::Error),

    #[error("IoError({0})")]
    IoError(std::io::Error),

    #[error("SlotSendError({0})")]
    SlotSendError(#[from] SlotSendError),

    #[error("SlotCrash")]
    SlotCrash,

    #[error("SlotChannelClosed")]
    SlotChannelClosed,

    #[error("TokioJoinError({0})")]
    TokioJoinError(#[from] tokio::task::JoinError),

    #[error("SlotAsyncChannelSendError")]
    SlotAsyncChannelSendError,

    #[error("SlotAsyncChannelRecvError({0})")]
    SlotAsyncChannelRecvError(#[from] flume::RecvError),
}
