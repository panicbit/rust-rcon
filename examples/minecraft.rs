// Copyright (c) 2015 [rust-rcon developers]
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

extern crate rcon;

/*
    This example expects a Minecraft with rcon enabled on port 25575
    and the rcon password "test"
*/

fn main() {
    let address = "localhost:25575";
    let mut conn = rcon::Connection::connect(address, "test").unwrap();

    demo(&mut conn, "list");
    demo(&mut conn, "say Rust lang rocks! ;P");
    demo(&mut conn, "save-all");
    //demo(&mut conn, "stop");
}

fn demo(conn: &mut rcon::Connection, cmd: &str) {
    let resp = conn.cmd(cmd).unwrap();
    println!("{}", resp);
}