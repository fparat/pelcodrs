#[macro_use]
extern crate bitflags;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    InvalidValue,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Error {
    kind: ErrorKind,
    description: String,
}

impl Error {
    pub fn new(kind: ErrorKind, description: &str) -> Error {
        Error {
            kind,
            description: String::from(description),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.description)
    }
}

type Result<T> = std::result::Result<T, Error>;

const MESSAGE_SIZE: usize = 7;

const SYNC_BYTE: u8 = 0xFF;
const TURBO: u8 = 0xFF;

bitflags! {
    pub struct Command1: u8 {
        const SENSE = 0x80;
        const AUTO_MANUAL_SCAN = 0x10;
        const CAMERA_ON_OFF = 0x08;
        const IRIS_CLOSE = 0x04;
        const IRIS_OPEN = 0x02;
        const FOCUS_NEAR = 0x01;
    }
}

bitflags! {
    pub struct Command2: u8 {
        const FOCUS_FAR = 0x80;
        const ZOOM_WIDE = 0x40;
        const ZOOM_TELE = 0x20;
        const DOWN = 0x10;
        const UP = 0x08;
        const LEFT = 0x04;
        const RIGHT = 0x02;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Message([u8; MESSAGE_SIZE]);

bitflags! {
    pub struct Direction: u8 {
        const UP = 1;
        const DOWN = 2;
        const LEFT = 4;
        const RIGHT = 8;
    }
}

impl Message {
    pub fn new(address: u8, cmd1: Command1, cmd2: Command2, data1: u8, data2: u8) -> Message {
        let mut msg: [u8; MESSAGE_SIZE] =
            [SYNC_BYTE, address, cmd1.bits, cmd2.bits, data1, data2, 0];
        msg[MESSAGE_SIZE - 1] = checksum(&msg[1..MESSAGE_SIZE]);
        Message(msg)
    }
}

impl AsRef<[u8]> for Message {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&MessageDraft> for Message {
    fn from(draft: &MessageDraft) -> Self {
        Message::new(
            draft.address,
            draft.cmd1,
            draft.cmd2,
            draft.data1,
            draft.data2,
        )
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MessageDraft {
    address: u8,
    cmd1: Command1,
    cmd2: Command2,
    data1: u8,
    data2: u8,
}

impl MessageDraft {
    pub fn new(address: u8) -> MessageDraft {
        MessageDraft {
            address,
            cmd1: Command1::empty(),
            cmd2: Command2::empty(),
            data1: 0,
            data2: 0,
        }
    }

    pub fn move_normal(&mut self, direction: Direction, speedx: f32, speedy: f32) -> &mut Self {
        unimplemented!();
    }

    pub fn move_turbo(&mut self, direction: Direction) -> &mut Self {
        unimplemented!();
    }

    pub fn up(&mut self, speed: f32) -> &mut Self {
        unimplemented!();
    }

    pub fn down(&mut self, speed: f32) -> &mut Self {
        unimplemented!();
    }

    pub fn left(&mut self, speed: f32) -> &mut Self {
        unimplemented!();
    }

    pub fn right(&mut self, speed: f32) -> &mut Self {
        unimplemented!();
    }

    pub fn stop(&mut self) -> &mut Self {
        unimplemented!();
    }

    pub fn zoom_in(&mut self) -> &mut Self {
        self.cmd2 |= Command2::ZOOM_TELE;
        self
    }

    pub fn zoom_out(&mut self) -> &mut Self {
        unimplemented!();
    }

    pub fn set_camera_on(&mut self) -> &mut Self {
        unimplemented!();
    }

    pub fn set_camera_off(&mut self) -> &mut Self {
        unimplemented!();
    }

    pub fn set_auto_scan(&mut self) -> &mut Self {
        unimplemented!();
    }

    pub fn set_manual_scan(&mut self) -> &mut Self {
        unimplemented!();
    }

    pub fn close_iris(&mut self) -> &mut Self {
        unimplemented!();
    }

    pub fn open_iris(&mut self) -> &mut Self {
        unimplemented!();
    }

    pub fn focus_near(&mut self) -> &mut Self {
        unimplemented!();
    }

    pub fn finalize(&self) -> Message {
        self.into()
    }
}

fn checksum(data: &[u8]) -> u8 {
    dbg!(data);
    let s: u32 = data.iter().map(|&b| u32::from(b)).sum();
    (s & 0xff) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        assert_eq!(0x62, checksum(&[0x0A, 0x88, 0x90, 0x00, 0x40]));
    }
}
