pub trait SlotBuilder<ST>: Default {
    fn build(self) -> ST;
}
