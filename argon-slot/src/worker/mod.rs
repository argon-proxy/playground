pub mod r#async;

mod error;
pub use error::SlotWorkerError;

mod handle;
pub use handle::SlotWorkerHandle;

pub mod sync;
