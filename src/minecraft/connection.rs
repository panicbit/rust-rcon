use tokio::net::{TcpStream, ToSocketAddrs};
use std::io;
use async_trait::async_trait;
use std::time::Duration;

use crate::connection::Connection;
use crate::error::{Result, Error};
use crate::packet::{Packet, PacketType};
use super::packet::MinecraftPacket;

/// If the size in bytes of the packet is above this the server will reject 
/// the connection
const MC_MAX_PAYLOAD_SIZE: usize = 1413;

/// I dunno if it should be 1 or 0 but `it works`
const INITIAL_PACKET_ID: usize = 1;

/// It dunno why :P
const DELAY_TIME_MILLIS: u64 = 3;

/// Representation of the client connection to a Remote Administration Interface
pub struct MinecraftConnection {
    stream: TcpStream,
    next_packet_id: i32,
}

impl MinecraftConnection {
    /// Auth into the server given the password, login is mandatory
    async fn auth(&mut self, password: &str) -> Result<()> {
        self.send_packet(
            MinecraftPacket::new(
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
impl Connection for MinecraftConnection {
    type Packet = MinecraftPacket;
    /// Connects to a rcon server, login is mandatory
    async fn connect<T: ToSocketAddrs + Send>(address: T, password: &str) -> Result<Self> {
        let stream = TcpStream::connect(address).await?;
        let mut conn = MinecraftConnection {
            stream,
            next_packet_id: INITIAL_PACKET_ID as i32,
        };

        conn.auth(password).await?;
        Ok(conn)
    }

    /// Send a certain command, in the implementation the `command` should be
    /// parsed to a packet ready bytes buffer and sended via `send_packet`
    async fn cmd(&mut self, command: &str) -> Result<String> {
        if command.len() >= MC_MAX_PAYLOAD_SIZE {
            return Err(Error::CommandTooLong);
        }

        self.next_packet_id = self.send_packet(
            MinecraftPacket::new(
                self.next_packet_id,
                PacketType::Request(0),
                command.to_owned()
            )
        ).await?;

        tokio::time::sleep(Duration::from_millis(DELAY_TIME_MILLIS)).await;

        let response = self.receive_response().await?;
        
        Ok(response)
    }

    /// Receives a response from the rcon server
    async fn receive_response(&mut self) -> io::Result<String> {
        // The server processes packets in order, so send an empty packet and
        // remember its id to detect the end of a multi-packt reponse
        let end_id = self.send_packet(
            MinecraftPacket::new(
                self.next_packet_id,
                PacketType::Request(0),
                "".to_string(),
            )
        ).await?;

        let mut result = String::with_capacity(48);

        loop {
            let received_packet = self.receive_packet().await?;

            if received_packet.get_id() == end_id {
                // This is the response to the end-marker packet
                return Ok(result);
            }

            result.push_str(received_packet.get_body());
        }

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
        MinecraftPacket::deserialize(&mut self.stream).await
    }
}


