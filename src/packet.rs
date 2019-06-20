// Copyright (c) 2015 [rust-rcon developers]
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::io::{self, Read, Write};
use std::fmt;
use podio::{LittleEndian, ReadPodExt, WritePodExt};

type LE = LittleEndian;

#[derive(Debug)]
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
            id,
            ptype: ptype.to_i32(),
            body,
            is_response: false
        }
    }

    pub fn is_error(&self) -> bool {
        self.id < 0
    }

    pub fn serialize<T: Write>(&self, w: &mut T) -> io::Result<()> {
        let length = 10 + self.body.len();

        // length
        w.write_i32::<LE>(length as i32)?;
        // id
        w.write_i32::<LE>(self.id)?;
        // type
        w.write_i32::<LE>(self.ptype)?;
        // body
        write!(w, "{}", &self.body)?;
        // terminating nulls
        w.write_u8(0)?;
        w.write_u8(0)?;

        w.flush()?;

        Ok(())
    }

    pub fn deserialize<T: Read>(r: &mut T) -> io::Result<Packet> {
        // length
        let length = r.read_i32::<LE>()?;
        // id
        let id = r.read_i32::<LE>()?;
        // type
        let ptype = r.read_i32::<LE>()?;
        // body
        let body_length = length - 10;
        let mut body_buffer = Vec::with_capacity(body_length as usize);
        r.take(body_length as u64).read_to_end(&mut body_buffer)?;
        let body = String::from_utf8(body_buffer).ok().unwrap();
        // terminating nulls
        r.read_u8()?;
        r.read_u8()?;

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
        &self.body
    }

    pub fn get_type(&self) -> PacketType {
        PacketType::from_i32(self.ptype, self.is_response)
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
r"Packet {{
    len: {:?},
    id: {:?},
    type: {:?},
    body: {:?}
}}",
        self.length, self.id, self.get_type(), self.body)
    }
}
