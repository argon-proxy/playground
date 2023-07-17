use super::SequentialSlot;

pub trait SlotBuilder<ST>: Default
where
    ST: SequentialSlot,
{
    fn build(self) -> ST;
}
