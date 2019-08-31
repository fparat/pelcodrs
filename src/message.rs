use crate::error::Result;

const MESSAGE_SIZE: usize = 7;

const SYNC_BYTE: u8 = 0xFF;
const SPEED_TURBO_BYTE: u8 = 0xFF;
const SPEED_MAX_RANGE: f32 = 1.0;
const SPEED_MIN_RANGE: f32 = 0.0;

pub enum Speed {
    Range(f32),
    Turbo,
}

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
    pub struct Direction: u8 {
        // the bits are pre-located to the command position so that we can "or"
        // the bitfield when building the Command2 byte
        const DOWN  = 0b0001_0000;
        const UP    = 0b0000_1000;
        const LEFT  = 0b0000_0100;
        const RIGHT = 0b0000_0010;
    }
}

/// Single command message object type
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Message([u8; MESSAGE_SIZE]);

impl Message {
    /// New "standard" command message. This constructor cannot be used for
    /// "extended" commands, for which [Message::from_btyes()] should be used.
    pub fn new(address: u8, cmd1: Command1, cmd2: Command2, data1: u8, data2: u8) -> Message {
        let mut msg = Message([SYNC_BYTE, address, cmd1.bits, cmd2.bits, data1, data2, 0]);
        msg.fill_checksum();
        msg
    }

    /// Alternate constructor taking the raw words to insert in the message.
    /// The sync byte and checksum automatically inserted.
    pub fn from_bytes(address: u8, words: [u8; 4]) -> Message {
        let mut msg = Message([
            SYNC_BYTE, address, words[0], words[1], words[2], words[3], 0,
        ]);
        msg.fill_checksum();
        msg
    }

    fn fill_checksum(&mut self) {
        self.0[MESSAGE_SIZE - 1] = checksum(&self.0[1..MESSAGE_SIZE]);
    }
}

impl AsRef<[u8]> for Message {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<MessageBuilder> for Message {
    fn from(draft: MessageBuilder) -> Self {
        Message::new(
            draft.address,
            draft.cmd1,
            draft.cmd2,
            draft.data1,
            draft.data2,
        )
    }
}

/// Builder of standard command messages. Provided methods fill the message
/// with corresponding bits, then .finalize() produce the final Message.
/// Please note that currently no logical validation is done. For example the
/// "sense" bit can be overwritten, and thus invalidate previous method call.
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

    pub fn finalize(self) -> Result<Message> {
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
