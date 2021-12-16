use tokio::net::{TcpStream, ToSocketAddrs};

use crate::{Builder, Connection, Result};

impl Connection<TcpStream> {
    /// Connect to an rcon server using the [Tokio](tokio) runtime.
    ///
    /// By default this enables Minecraft quirks.
    /// If you need to customize this behaviour, use a [`Builder`].
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rt-tokio")))]
    pub async fn connect<A: ToSocketAddrs>(address: A, password: &str) -> Result<Self> {
        Self::builder()
            .enable_minecraft_quirks(true)
            .connect(address, password)
            .await
    }
}

impl Builder<TcpStream> {
    /// Connect to an rcon server using the [Tokio](tokio) runtime.
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rt-tokio")))]
    pub async fn connect<A: ToSocketAddrs>(
        self,
        address: A,
        password: &str,
    ) -> Result<Connection<TcpStream>> {
        self.handshake(TcpStream::connect(address).await?, password)
            .await
    }
}
