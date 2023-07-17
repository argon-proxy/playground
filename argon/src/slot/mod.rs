mod builder;
pub use builder::SlotBuilder;

mod config;
pub use config::{SequentialSlotRunnerConfig, SlotRunnerConfig};

mod event;
pub use event::SlotPacket;

mod handle;
pub use handle::SlotHandle;

mod process;
pub use process::SlotProcessResult;

mod sequential;
pub use sequential::SequentialSlot;

mod parallel;
pub use parallel::{ParallelSlot, ParallelSlotContainer};
