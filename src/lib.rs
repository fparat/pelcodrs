//! Library for handling Pelco D, a protocol used for PTZ camera control.
//!
//! * [Message](struct.Message.html) is the type defining a control message.
//!
//! * [PelcoDPort](struct.PelcoDPort.html) can be used for sending the messages
//!   to the device.
//!

#[macro_use]
extern crate bitflags;

pub use error::*;
pub use message::*;
pub use port::*;
pub use response::*;

mod error;
mod message;
mod port;
mod response;

/// Checksum algorithm used by Pelco D.
pub fn checksum(data: &[u8]) -> u8 {
    let s: u32 = data.iter().map(|&b| u32::from(b)).sum();
    (s & 0xff) as u8
}
