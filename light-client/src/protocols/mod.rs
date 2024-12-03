mod http;
mod tcp;
mod tty;
mod udp;

use async_trait::async_trait;
use bytes::Bytes;
use lightfx::{Color, Frame};
use log::info;

use crate::{backoff_decorator::WithBackoff, config::ByteOrder, LightClient, LightClientError};

pub use http::HttpLightClient;
pub use tcp::TcpLightClient;
pub use tty::TtyLightClient;
pub use udp::UdpLightClient;

#[async_trait]
pub trait ProtocolLightClient: Sized + Send + Sync {
    async fn display_frame(&self, frame: Bytes) -> Result<(), LightClientError>;

    fn with_byte_order(self, byte_order: ByteOrder) -> ByteOrderAdapter<Self> {
        info!("Using byte order {byte_order:?}");
        ByteOrderAdapter {
            inner: self,
            byte_order,
        }
    }
}

pub struct ByteOrderAdapter<T: ProtocolLightClient> {
    inner: T,
    byte_order: ByteOrder,
}

#[async_trait]
impl<T> LightClient for ByteOrderAdapter<T>
where
    T: ProtocolLightClient,
{
    async fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        let to_bytes = match self.byte_order {
            ByteOrder::Rgb => |c: Color| -> [u8; 3] { [c.r, c.g, c.b] },
            ByteOrder::Grb => |c: Color| -> [u8; 3] { [c.g, c.r, c.b] },
        };
        let pixels = frame
            .pixels_iter()
            .copied()
            .flat_map(to_bytes)
            .map(gamma_correction)
            .collect();

        self.inner.display_frame(pixels).await
    }
}

impl<T> WithBackoff for ByteOrderAdapter<T> where T: ProtocolLightClient {}

fn gamma_correction(component: u8) -> u8 {
    (((component as f64) / 255.0).powi(2) * 255.0) as u8
}
