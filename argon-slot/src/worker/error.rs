use argon::rotary::RotaryCanonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SlotWorkerError {
    #[error("RotaryCannonError({0})")]
    RotaryCannonError(#[from] RotaryCanonError),
}
