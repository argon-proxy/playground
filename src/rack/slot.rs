use super::slot_handle::TunRackSlotHandle;

pub trait TunRackSlot {
    fn run(self) -> TunRackSlotHandle;
}
