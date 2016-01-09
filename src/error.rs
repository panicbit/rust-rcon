// Copyright (c) 2015 [rust-rcon developers]
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::io;
use std::error::Error;
use std::fmt;
use std::convert::From;

pub type RconResult<T> = Result<T, RconError>;

#[derive(Debug)]
pub enum RconError {
    Auth,
    Other(Box<Error>)
}

impl Error for RconError {
    fn description(&self) -> &str {
        match *self {
            RconError::Auth => "authentication failed",
            RconError::Other(ref err) => err.description()
        }
    }
}

impl fmt::Display for RconError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl From<io::Error> for RconError {
    fn from(err: io::Error) -> RconError {
        RconError::Other(Box::new(err))
    }
}
