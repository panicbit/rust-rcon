use rcon::battlefield3::Btf3Connection;
use rcon::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let address = "127.0.0.1:47201";
    let mut conn = Btf3Connection::connect(address, "root").await?;

    demo(&mut conn, "serverInfo").await?;

    Ok(())
}

async fn demo(conn: &mut Btf3Connection, cmd: &str) -> Result<()> {
    println!("request: {}", cmd);
    let resp = conn.cmd(cmd).await?;
    println!("response: {}", resp);
    Ok(())
}