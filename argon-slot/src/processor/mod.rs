pub mod r#async;

pub mod ffi;

mod packet;
pub use packet::SlotPacket;

mod result;
pub use result::SlotProcessorResult;

pub mod sync;
