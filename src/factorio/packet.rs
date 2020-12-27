use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use async_trait::async_trait;
use std::io;

use crate::packet::{Packet, PacketType};

/// #### Factorio packets are designed in this way:
/// 
/// TODO: ...
/// 
#[derive(Debug)]
pub struct FactorioPacket {
    id: i32,
    packet_size: i32,

    /// This String must be converted to the required specification at the send
    /// time, here the words are separated by spaces
    body: String,

    /// The packet type
    ptype_n: i32,
}

impl FactorioPacket {
    /// Instantiates a new FactorioPacket
    pub fn new(id: i32, ptype: PacketType, body: String) -> Self {
        // NOTE: This still does not discern the origin of the packet it 
        // suposses that the reponses always come from the server and the 
        // requests from the client

        let packet_size = 10 + body.len() as i32;
        let ptype_n = match ptype {
            PacketType::Request(n) => n,
            PacketType::Response(n) => n,
            PacketType::Custom(_) => todo!(),
        };

        FactorioPacket {
            id,
            packet_size,
            body,
            ptype_n,
        }
    }
}

#[async_trait]
impl Packet for FactorioPacket {
    /// Checks if the packet is an error, probably just useful for Responses
    fn is_error(&self) -> bool {
        self.id < 0
    }

    /// Serializes de packets, aka convert and send
    async fn serialize<T: Unpin + AsyncWrite + Send>(&self, w: &mut T) -> io::Result<()> {
        let mut buf = Vec::with_capacity(self.packet_size as usize);

        buf.write_i32_le(self.packet_size).await?;
        buf.write_i32_le(self.id).await?;
        buf.write_i32_le(self.ptype_n).await?;
        buf.write_all(self.body.as_bytes()).await?;
        buf.write_all(b"\x00\x00").await?;

        w.write_all(&buf).await?;

        Ok(())
    }

    /// Deserializes de packets, aka receive and convert
    async fn deserialize<T: Unpin + AsyncRead + Send>(r: &mut T) -> io::Result<Self> {
        let packet_size = r.read_i32_le().await?;
        let id = r.read_i32_le().await?;
        let ptype_n = r.read_i32_le().await?;
        let body_length = packet_size - 10;

        let mut body = Vec::with_capacity(body_length as usize);

        r.take(body_length as u64)
            .read_to_end(&mut body)
            .await?;

        let body = String::from_utf8(body)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;

        // Terminated nulls (\x00\x00)
        r.read_u16().await?;
        
        Ok(
            FactorioPacket {
                id,
                packet_size,
                body,
                ptype_n,
            }
        )
    }

    /// Gets the body of the packet
    #[inline]
    fn get_body(&self) -> &str {
        &self.body
    }

    /// Gets the packet type
    #[inline]
    fn get_type(&self) -> PacketType {
        match self.ptype_n {
            0 => PacketType::Response(0),
            2 => PacketType::Request(2),
            3 => PacketType::Request(3),
            _ => todo!(),
        }
    }

    /// Returns the id of the packet and also increments it
    #[inline]
    fn get_id(&self) -> i32 {
        self.id
    }
}