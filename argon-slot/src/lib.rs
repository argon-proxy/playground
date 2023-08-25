#![feature(vec_into_raw_parts)]

mod r#async;
pub use r#async::AsyncSlot;

mod config;
pub use config::SlotConfig;

mod error;
pub use error::ArgonSlotError;

pub mod processor;

mod sync;
pub use sync::SyncSlot;

mod traits;
pub use traits::Slot;

pub mod worker;
