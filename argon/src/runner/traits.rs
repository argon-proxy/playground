use super::SlotRunnerHandle;
use crate::rack::{SlotReceiver, SlotSender};

pub trait SlotRunner<S> {
    fn run(self, rx: SlotReceiver, tx: SlotSender, exit_tx: SlotSender) -> SlotRunnerHandle;
}