mod builder;
pub use builder::SlotBuilder;

mod event;
pub use event::SlotPacket;

mod process;
pub use process::SlotProcessResult;

mod sequential;
pub use sequential::SequentialSlot;

mod parallel;
pub use parallel::ParallelSlot;
