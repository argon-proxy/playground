use crate::{
    rack::{SlotReceiver, SlotSender},
    slot::SlotHandle,
};

mod sequential;
pub use sequential::*;

pub trait SlotRunner<S> {
    fn new(slot: S) -> Self;

    fn run(self, rx: SlotReceiver, tx: SlotSender, exit_tx: SlotSender) -> SlotHandle;
}
