use std::io::IoError;
use std::error::{Error, FromError};
use std::fmt;

pub type RconResult<T> = Result<T, RconError>;

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

    fn detail(&self) -> Option<String> {
        match *self {
            RconError::Auth => None,
            RconError::Other(ref err) => err.detail()
        }
    }
}

impl fmt::Show for RconError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let desc = self.description();
        match self.detail() {
            Some(detail) => write!(fmt, "{} ({})", desc, detail),
            None => write!(fmt, "{}", desc)
        }
    }
}

impl FromError<IoError> for RconError {
    fn from_error(err: IoError) -> RconError {
        RconError::Other(box err)
    }
}
