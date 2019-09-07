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

mod error;
mod message;
mod port;
