use futures::Stream;
use nonempty::NonEmpty;
use thiserror::Error;

type Packet = tun::TunPacket;

type IntraSlotSender = tokio::sync::mpsc::Sender<Packet>;
type IntraSlotSyncSendError = tokio::sync::mpsc::error::TrySendError<Packet>;
type IntraSlotReceiver = tokio::sync::mpsc::Receiver<Packet>;

pub fn build_single_channel(
    channel_size: usize,
) -> (IntraSlotSender, IntraSlotReceiver) {
    tokio::sync::mpsc::channel(channel_size)
}

#[derive(Error, Debug)]
pub enum RotaryCanonError {
    #[error("ChannelClosed")]
    ChannelClosed,
}

#[derive(Clone)]
pub struct RotaryCanon {
    canons: NonEmpty<IntraSlotSender>,
    index: usize,
}

impl RotaryCanon {
    pub fn new(canons: NonEmpty<IntraSlotSender>) -> Self {
        Self { canons, index: 0 }
    }

    pub fn fire(
        &mut self,
        mut packet: Packet,
    ) -> Result<bool, RotaryCanonError> {
        let index_start = self.index;

        while let Some(canon) = self.canons.get(self.index) {
            let send_error = match canon.try_send(packet) {
                Ok(()) => return Ok(true),
                Err(e) => e,
            };

            packet = match send_error {
                IntraSlotSyncSendError::Full(packet) => packet,
                IntraSlotSyncSendError::Closed(_) => {
                    return Err(RotaryCanonError::ChannelClosed)
                },
            };

            self.index = (self.index + 1) % self.canons.len();

            if self.index == index_start {
                return Ok(false);
            }
        }

        unreachable!()
    }
}

pub struct RotaryTarget {
    targets: Vec<IntraSlotReceiver>,
    index: usize,
}

impl RotaryTarget {
    pub fn new(targets: NonEmpty<IntraSlotReceiver>) -> Self {
        Self {
            targets: targets.into(),
            index: 0,
        }
    }
}

impl Stream for RotaryTarget {
    type Item = Packet;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let index = self.index;

        let (front, back) = self.targets.split_at_mut(index);

        for (offset, target) in back.iter_mut().chain(front).enumerate() {
            let packet_option = match target.poll_recv(cx) {
                std::task::Poll::Ready(ready) => ready,
                std::task::Poll::Pending => continue,
            };

            self.index = (self.index + offset + 1) % self.targets.len();

            return std::task::Poll::Ready(packet_option);
        }

        std::task::Poll::Pending
    }
}
