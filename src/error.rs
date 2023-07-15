use thiserror::Error;

use crate::rack::TunRackSendError;

#[derive(Error, Debug)]
pub enum TunRackError {
    #[error("TunError({0})")]
    TunError(#[from] tun::Error),

    #[error("TunIoError({0})")]
    TunIoError(std::io::Error),

    #[error("TunRackSendError({0})")]
    TunRackSendError(#[from] TunRackSendError),

    #[error("TunRackSlotCrash")]
    TunRackSlotCrash,

    #[error("TunRackChannelClosed")]
    TunRackChannelClosed,

    #[error("TokioJoinError({0})")]
    TokioJoinError(#[from] tokio::task::JoinError),
}
