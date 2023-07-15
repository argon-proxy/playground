use super::{TunRackSlot, TunRackSlotHandle};

pub struct RackSlotRunner<S: TunRackSlot> {
    slot: S,
}

impl<S: TunRackSlot> RackSlotRunner<S> {
    fn run(self) -> TunRackSlotHandle {
        TunRackSlotHandle { handle: todo!() }
    }
}
