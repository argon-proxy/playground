mod r#async;
pub use r#async::{AbiAsyncSlotProcessor, AsyncSlot};

mod config;
pub use config::SlotConfig;

mod error;
pub use error::ArgonSlotError;

pub mod processor;

mod sync;
pub use sync::{AbiSyncSlotProcessor, SyncSlot};

mod traits;
pub use traits::Slot;

pub mod worker;
