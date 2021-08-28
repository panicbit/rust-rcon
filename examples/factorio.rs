use rcon::{AsyncStdStream, Connection, Error};

#[async_std::main]
async fn main() -> Result<(), Error> {
    let address = "localhost:1234";
    let mut conn = <Connection<AsyncStdStream>>::builder()
        .enable_factorio_quirks(true)
        .connect(address, "test")
        .await?;

    demo(&mut conn, "/c print('hello')").await?;
    demo(&mut conn, "/c print('world')").await?;
    println!("commands finished");

    Ok(())
}

async fn demo(conn: &mut Connection<AsyncStdStream>, cmd: &str) -> Result<(), Error> {
    println!("request: {}", cmd);
    let resp = conn.cmd(cmd).await?;
    println!("response: {}", resp);
    Ok(())
}
