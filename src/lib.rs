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
    fn new(kind: ErrorKind, description: &str) -> Error {
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

bitflags! {
    struct MessageDraftStatus: u8 {
        const SYNC = 0b01000000;
        const ADDR = 0b00100000;
        const CMD1 = 0b00010000;
        const CMD2 = 0b00001000;
        const DAT1 = 0b00000100;
        const DAT2 = 0b00000010;
        const CHKS = 0b00000001;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct MessageDraft {
    msg: [u8; MESSAGE_SIZE],
    status: MessageDraftStatus,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Message([u8; MESSAGE_SIZE]);

impl Message {
    pub fn new(address: u8) -> MessageDraft {
        MessageDraft {
            msg: [SYNC_BYTE, address, 0, 0, 0, 0, 0],
            status: MessageDraftStatus::SYNC | MessageDraftStatus::ADDR,
        }
    }
}

impl AsRef<[u8]> for Message {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl MessageDraft {
    pub fn with_command1(&mut self, cmd: Command1) -> &mut Self {
        self.msg[2] = cmd.bits;
        self.status |= MessageDraftStatus::CMD1;
        self
    }

    pub fn with_command2(&mut self, cmd: Command2) -> &mut Self {
        self.msg[3] = cmd.bits;
        self.status |= MessageDraftStatus::CMD2;
        self
    }

    pub fn with_data1(&mut self, data: u8) -> &mut Self {
        self.msg[4] = data;
        self.status |= MessageDraftStatus::DAT1;
        self
    }

    pub fn with_data2(&mut self, data: u8) -> &mut Self {
        self.msg[5] = data;
        self.status |= MessageDraftStatus::DAT2;
        self
    }

    pub fn with_checksum(&mut self) -> &mut Self {
        self.msg[6] = checksum(&self.msg[1..self.msg.len() - 1]);
        self.status |= MessageDraftStatus::CHKS;
        self
    }

    pub fn finalize(&mut self) -> Result<Message> {
        if self.with_checksum().status.is_all() {
            Ok(Message { 0: self.msg })
        } else {
            Err(Error::new(
                ErrorKind::InvalidValue,
                &format!("Missing message fields: {:?}", self.status),
            ))
        }
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
