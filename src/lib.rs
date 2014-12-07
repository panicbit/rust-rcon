use std::io::net::tcp::TcpStream;
use std::io::BufferedStream;
use std::io::IoResult;
use packet::{Packet, PacketType};
pub use error::RconResult;
pub use error::RconError;

mod packet;
mod error;

pub struct Connection {
    stream: BufferedStream<TcpStream>,
}

impl Connection {
    pub fn connect(address: &str, password: &str) -> RconResult<Connection> {
        let tcp_stream = try!(TcpStream::connect(address));
        let mut conn = Connection {
            stream: BufferedStream::new(tcp_stream),
        };

        try!(conn.auth(password));

        Ok(conn)
    }

    pub fn cmd(&mut self, cmd: &str) -> IoResult<String> {
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
    fn send(&mut self, ptype: PacketType, body: &str) -> IoResult<Packet> {
        let id = 0x504F4F50; // man ascii ;P
        let packet = Packet::new(id, ptype, body.to_string());

        //println!("Sending:\n{}", packet)

        try!(packet.serialize(&mut self.stream));

        let received_packet = try!(Packet::deserialize(&mut self.stream));

        //println!("Received:\n{}", received_packet)

        Ok(received_packet)
    }
}