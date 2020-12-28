use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use async_trait::async_trait;
use std::io;

use crate::packet::{Packet, PacketType};

/// ### The Battlefield packets are designed in this way:
///      i32 (le)        i32 (le)       i32 (le)   [u8; packet_size - sizeof(i32)*3]
/// -------------------------------------------------------------------------
/// |  sequence  |  packet_size  |  word_count  |           body            |
/// ------------------------------------------------------------------------
/// 
/// Knowing that there are also two special fields:
/// 
/// #### SEQUENCE:
/// 0                 29               30              31    (i32 bits)
/// ----------------------------------------------------
/// |       id        |      type      |     origin    |
/// ----------------------------------------------------
///     id: number that grows apart from the client and the server, the spec
///         doesn't say initial number so we will be using 1 in this 
///         implementation 
///     origin: if this bit = 0, its originated from the server, if its = 1
///             its originated from the client (us) 
///     type: 0 = Request, 1 = Response, ussually we are the request and the 
///           server just response
/// 
/// #### BODY: 
/// The body is composed by a determined number of words and each word 
/// have the following design
///        i32 (le)            [u8; word_size]         u8
/// ------------------------------------------------------------
/// |      word_size    |         word           |    null     |
/// -----------------------------------------------------------
///     NOTE: note that word can only contain ASCII characters and the null
///           terminator is not counted in the word_size
#[derive(Debug)]
pub struct Btf3Packet {
    /// This 3 fields are copied without needed modifications
    sequence: i32,
    packet_size: i32,
    word_count: i32,

    /// This String must be converted to the required specification at the send
    /// time, here the words are separated by spaces
    body: String,

    /// The packet type
    ptype: PacketType,
}

impl Btf3Packet {
    /// Instantiates a new Btf3Packet
    pub fn new(id: i32, ptype: PacketType, body: String) -> Self {
        // NOTE: This still does not discern the origin of the packet it 
        // suposses that the reponses always come from the server and the 
        // requests from the client
        let sequence = match ptype {
            PacketType::Request(_) => id,
            PacketType::Response(_) => 3 << 29 | id,
            _ => todo!(),
        };

        let body_size: usize = body
            .split_ascii_whitespace()
            .map(|s| s.len())
            .fold(0, |acc, x| acc + x + 5);

        let packet_size = 4 * 3 + body_size as i32;
        Btf3Packet {
            sequence,
            packet_size,
            word_count: body.split_ascii_whitespace().count() as i32,
            body,
            ptype,
        }
    }
}

#[async_trait]
impl Packet for Btf3Packet {
    /// Checks if the packet is an error, probably just useful for Responses
    fn is_error(&self) -> bool {
        !self.get_body().contains("OK")
    }

    /// Serializes de packets, aka convert and send
    async fn serialize<T: Unpin + AsyncWrite + Send>(&self, w: &mut T) -> io::Result<()> {
        let mut buf = Vec::with_capacity(self.packet_size as usize);
        buf.write_i32_le(self.sequence).await?;
        buf.write_i32_le(self.packet_size).await?;
        buf.write_i32_le(self.word_count).await?;
        for word_str in self.body.split_ascii_whitespace() {
            buf.write_i32_le(word_str.len() as i32).await?;
            buf.write_all(word_str.as_bytes()).await?;
            buf.write_all(b"\x00").await?;
        }

        w.write_all(&buf).await?;

        Ok(())
    }

    /// Deserializes de packets, aka receive and convert
    async fn deserialize<T: Unpin + AsyncRead + Send>(r: &mut T) -> io::Result<Self> {
        let sequence = r.read_i32_le().await?;
        let (_id, ptype) = (sequence & 0xC000, match sequence >> 29 {
            0b10 | 0b11 => PacketType::Response(1),
            0b00 | 0b01 => PacketType::Request(0),
            // TODO: more sofisticated error report
            _ => return Err(io::Error::from(io::ErrorKind::Other))
        });

        let packet_size = r.read_i32_le().await?;
        let word_count = r.read_i32_le().await?;

        // Overallocation by 4*3
        let mut body = Vec::with_capacity(packet_size as usize);

        for _ in 0..word_count {
            let word_size = r.read_i32_le().await?;
            let mut word_buffer = Vec::with_capacity(word_size as usize);
            r.take(word_size as u64)
                .read_to_end(&mut word_buffer)
                .await?;
            body.extend_from_slice(&word_buffer);
            body.push(' ' as u8);
            r.read_u8().await?;
        }
        body.pop();
        let body = String::from_utf8(body)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        
        Ok(
            Btf3Packet {
                sequence,
                packet_size,
                word_count,
                body,
                ptype,
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
    fn get_type(&self, _is_response: bool) -> PacketType {
        self.ptype
    }
    /// Returns the id of the packet and also increments it
    #[inline]
    fn get_id(&self) -> i32 {
        self.sequence & 0xC000
    }
}

unsafe impl Send for Btf3Packet {}