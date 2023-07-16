use super::TunRackSequentialSlot;

pub trait TunRackSlotBuilder<ST>: Default
where
    ST: TunRackSequentialSlot,
{
    fn build(self) -> ST;
}
