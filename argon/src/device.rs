use futures::{Sink, SinkExt, Stream, StreamExt};
use tokio_util::codec::Framed;

pub struct Tun {
    frame: Framed<tun::AsyncDevice, tun::TunPacketCodec>,
    config: tun::Configuration,
}

impl Tun {
    pub fn new(mtu: Option<u16>) -> Result<Self, tun::Error> {
        let mut config = tun::Configuration::default();

        if let Some(mtu) = mtu {
            config.mtu(mtu.into());
        }

        config.address((10, 0, 0, 1)).netmask((255, 255, 255, 0)).up();

        #[cfg(target_os = "linux")]
        config.platform(|config| {
            config.packet_information(true);
        });

        let device = tun::create_as_async(&config)?;

        let frame = device.into_framed();

        Ok(Self { frame, config })
    }
}

impl Stream for Tun {
    type Item = Result<tun::TunPacket, std::io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.frame.poll_next_unpin(cx)
    }
}

impl Sink<tun::TunPacket> for Tun {
    type Error = std::io::Error;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.frame.poll_ready_unpin(cx)
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: tun::TunPacket) -> Result<(), Self::Error> {
        self.frame.start_send_unpin(item)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.frame.poll_flush_unpin(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.frame.poll_close_unpin(cx)
    }
}

#[cfg(test)]
mod tests {
    use crate::Tun;
    #[ignore]
    #[tokio::test]
    async fn create_tun_test() {
        let argon_tun = Tun::new(None);

        assert!(argon_tun.is_ok());
    }
}
