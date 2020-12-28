use err_derive::Error;
use std::{
    io,
    result,
};

/// The different kind of errors that can happend while interacting with the 
/// rcon server
#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "authentication failed")]
    Auth,
    #[error(display = "command exceeds the maximum length")]
    CommandTooLong,
    #[error(display = "{}", _0)]
    Io(#[error(source)] io::Error),
}

/// The error result used throught the library
pub type Result<T> = result::Result<T, Error>;