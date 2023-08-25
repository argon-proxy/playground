use futures::{Sink, SinkExt, Stream, StreamExt};
use tokio_util::codec::Framed;

use crate::{config::ArgonTunConfig, ArgonTunError};

pub struct ArgonTun {
    frame: Framed<tun::AsyncDevice, tun::TunPacketCodec>,
    tun_config: tun::Configuration,
    argon_config: ArgonTunConfig,
}

impl ArgonTun {
    pub fn new(config: ArgonTunConfig) -> Result<Self, ArgonTunError> {
        let mut tun_config = tun::Configuration::default();

        tun_config.mtu(config.mtu.into());

        tun_config
            .address((10, 0, 0, 1))
            .netmask((255, 255, 255, 0))
            .up();

        #[cfg(target_os = "linux")]
        tun_config.platform(|config| {
            config.packet_information(true);
        });

        let device = tun::create_as_async(&tun_config)?;

        let frame = device.into_framed();

        Ok(Self {
            frame,
            tun_config,
            argon_config: config,
        })
    }
}

impl Stream for ArgonTun {
    type Item = Result<tun::TunPacket, std::io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.frame.poll_next_unpin(cx)
    }
}

impl Sink<tun::TunPacket> for ArgonTun {
    type Error = std::io::Error;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.frame.poll_ready_unpin(cx)
    }

    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: tun::TunPacket,
    ) -> Result<(), Self::Error> {
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
    use crate::{config::ArgonTunConfig, ArgonTun};

    #[ignore]
    #[tokio::test]
    async fn create_tun_test() {
        let argon_tun = ArgonTun::new(ArgonTunConfig::default());

        assert!(argon_tun.is_ok());
    }
}
