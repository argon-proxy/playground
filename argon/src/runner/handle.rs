use tokio::task::JoinHandle;

use crate::error::TunRackError;

pub struct SlotRunnerHandle {
    pub handle: JoinHandle<Result<(), TunRackError>>,
}
