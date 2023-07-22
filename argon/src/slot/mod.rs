mod r#async;
pub use r#async::{AsyncSlot, AsyncSlotProcessor};

mod event;
pub use event::SlotPacket;

mod process;
pub use process::SlotProcessResult;

mod sync;
pub use sync::{SyncSlot, SyncSlotProcessor};

mod traits;
pub use traits::Slot;

pub mod worker;
