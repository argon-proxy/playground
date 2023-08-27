use argon_plugin_registry::ArgonPluginRegistryError;
use argon_slot::{worker::SlotWorkerError, ArgonSlotError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TunRackError {
    #[error(transparent)]
    ArgonSlotError(#[from] ArgonSlotError),

    #[error(transparent)]
    ArgonPluginRegistryError(#[from] ArgonPluginRegistryError),

    #[error("SlotCrash")]
    SlotCrash,

    #[error("SlotChannelClosed")]
    SlotChannelClosed,

    #[error("SlotRunnerError({0})")]
    SlotWorkerError(#[from] SlotWorkerError),

    #[error("TokioJoinError({0})")]
    TokioJoinError(#[from] tokio::task::JoinError),

    #[error("InternalError")]
    InternalError,
}
