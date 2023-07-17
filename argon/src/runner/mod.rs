mod config;
pub use config::SlotRunnerConfig;

mod handle;
pub use handle::SlotRunnerHandle;

mod parallel;
pub use parallel::*;

mod sequential;
pub use sequential::*;

mod traits;
pub use traits::*;
