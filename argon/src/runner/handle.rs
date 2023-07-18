use tokio::task::JoinHandle;

use super::SlotRunnerError;

pub struct SlotRunnerHandle {
    pub handle: JoinHandle<Result<(), SlotRunnerError>>,
}
