use tokio::task::JoinHandle;

use crate::error::TunRackError;

pub type TunRackSlotHandleResult = Result<(), TunRackError>;

pub struct TunRackSlotHandle {
    pub handle: JoinHandle<TunRackSlotHandleResult>,
}

impl TunRackSlotHandle {
    pub fn new(handle: JoinHandle<TunRackSlotHandleResult>) -> Self {
        Self { handle }
    }
}
