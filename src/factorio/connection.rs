use tokio::net::{TcpStream, ToSocketAddrs};
use std::io;
use async_trait::async_trait;

use crate::connection::Connection;
use crate::error::{Result, Error};
use crate::packet::{Packet, PacketType};
use super::packet::FactorioPacket;

/// If the size in bytes of the packet is above this the server will reject 
/// the connection
/// TODO:
// const FACTORIO_MAX_PAYLOAD_SIZE: usize = ...;

/// I dunno if it should be 1 or 0 but `it works`
const INITIAL_PACKET_ID: usize = 1;

/// Representation of the client connection to a Remote Administration Interface
pub struct FactorioConnection {
    stream: TcpStream,
    next_packet_id: i32,
}

impl FactorioConnection {
    /// Auth into the server given the password, login is mandatory
    async fn auth(&mut self, password: &str) -> Result<()> {
        self.send_packet(
            FactorioPacket::new(
                self.next_packet_id,
                PacketType::Request(3),
                password.to_string(),
            )
        ).await?;

        let received_packet = loop {
            let received_packet = self.receive_packet().await?;
            if let PacketType::Response(2) = received_packet.get_type() {
                break received_packet;
            }
        };

        if received_packet.is_error() {
            Err(Error::Auth)
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl Connection for FactorioConnection {
    type Packet = FactorioPacket;
    /// Connects to a rcon server, login is mandatory
    async fn connect<T: ToSocketAddrs + Send>(address: T, password: &str) -> Result<Self> {
        let stream = TcpStream::connect(address).await?;
        let mut conn = FactorioConnection {
            stream,
            next_packet_id: INITIAL_PACKET_ID as i32,
        };

        conn.auth(password).await?;
        Ok(conn)
    }

    /// Send a certain command, in the implementation the `command` should be
    /// parsed to a packet ready bytes buffer and sended via `send_packet`
    async fn cmd(&mut self, command: &str) -> Result<String> {
        self.next_packet_id = self.send_packet(
            FactorioPacket::new(
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
        // I dont know if factorio uses some bits for something in particular
        let id = match self.next_packet_id + 1 {
            n if n & 0x3fff != 0 => INITIAL_PACKET_ID as i32,
            n => n,
        };

        packet.serialize(&mut self.stream).await?;

        Ok(id)
    }

    /// Low level function that receives a Packet
    async fn receive_packet(&mut self) -> io::Result<Self::Packet> {
        FactorioPacket::deserialize(&mut self.stream).await
    }
}

