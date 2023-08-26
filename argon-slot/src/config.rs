use argon::config::ArgonSlotConfig;

#[derive(Clone)]
pub struct SlotConfig {
    pub name: String,
    pub workers: usize,
}

impl Default for SlotConfig {
    fn default() -> Self {
        Self {
            name: "slot".to_owned(),
            workers: 1,
        }
    }
}

impl SlotConfig {
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;

        self
    }

    pub fn with_workers(mut self, workers: usize) -> Self {
        debug_assert!(workers > 0);

        self.workers = workers;

        self
    }
}

impl From<&ArgonSlotConfig> for SlotConfig {
    fn from(value: &ArgonSlotConfig) -> Self {
        Self {
            name: value.plugin.to_owned(),
            workers: value.workers,
        }
    }
}
