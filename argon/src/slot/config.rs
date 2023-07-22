#[derive(Clone, Copy)]
pub struct SlotConfig {
    pub workers: usize,
}

impl Default for SlotConfig {
    fn default() -> Self {
        Self { workers: 1 }
    }
}
