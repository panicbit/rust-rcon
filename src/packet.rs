// Copyright (c) 2015 [rust-rcon developers]
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
use async_trait::async_trait;

/// Representation of the Type of a packet, the number that contains each
/// variant represents some kind of bits into the raw data received or sended
/// with mark the type, in some rcon implementations its just 0 or 1, like in
/// Battlefield but in minecraft the Authorization and AuthorizationResponse are
/// different from normal Requests and Responses so its implementation specific.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketType {
    Request(i32),
    Response(i32),
    Custom(i32),
}

/// NOTE: The `#[async_trait]` macro is to be able to declare async functions 
/// inside the trait. Maybe in the future is its stabilized remove the
/// dependency
#[async_trait]
pub trait Packet: Sized {
    /// Checks if the packet is an error, probably just useful for Responses
    fn is_error(&self) -> bool;

    /// Serializes de packets, aka convert and send
    async fn serialize<T: Unpin + AsyncWrite + Send>(&self, w: &mut T) -> io::Result<()>;

    /// Deserializes de packets, aka receive and convert
    async fn deserialize<T: Unpin + AsyncRead + Send>(r: &mut T) -> io::Result<Self>;

    /// Gets the body of the packet
    fn get_body(&self) -> &str; 
    /// Gets the packet type
    fn get_type(&self, is_response: bool) -> PacketType;

    /// Returns the id of the packet and also increments it
    fn get_id(&self) -> i32;
}