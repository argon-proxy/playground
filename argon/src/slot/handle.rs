use tokio::task::JoinHandle;

use crate::error::TunRackError;

pub struct SlotHandle {
    pub handle: JoinHandle<Result<(), TunRackError>>,
}
