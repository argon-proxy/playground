use argon::ArgonTunError;
use argon_plugin_registry::ArgonPluginRegistryError;
use argon_rack::{TunRackError, TunRackLayoutError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArgonDriverError {
    #[error(transparent)]
    ArgonPluginRegistryError(#[from] ArgonPluginRegistryError),

    #[error(transparent)]
    ArgonTunError(#[from] ArgonTunError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("RackChannelClosed")]
    RackChannelClosed,

    #[error(transparent)]
    TunRackError(#[from] TunRackError),

    #[error(transparent)]
    TunRackLayoutError(#[from] TunRackLayoutError),
}
