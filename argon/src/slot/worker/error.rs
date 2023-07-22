use thiserror::Error;

use crate::rotary::RotaryCanonError;

#[derive(Error, Debug)]
pub enum SlotWorkerError {
    #[error("RotaryCannonError({0})")]
    RotaryCannonError(#[from] RotaryCanonError),

    #[error("SlotCrash")]
    SlotCrash,

    #[error("SlotChannelClosed")]
    SlotChannelClosed,
}
