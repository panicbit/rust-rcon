use rcon::minecraft::MinecraftConnection;
use rcon::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let address = "127.0.0.1:1234";
    let mut conn = MinecraftConnection::connect(address, "test").await?;

    demo(&mut conn, "list").await?;
    demo(&mut conn, "say Rust lang rocks! ;P").await?;
    demo(&mut conn, "save-all").await?;

    Ok(())
}

async fn demo(conn: &mut MinecraftConnection, cmd: &str) -> Result<()> {
    let resp = conn.cmd(cmd).await?;
    println!("Response: {}", resp);
    Ok(())
}