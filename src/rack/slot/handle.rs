use tokio::task::JoinHandle;

use crate::error::TunRackError;

pub struct TunRackSlotHandle {
    pub handle: JoinHandle<Result<(), TunRackError>>,
}
