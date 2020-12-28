use super::error::Result;
use std::io;
use tokio::net::ToSocketAddrs;
use async_trait::async_trait;

/// NOTE: the `#[async_trait]` macro is to be able to declare async functions
/// NOTE: The connection at least must contain two fields
///     - stream: TcpStream
///     - id: i32
/// Representation of an Rcon Connection, in the high level it con `connect` 
/// with or without password and the it can send commnands `cmd` and it can 
/// `receive_response`, also some low level functions like `receive_packet` and
/// `send_packet` are needed to them be wrapped inside the high level ones
#[async_trait]
pub trait Connection: Sized {
    type Packet;
    /// Connects to a rcon server, if the password is mandatory is implementation
    /// specific, but ensure in the implementation to allow empty string `""` 
    /// for no password provided and no auth needed
    async fn connect<T: ToSocketAddrs + Send>(address: T, password: &str) -> Result<Self>;

    /// Send a certain command, in the implementation the `command` should be
    /// parsed to a packet ready bytes buffer and sended via `send_packet`
    async fn cmd(&mut self, command: &str) -> Result<String>;

    /// Receives a response from the rcon server, in the implementation it must
    /// call receive packet as many times it requires parse the body and present
    /// it as a clear response String
    async fn receive_response(&mut self) -> io::Result<String>;

    /// Low level function that send a Packet, returns the `id` of the sended 
    /// packet to be incremented
    async fn send_packet(&mut self, packet: Self::Packet) -> io::Result<i32>;

    /// Low level function that receives a Packet
    async fn receive_packet(&mut self) -> io::Result<Self::Packet>;
}

