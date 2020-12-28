mod packet;
pub mod error;
mod connection;

pub mod battlefield3;
pub mod factorio;
pub mod minecraft;

/// The prelude should import all the required traits and error related
/// elements, and event in the future macros
pub mod prelude {
    pub use super::{
        connection::Connection,
        error::Result,
    };
}
