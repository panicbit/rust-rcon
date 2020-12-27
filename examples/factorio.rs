use rcon::factorio::FactorioConnection;
use rcon::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let address = "127.0.0.1:1234";
    let mut conn = FactorioConnection::connect(address, "test").await?;

    demo(&mut conn, "/c print('hello')").await?;
    demo(&mut conn, "/c print('world')").await?;

    Ok(())
}

async fn demo(conn: &mut FactorioConnection, cmd: &str) -> Result<()> {
    println!("request: {}", cmd);
    let resp = conn.cmd(cmd).await?;
    println!("response: {}", resp);
    Ok(())
}