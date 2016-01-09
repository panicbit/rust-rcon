// Copyright (c) 2015 [rust-rcon developers]
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

extern crate bufstream;
extern crate podio;

use std::net::{TcpStream, ToSocketAddrs};
use std::io;
use packet::{Packet, PacketType};
use bufstream::BufStream;
pub use error::RconResult;
pub use error::RconError;

mod packet;
mod error;

pub struct Connection {
    stream: BufStream<TcpStream>,
}

impl Connection {
    pub fn connect<T: ToSocketAddrs>(address: T, password: &str) -> RconResult<Connection> {
        let tcp_stream = try!(TcpStream::connect(address));
        let mut conn = Connection {
            stream: BufStream::new(tcp_stream),
        };

        try!(conn.auth(password));

        Ok(conn)
    }

    pub fn cmd(&mut self, cmd: &str) -> io::Result<String> {
        let send_result = self.send(PacketType::ExecCommand, cmd);
        let received_packet = try!(send_result);
        Ok(received_packet.get_body().to_string())
    }

    fn auth(&mut self, password: &str) -> RconResult<()> {
        let send_result = self.send(PacketType::Auth, password);
        let received_packet = try!(send_result);

        if received_packet.is_error() {
            Err(RconError::Auth)
        } else {
            Ok(())
        }
    }

    // TODO: implement packet splitting
    fn send(&mut self, ptype: PacketType, body: &str) -> io::Result<Packet> {
        let id = 0x504F4F50; // man ascii ;P
        let packet = Packet::new(id, ptype, body.to_string());

        //println!("Sending:\n{}", packet)

        try!(packet.serialize(&mut self.stream));

        let received_packet = try!(Packet::deserialize(&mut self.stream));

        //println!("Received:\n{}", received_packet)

        Ok(received_packet)
    }
}