use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArgonSlotError {
    #[error("TokioJoinError({0})")]
    TokioJoinError(#[from] tokio::task::JoinError),

    #[error("InternalError")]
    InternalError,
}
