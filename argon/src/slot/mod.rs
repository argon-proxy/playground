mod builder;
pub use builder::SlotBuilder;

mod event;
pub use event::SlotPacket;

mod handle;
pub use handle::SlotHandle;

mod process;
pub use process::SlotProcessResult;

mod sequential;
pub use sequential::SequentialSlot;

mod parallel;
pub use parallel::ParallelSlot;
