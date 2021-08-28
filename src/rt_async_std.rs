use async_std::io::{Read, Write};
use async_std::net::{TcpStream, ToSocketAddrs};
use async_std::task::ready;
use std::io::{self, IoSlice};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead as TokioRead, AsyncWrite as TokioWrite, ReadBuf};

use crate::{Builder, Connection, Result};

impl Connection<AsyncStdStream> {
    /// Connect to an rcon server using the [async-std](async_std) runtime.
    ///
    /// By default this enables Minecraft quirks.
    /// If you need to customize this behaviour, use a [`Builder`].
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rt-async-std")))]
    pub async fn connect<A: ToSocketAddrs>(address: A, password: &str) -> Result<Self> {
        Self::builder()
            .enable_minecraft_quirks(true)
            .connect(address, password)
            .await
    }
}

impl Builder<AsyncStdStream> {
    /// Connect to an rcon server using the [async-std](async_std) runtime.
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rt-async-std")))]
    #[cfg_attr(not(feature = "tokio"), allow(unused_mut))]
    pub async fn connect<A: ToSocketAddrs>(
        mut self,
        address: A,
        password: &str,
    ) -> Result<Connection<AsyncStdStream>> {
        // If the `rt-tokio` feature flag is also enabled the sleep_fn will use it by default, so
        // we have to change it to use async-std instead.
        #[cfg(feature = "rt-tokio")]
        if let crate::SleepFn::Tokio = self.sleep_fn {
            self.sleep_fn = crate::SleepFn::AsyncStd;
        }
        self.handshake(AsyncStdStream(TcpStream::connect(address).await?), password)
            .await
    }
}

/// The inner transport of an [async-std](async_std) rcon connection.
///
/// This is a simple wrapper around a [`TcpStream`] that implements Tokio's I/O traits so it can be
/// used inside [`Connection`].
#[derive(Debug)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "rt-async-std")))]
pub struct AsyncStdStream(pub TcpStream);

impl TokioRead for AsyncStdStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let bytes = ready!(Pin::new(&mut self.0).poll_read(cx, buf.initialize_unfilled()))?;
        buf.advance(bytes);
        Poll::Ready(Ok(()))
    }
}

impl TokioWrite for AsyncStdStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write_vectored(cx, bufs)
    }
}
