use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArgonTunError {
    #[error(transparent)]
    TunError(#[from] tun::Error),
}
