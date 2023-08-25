#[derive(Clone)]
pub struct SlotConfig {
    pub workers: usize,
    pub name: String,
}

impl Default for SlotConfig {
    fn default() -> Self {
        Self {
            workers: 1,
            name: "slot".to_owned(),
        }
    }
}

impl SlotConfig {
    pub fn set_workers(mut self, workers: usize) -> Self {
        debug_assert!(workers > 0);

        self.workers = workers;

        self
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;

        self
    }
}
