use std::io::IoResult;
use std::fmt;

#[deriving(Show)]
pub enum PacketType {
    Auth,
    AuthResponse,
    ExecCommand,
    ResponseValue,
    Unknown(i32)
}

impl PacketType {
    fn to_i32(self) -> i32 {
        match self {
            PacketType::Auth => 3,
            PacketType::AuthResponse => 2,
            PacketType::ExecCommand => 2,
            PacketType::ResponseValue => 0,
            PacketType::Unknown(n) => n
        }
    }

    pub fn from_i32(n: i32, is_response: bool) -> PacketType {
        match n {
            3 => PacketType::Auth,
            2 if is_response => PacketType::AuthResponse,
            2 => PacketType::ExecCommand,
            0 => PacketType::ResponseValue,
            n => PacketType::Unknown(n)
        }
    }
}

pub struct Packet {
    length: i32,
    id: i32,
    ptype: i32,
    body: String,
    is_response: bool
}

impl Packet {
    pub fn new(id: i32, ptype: PacketType, body: String) -> Packet {
        Packet {
            length: 10 + body.len() as i32,
            id: id,
            ptype: ptype.to_i32(),
            body: body,
            is_response: false
        }
    }

    pub fn is_error(&self) -> bool {
        self.id < 0
    }

    pub fn serialize<T: Writer>(&self, w: &mut T) -> IoResult<()> {
        let length = 10 + self.body.len();

        // length
        try!(w.write_le_i32(length as i32));
        // id
        try!(w.write_le_i32(self.id));
        // type
        try!(w.write_le_i32(self.ptype));
        // body
        try!(w.write_str(self.body.as_slice()));
        // terminating nulls
        try!(w.write_char('\0'));
        try!(w.write_char('\0'));

        try!(w.flush());

        Ok(())
    }

    pub fn deserialize<T: Reader>(r: &mut T) -> IoResult<Packet> {
        // length
        let length = try!(r.read_le_i32());
        // id
        let id = try!(r.read_le_i32());
        // type
        let ptype = try!(r.read_le_i32());
        // body
        let body_length = (length - 10) as uint;
        let body = String::from_utf8(try!(r.read_exact(body_length as uint))).ok().unwrap();
        // terminating nulls
        try!(r.read_byte());
        try!(r.read_byte());

        let packet = Packet {
            length: length,
            id: id,
            ptype: ptype,
            body: body,
            is_response: true
        };

        Ok(packet)
    }

    pub fn get_body(&self) -> &str {
        self.body.as_slice()
    }

    pub fn get_type(&self) -> PacketType {
        PacketType::from_i32(self.ptype, self.is_response)
    }
}

impl fmt::Show for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
r"Packet {{
    len: {},
    id: {},
    type: {},
    body: {}
}}",
        self.length, self.id, self.get_type(), self.body)
    }
}