use super::TunRackSlot;

pub trait TunRackSlotBuilder<ST>: Default
where
    ST: TunRackSlot,
{
    fn build(self) -> ST;
}
