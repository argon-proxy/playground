use super::{slot::TunRackSlot, TunRackSlotReceiver, TunRackSlotSender};

pub trait TunRackSlotBuilder<ST>
where
    ST: TunRackSlot,
{
    fn build(self, rx: TunRackSlotReceiver, tx: TunRackSlotSender, exit_tx: TunRackSlotSender) -> ST;
}
