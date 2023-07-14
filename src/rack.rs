pub type TunRackSlotSender = tokio::sync::mpsc::Sender<tun::TunPacket>;
pub type TunRackSlotReceiver = tokio::sync::mpsc::Receiver<tun::TunPacket>;

pub type TunRackSendError = tokio::sync::mpsc::error::SendError<tun::TunPacket>;

pub fn build_tunrack_channel(channel_size: usize) -> (TunRackSlotSender, TunRackSlotReceiver) {
    tokio::sync::mpsc::channel(channel_size)
}

pub trait TunRackSlot {}

pub trait TunRackSlotBuilder {
    fn build(self: Box<Self>, rx: TunRackSlotReceiver, tx: TunRackSlotSender) -> Box<dyn TunRackSlot>;
}

pub struct TunRack {
    racks: Vec<Box<dyn TunRackSlot>>,

    channel_size: usize,
    tx: TunRackSlotSender,
    rx: TunRackSlotReceiver,
}

impl TunRack {
    pub fn new(channel_size: usize) -> Self {
        let (tx, rx) = build_tunrack_channel(channel_size);

        Self {
            channel_size,
            racks: Vec::new(),
            tx,
            rx,
        }
    }

    pub fn add_slot(&mut self, slot: Box<dyn TunRackSlotBuilder>) {
        let (slot_tx, mut slot_rx) = build_tunrack_channel(self.channel_size);

        std::mem::swap(&mut self.rx, &mut slot_rx);

        let slot = slot.build(slot_rx, slot_tx);

        self.racks.push(slot);
    }

    pub async fn process(&mut self, packet: tun::TunPacket) -> Result<(), TunRackSendError> {
        self.tx.send(packet).await
    }
}
