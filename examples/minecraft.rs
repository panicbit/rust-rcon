// Copyright (c) 2015 [rust-rcon developers]
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use rcon::{Connection, Error};

/*
    This example expects a Minecraft with rcon enabled on port 25575
    and the rcon password "test"
*/

#[tokio::main]
async fn main() -> Result<(), Error> {
    let address = "localhost:25575";
    let mut conn = Connection::connect(address, "test").await?;

    demo(&mut conn, "list").await?;
    demo(&mut conn, "say Rust lang rocks! ;P").await?;
    demo(&mut conn, "save-all").await?;
    //demo(&mut conn, "stop");
    Ok(())
}

async fn demo(conn: &mut Connection, cmd: &str) -> Result<(), Error> {
    let resp = conn.cmd(cmd).await?;
    println!("{}", resp);
    Ok(())
}