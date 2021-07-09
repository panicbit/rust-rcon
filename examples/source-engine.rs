use rcon::{Connection, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let address = "localhost:27015";
    let mut conn = Connection::builder()
        .connect(address, "test").await?;

    demo(&mut conn, "status").await?;
    demo(&mut conn, "users").await?;
    demo(&mut conn, "echo \"Rust lang rocks! ;P\"").await?;
    println!("commands finished");

    Ok(())
}

async fn demo(conn: &mut Connection, cmd: &str) -> Result<(), Error> {
    println!("request: {}", cmd);
    let resp = conn.cmd(cmd).await?;
    println!("response: {}", resp);
    Ok(())
}
