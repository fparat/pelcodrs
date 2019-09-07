use std::io::{Read, Write};

use crate::message::*;
use crate::Result;

/// Interface for communicating with a device using Pelco D protocol.
///
/// This structure encapsulates a `Read + Write` object, and uses it for
/// communicating with the target Pelco D device.
///
/// For example, a `SerialPort` object from the crate
/// [serialport](https://crates.io/crates/serialport) can be used as port:
///
/// ```rust, ignore
/// use serialport;
/// use pelcodrs::{PelcoDPort, Message};
///
/// # use std::error::Error;
/// # fn example() -> Result<(), Box<dyn Error>> {
/// let pelcod_dev = PelcoDPort::new(serialport::open("/dev/ttyS0"));
/// pelcod_dev.send_message(Message::flip_180(10)?)?;
/// # Ok(())}
/// ```
pub struct PelcoDPort<T: Read + Write>(T);

impl<T: Read + Write> PelcoDPort<T> {
    pub fn new(ser: T) -> PelcoDPort<T> {
        PelcoDPort(ser)
    }
}

impl<T: Read + Write> Read for PelcoDPort<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        self.0.read(buf)
    }
}

impl<T: Read + Write> Write for PelcoDPort<T> {
    fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        self.0.flush()
    }
}

impl<T: Read + Write> PelcoDPort<T> {
    pub fn send_message(&mut self, message: Message) -> Result<()> {
        Ok(self.write_all(message.as_ref())?)
    }
}
