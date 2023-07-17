pub type SlotSender = tokio::sync::mpsc::Sender<tun::TunPacket>;
pub type SlotSendError = tokio::sync::mpsc::error::SendError<tun::TunPacket>;
pub type SlotReceiver = tokio::sync::mpsc::Receiver<tun::TunPacket>;
