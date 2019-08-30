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
const SPEED_TURBO_BYTE: u8 = 0xFF;
const SPEED_MAX_RANGE: f32 = 1.0;
const SPEED_MIN_RANGE: f32 = 0.0;

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
        // the bits are pre-located to the command position so that we can "or"
        // the bitfield when building the Command2 byte
        const DOWN  = 0b00010000;
        const UP    = 0b00001000;
        const LEFT  = 0b00000100;
        const RIGHT = 0b00000010;
    }
}

pub enum Speed {
    Range(f32),
    Turbo,
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

impl From<&MessageBuilder> for Message {
    fn from(draft: &MessageBuilder) -> Self {
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
pub struct MessageBuilder {
    address: u8,
    cmd1: Command1,
    cmd2: Command2,
    data1: u8,
    data2: u8,
}

impl MessageBuilder {
    pub fn new(address: u8) -> MessageBuilder {
        MessageBuilder {
            address,
            cmd1: Command1::empty(),
            cmd2: Command2::empty(),
            data1: 0,
            data2: 0,
        }
    }

    pub fn direction(&mut self, direction: Direction) -> &mut Self {
        self.cmd2 |= Command2::from_bits(direction.bits).unwrap();
        self
    }

    pub fn down(&mut self) -> &mut Self {
        self.direction(Direction::DOWN);
        self
    }

    pub fn up(&mut self) -> &mut Self {
        self.direction(Direction::UP);
        self
    }

    pub fn left(&mut self) -> &mut Self {
        self.direction(Direction::LEFT);
        self
    }

    pub fn right(&mut self) -> &mut Self {
        self.direction(Direction::RIGHT);
        self
    }

    pub fn pan(&mut self, speed: Speed) -> &mut Self {
        self.data1 = speed_to_byte(speed);
        self
    }

    pub fn tilt(&mut self, speed: Speed) -> &mut Self {
        self.data2 = speed_to_byte(speed);
        self
    }

    pub fn stop(&mut self) -> &mut Self {
        self.cmd1 = Command1::empty();
        self.cmd2 = Command2::empty();
        self.data1 = 0;
        self.data2 = 0;
        self
    }

    pub fn zoom_in(&mut self) -> &mut Self {
        self.cmd2 |= Command2::ZOOM_TELE;
        self
    }

    pub fn zoom_out(&mut self) -> &mut Self {
        self.cmd2 |= Command2::ZOOM_WIDE;
        self
    }

    pub fn camera_on(&mut self) -> &mut Self {
        self.cmd1 |= Command1::SENSE | Command1::CAMERA_ON_OFF;
        self
    }

    pub fn camera_off(&mut self) -> &mut Self {
        self.cmd1 |= Command1::CAMERA_ON_OFF;
        self
    }

    pub fn auto_scan(&mut self) -> &mut Self {
        self.cmd1 |= Command1::SENSE | Command1::AUTO_MANUAL_SCAN;
        self
    }

    pub fn manual_scan(&mut self) -> &mut Self {
        self.cmd1 |= Command1::AUTO_MANUAL_SCAN;
        self
    }

    pub fn close_iris(&mut self) -> &mut Self {
        self.cmd1 |= Command1::IRIS_CLOSE;
        self
    }

    pub fn open_iris(&mut self) -> &mut Self {
        self.cmd1 |= Command1::IRIS_OPEN;
        self
    }

    pub fn focus_far(&mut self) -> &mut Self {
        self.cmd2 |= Command2::FOCUS_FAR;
        self
    }

    pub fn focus_near(&mut self) -> &mut Self {
        self.cmd1 |= Command1::FOCUS_NEAR;
        self
    }

    pub fn finalize(&self) -> Result<Message> {
        Ok(self.into())
    }
}

pub fn checksum(data: &[u8]) -> u8 {
    let s: u32 = data.iter().map(|&b| u32::from(b)).sum();
    (s & 0xff) as u8
}

fn speed_to_byte(speed: Speed) -> u8 {
    match speed {
        Speed::Range(range) => {
            let range = if range > SPEED_MAX_RANGE {
                SPEED_MAX_RANGE
            } else if range < SPEED_MIN_RANGE {
                SPEED_MIN_RANGE
            } else {
                range
            };

            (((range / (SPEED_MAX_RANGE - SPEED_MIN_RANGE)) + SPEED_MIN_RANGE) * 63.0).round() as u8
        }
        Speed::Turbo => SPEED_TURBO_BYTE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        assert_eq!(0x62, checksum(&[0x0A, 0x88, 0x90, 0x00, 0x40]));
    }

    #[test]
    fn test_speed_to_byte() {
        assert_eq!(0, speed_to_byte(Speed::Range(0.0)));
        assert_eq!(0x3f, speed_to_byte(Speed::Range(1.0)));
        assert_eq!(0x26, speed_to_byte(Speed::Range(0.603)));
        assert_eq!(0xff, speed_to_byte(Speed::Turbo));
    }
}
