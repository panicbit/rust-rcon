// Copyright (c) 2015 [rust-rcon developers]
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.
//! Asynchronous API for the RCON protocol used by games such as Minecraft and Factorio.
//!
//! # Feature flags
//!
//! - `rt-tokio`: Enable integration with the [Tokio](tokio) asynchronous runtime.
//! - `rt-async-std`: Enable integration with the [async-std](async_std) asynchronous runtime.
#![cfg_attr(doc_cfg, feature(doc_cfg))]

use err_derive::Error;
use packet::{Packet, PacketType};
use std::future::Future;
use std::io;
use std::fmt::{self, Formatter, Debug};
use std::marker::PhantomData;
use std::pin::Pin;
use std::time::Duration;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};

#[cfg(feature = "rt-async-std")]
mod rt_async_std;
#[cfg(feature = "rt-async-std")]
pub use rt_async_std::AsyncStdStream;

#[cfg(feature = "rt-tokio")]
mod rt_tokio;

mod packet;

const INITIAL_PACKET_ID: i32 = 1;
const DELAY_TIME_MILLIS: u64 = 3;
const MINECRAFT_MAX_PAYLOAD_SIZE: usize = 1413;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "authentication failed")]
    Auth,
    #[error(display = "command exceeds the maximum length")]
    CommandTooLong,
    #[error(display = "{}", _0)]
    Io(#[error(source)] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Connection<T> {
    io: T,
    next_packet_id: i32,
    minecraft_quirks_enabled: bool,
    factorio_quirks_enabled: bool,
    sleep_fn: SleepFn,
}

impl<T: AsyncRead + AsyncWrite + Unpin> Connection<T> {
    /// Create a connectiion builder.
    /// Allows configuring the rcon connection.
    pub fn builder() -> Builder<T> {
        Builder::new()
    }

    /// Perform a handshake on an existing connection to an rcon server.
    ///
    /// This is a lower-level method mostly useful when integrating this crate with another
    /// runtime, or running rcon over a transport other than TCP. You generally will want to use
    /// one of the higher-level `connect` methods.
    ///
    /// By default this enables Minecraft quirks.
    /// If you need to customize this behaviour, use a [`Builder`].
    ///
    /// This method requires one of the runtime features to be activated so that Minecraft quirks
    /// mode is able to asynchronously sleep. If you want to provide a custom sleep function, see
    /// [`Builder::sleep_fn`].
    #[cfg(any(feature = "rt-tokio", feature = "rt-async-std"))]
    #[cfg_attr(doc_cfg, doc(cfg(any(feature = "rt-tokio", feature = "rt-async-std"))))]
    pub async fn handshake(io: T, password: &str) -> Result<Self> {
        Self::builder()
            .enable_minecraft_quirks(true)
            .handshake(io, password)
            .await
    }

    pub async fn cmd(&mut self, cmd: &str) -> Result<String> {
        if self.minecraft_quirks_enabled && cmd.len() > MINECRAFT_MAX_PAYLOAD_SIZE {
            return Err(Error::CommandTooLong);
        }

        self.send(PacketType::ExecCommand, cmd).await?;

        if self.minecraft_quirks_enabled {
            self.sleep_fn
                .call(Duration::from_millis(DELAY_TIME_MILLIS))
                .await;
        }

        let response = self.receive_response().await?;

        Ok(response)
    }

    async fn receive_response(&mut self) -> Result<String> {
        if self.factorio_quirks_enabled {
            self.receive_single_packet_response().await
        } else {
            self.receive_multi_packet_response().await
        }
    }

    async fn receive_single_packet_response(&mut self) -> Result<String> {
        let received_packet = self.receive_packet().await?;

        Ok(received_packet.get_body().into())
    }

    async fn receive_multi_packet_response(&mut self) -> Result<String> {
        // the server processes packets in order, so send an empty packet and
        // remember its id to detect the end of a multi-packet response
        let end_id = self.send(PacketType::ExecCommand, "").await?;

        let mut result = String::new();

        loop {
            let received_packet = self.receive_packet().await?;

            if received_packet.get_id() == end_id {
                // This is the response to the end-marker packet
                return Ok(result);
            }

            result += received_packet.get_body();
        }
    }

    async fn auth(&mut self, password: &str) -> Result<()> {
        self.send(PacketType::Auth, password).await?;
        let received_packet = loop {
            let received_packet = self.receive_packet().await?;
            if received_packet.get_type() == PacketType::AuthResponse {
                break received_packet;
            }
        };

        if received_packet.is_error() {
            Err(Error::Auth)
        } else {
            Ok(())
        }
    }

    async fn send(&mut self, ptype: PacketType, body: &str) -> io::Result<i32> {
        let id = self.generate_packet_id();

        let packet = Packet::new(id, ptype, body.into());

        packet.serialize(&mut self.io).await?;

        Ok(id)
    }

    async fn receive_packet(&mut self) -> io::Result<Packet> {
        Packet::deserialize(&mut self.io).await
    }

    fn generate_packet_id(&mut self) -> i32 {
        let id = self.next_packet_id;

        // only use positive ids as the server uses negative ids to signal
        // a failed authentication request
        self.next_packet_id = self
            .next_packet_id
            .checked_add(1)
            .unwrap_or(INITIAL_PACKET_ID);

        id
    }
}

#[derive(Clone)]
enum SleepFn {
    #[cfg(feature = "rt-tokio")]
    Tokio,
    #[cfg(feature = "rt-async-std")]
    AsyncStd,
    #[cfg(not(any(feature = "rt-tokio", feature = "rt-async-std")))]
    None,
    Custom(CustomSleepFn),
}

impl Debug for SleepFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "rt-tokio")]
            Self::Tokio => f.write_str("tokio::time::sleep"),
            #[cfg(feature = "rt-async-std")]
            Self::AsyncStd => f.write_str("async_std::task::sleep"),
            #[cfg(not(any(feature = "rt-tokio", feature = "rt-async-std")))]
            Self::None => f.write_str("None"),
            Self::Custom(_) => f.write_str("custom sleep function"),
        }
    }
}

type CustomSleepFn = Arc<dyn Fn(Duration) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

impl SleepFn {
    async fn call(&mut self, duration: Duration) {
        match self {
            #[cfg(feature = "rt-tokio")]
            Self::Tokio => tokio::time::sleep(duration).await,
            #[cfg(feature = "rt-async-std")]
            Self::AsyncStd => async_std::task::sleep(duration).await,
            #[cfg(not(any(feature = "rt-tokio", feature = "rt-async-std")))]
            Self::None => unreachable!(),
            Self::Custom(f) => f(duration).await,
        }
    }
}

#[derive(Debug)]
pub struct Builder<T> {
    minecraft_quirks_enabled: bool,
    factorio_quirks_enabled: bool,
    sleep_fn: SleepFn,
    _io: PhantomData<fn() -> T>,
}

impl<T> Default for Builder<T> {
    fn default() -> Self {
        #[cfg(feature = "rt-tokio")]
        let sleep_fn = SleepFn::Tokio;
        #[cfg(all(feature = "rt-async-std", not(feature = "rt-tokio")))]
        let sleep_fn = SleepFn::AsyncStd;
        #[cfg(not(any(feature = "rt-async-std", feature = "rt-tokio")))]
        let sleep_fn = SleepFn::None;

        Self {
            minecraft_quirks_enabled: false,
            factorio_quirks_enabled: false,
            sleep_fn,
            _io: PhantomData,
        }
    }
}

impl<T> Clone for Builder<T> {
    fn clone(&self) -> Self {
        Self {
            minecraft_quirks_enabled: self.minecraft_quirks_enabled,
            factorio_quirks_enabled: self.factorio_quirks_enabled,
            sleep_fn: self.sleep_fn.clone(),
            _io: PhantomData,
        }
    }
}

impl<T> Builder<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// This enables the following quirks for Minecraft:
    ///
    /// Commands are delayed by 3ms to reduce the chance of crashing the server.
    /// See <https://bugs.mojang.com/browse/MC-72390>.
    ///
    /// The command length is limited to 1413 bytes.
    /// Tests have shown the server to not work reliably
    /// with greater command lengths.
    pub fn enable_minecraft_quirks(mut self, value: bool) -> Self {
        self.minecraft_quirks_enabled = value;
        self
    }

    /// This enables the following quirks for Factorio:
    ///
    /// Only single-packet responses are enabled.
    /// Multi-packets appear to work differently than in other server implementations
    /// (an empty packet gives no response).
    pub fn enable_factorio_quirks(mut self, value: bool) -> Self {
        self.factorio_quirks_enabled = value;
        self
    }

    /// Set a custom function to use for sleeping between requests when [Minecraft quirks mode is
    /// enabled](Self::enable_minecraft_quirks).
    ///
    /// When either of the `rt-tokio` or `rt-async-std` feature flags is enabled, this library will
    /// default to using the runtime's native sleeping function. This can be used to override it,
    /// or set the sleeping function to use when no runtime feature is active.
    ///
    /// # Example
    ///
    /// Using [futures-timer](https://docs.rs/futures-timer) instead of Tokio's native timer:
    ///
    /// ```
    /// # use tokio::net::TcpStream;
    /// # async {
    /// let connection = <rcon::Connection<TcpStream>>::builder()
    ///     .enable_minecraft_quirks(true)
    ///     .sleep_fn(futures_timer::Delay::new)
    ///     .connect("localhost:25575", "hunter2")
    ///     .await?;
    /// # drop(connection);
    /// # rcon::Result::Ok(())
    /// # };
    /// ```
    pub fn sleep_fn<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(Duration) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.sleep_fn = SleepFn::Custom(Arc::new(move |duration| {
            Box::pin(f(duration))
        }));
        self
    }

    /// Perform a handshake on an existing connection to an rcon server.
    ///
    /// This is a lower-level method mostly useful when integrating this crate with another
    /// runtime, or running rcon over a transport other than TCP. You generally will want to use
    /// one of the higher-level `connect` methods.
    ///
    /// # Panics
    ///
    /// If neither of the `rt-tokio` or `rt-async-std` feature flags are activated, no [custom sleep
    /// function](Self::sleep_fn) has been set and [Minecraft quirks](Self::enable_minecraft_quirks)
    /// have been enabled, this function will panic as Minecraft quirks need some way to
    /// asynchronously sleep.
    pub async fn handshake(self, io: T, password: &str) -> Result<Connection<T>>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        #[cfg(not(any(feature = "rt-tokio", feature = "rt-async-std")))]
        if self.minecraft_quirks_enabled && matches!(self.sleep_fn, SleepFn::None) {
            panic!(
                "\
                Minecraft quirks mode is enabled, but no runtime or custom sleep function has been \
                set. Enable one of the `rt-tokio` or `rt-async-std` feature flags, or set a custom \
                sleep function with `rcon::Builder::sleep_fn`.\
            "
            );
        }

        let mut conn = Connection {
            io,
            next_packet_id: INITIAL_PACKET_ID,
            minecraft_quirks_enabled: self.minecraft_quirks_enabled,
            factorio_quirks_enabled: self.factorio_quirks_enabled,
            sleep_fn: self.sleep_fn,
        };

        conn.auth(password).await?;

        Ok(conn)
    }
}
