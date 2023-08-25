use argon::ArgonTunError;
use argon_rack::{TunRackError, TunRackLayoutError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArgonDriverError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    TunRackError(#[from] TunRackError),

    #[error(transparent)]
    TunRackLayoutError(#[from] TunRackLayoutError),

    #[error(transparent)]
    ArgonTunError(#[from] ArgonTunError),

    #[error("RackChannelClosed")]
    RackChannelClosed,
}
