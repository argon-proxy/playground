use tokio::task::JoinHandle;

use super::SlotWorkerError;

pub struct SlotWorkerHandle {
    pub handle: JoinHandle<Result<(), SlotWorkerError>>,
}
