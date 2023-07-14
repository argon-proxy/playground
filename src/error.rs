use thiserror::Error;

use crate::rack::TunRackSendError;

#[derive(Error, Debug)]
pub enum TunRackError {
    #[error("TunError({0})")]
    TunError(#[from] tun::Error),

    #[error("TunRackSendError({0})")]
    TunRackSendError(#[from] TunRackSendError),
}
