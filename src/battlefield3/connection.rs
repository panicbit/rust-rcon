use tokio::net::{TcpStream, ToSocketAddrs};
use std::io;
use async_trait::async_trait;

use crate::connection::Connection;
use crate::error::{Result, Error};
use crate::packet::{Packet, PacketType};
use super::packet::Btf3Packet;

/// If the size in bytes of the packet is above this the server will reject 
/// the connection
const BTF3_MAX_PAYLOAD_SIZE: usize = 16384;

/// I dunno if it should be 1 or 0 but `it works`
const INITIAL_PACKET_ID: usize = 1;

/// Representation of the client connection to a Remote Administration Interface
pub struct Btf3Connection {
    stream: TcpStream,
    next_packet_id: i32,
}

impl Btf3Connection {
    /// Auth into the server given the password, if the password is `""` it will
    /// skip the login
    async fn auth(&mut self, password: &str) -> Result<bool> {
        if password == "" {
            println!("Skipped login");
            Ok(true)
        } else if self.cmd(
                &format!("login.PlainText {}", password)
            ).await?.contains("OK") 
        {
            println!("Logged in");
            Ok(true)
        } else {
            Err(Error::Auth)
        }
    }
}

#[async_trait]
impl Connection for Btf3Connection {
    type Packet = Btf3Packet;
    /// Connects to a rcon server, if the password if `""` if will skip 
    /// the login
    async fn connect<T: ToSocketAddrs + Send>(address: T, password: &str) -> Result<Self> {
        let stream = TcpStream::connect(address).await?;
        let mut conn = Btf3Connection {
            stream,
            next_packet_id: INITIAL_PACKET_ID as i32,
        };

        conn.auth(password).await?;
        Ok(conn)
    }

    /// Send a certain command, in the implementation the `command` should be
    /// parsed to a packet ready bytes buffer and sended via `send_packet`
    async fn cmd(&mut self, command: &str) -> Result<String> {
        if command.len() > BTF3_MAX_PAYLOAD_SIZE {
            return Err(Error::CommandTooLong);
        }
        self.next_packet_id = self.send_packet(
            Btf3Packet::new(
                self.next_packet_id,
                PacketType::Request(0),
                command.to_owned()
            )
        ).await?;

        let response = self.receive_response().await?;
        
        Ok(response)
    }

    /// Receives a response from the rcon server
    async fn receive_response(&mut self) -> io::Result<String> {
        let received_packet = self.receive_packet().await?;
        Ok(received_packet.get_body().into())
    }

    /// Low level function that send a Packet, returns the `id` of the sended 
    /// packet to be incremented
    async fn send_packet(&mut self, packet: Self::Packet) -> io::Result<i32> {
        let id = match self.next_packet_id + 1 {
            n if n & 0x3fff != 0 => INITIAL_PACKET_ID as i32,
            n => n,
        };

        packet.serialize(&mut self.stream).await?;

        Ok(id)
    }

    /// Low level function that receives a Packet
    async fn receive_packet(&mut self) -> io::Result<Self::Packet> {
        Btf3Packet::deserialize(&mut self.stream).await
    }
}

