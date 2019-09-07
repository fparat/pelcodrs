use crate::error::*;
use std::convert::TryFrom;

const MESSAGE_SIZE: usize = 7;

const SYNC_BYTE: u8 = 0xFF;
const SPEED_TURBO_BYTE: u8 = 0xFF;
const SPEED_MAX_RANGE: f32 = 1.0;
const SPEED_MIN_RANGE: f32 = 0.0;

/// Speed parameter type for pan and tilt moves.
///
/// The `Range` value must be between 0.0 and 1.0.
/// `Turbo` works only for pan movements.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Speed {
    Range(f32),
    Turbo,
}

/// Argument type for Zoom Speed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoomSpeed {
    Slow = 0,
    Medium = 1,
    High = 2,
    Highest = 3,
}

/// Argument type for FocusSpeed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusSpeed {
    Slow = 0,
    Medium = 1,
    High = 2,
    Highest = 3,
}

/// Argument type for Auto/On/Off
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoCtrl {
    Auto = 0,
    Off = 1,
}

/// On/Off argument type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OnOff {
    On,
    Off,

    /// Manual value for specific device like Spectra IV
    Value(u8),
}

/// Argument type for shutter speed.
///
/// Bytes can be manually given to the function using `Bytes`.
/// Other variants are provided for convenience, but should be used in
/// accordance to the target device capabilities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShutterSpeed {
    /// Manual bytes input (byte 5, byte 6)
    Bytes(u8, u8),
    /// Spectra II and older
    DefaultValue,
    /// Spectra II and older
    Increment,
    /// Spectra II and older
    Decrement,
    /// Spectra II and older
    PAL,
    /// Spectra II and older
    NTSC,
    /// Spectra II and older
    Value(u16),
    /// Spectra III and newer
    AutoShutter,
    /// Spectra III and newer
    Index(u8),
}

/// Argument type for value adjustment functions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdjustmentValue {
    New(u16),
    Delta(i16),
}

bitflags! {
    /// Bitflag for generating the "command1" word of the message.
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
    /// Bitflag for generating the "command2" word of the message.
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
    /// Bitflag for direction parameter.
    pub struct Direction: u8 {
        // the bits are pre-located to the command position so that we can "or"
        // the bitfield when building the Command2 byte
        const DOWN  = 0b0001_0000;
        const UP    = 0b0000_1000;
        const LEFT  = 0b0000_0100;
        const RIGHT = 0b0000_0010;
    }
}

/// Single command message object type.
///
/// There are several way to build a message:
///
///  * The main constructor [Message::new()](struct.Message.html#method.new) for
///    "standard" messages.
///
///  * Use the builder pattern with [MessageBuilder](struct.MessageBuilder.html).
///
///  * Many convenient constructors are provided for "extended" commands.
///
///  * [Message::from_bytes()](struct.Message.html#method.from_bytes) allows
///    to manually specify the address and data words bytes, and will
///    automatically fill the sync byte and checksum.
///
///  * Full control of all the bytes values is possible with the trait
///    implementaion
///    [From<[u8;7]>](struct.Message.html#impl-From%3C%5Bu8%3B%207%5D%3E).
///
/// The bytes of a `Message` can be accessed with `Message::as_ref()`.
/// 
/// `Message` objects can also be sent to the target device with
/// [PelcoDPort::send_message()](struct.PelcoDPort.html#method.send_message).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Message([u8; MESSAGE_SIZE]);

impl Message {
    /// New "standard" command message. This constructor cannot be used for
    /// "extended" commands, for which dedicated constructors or
    /// [Message::from_bytes()](struct.Message.html#method.from_bytes) should
    /// be used.
    ///
    /// Example:
    ///
    /// ```
    /// # use pelcodrs::*;
    /// let msg = Message::new(
    ///     10,
    ///     Command1::SENSE | Command1::CAMERA_ON_OFF,
    ///     Command2::FOCUS_FAR | Command2::DOWN,
    ///     0x00,
    ///     0x40,
    /// );
    /// assert_eq!(&[0xFF, 0x0A, 0x88, 0x90, 0x00, 0x40, 0x62], msg.as_ref());
    /// ```
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

    // Extended commands constructors

    /// Set Preset. An error is returned if `preset_id` is 0, but special values
    /// (like "flip 180") are not checked.
    pub fn set_preset(address: u8, preset_id: u8) -> Result<Message> {
        validate_preset_id(preset_id)?;
        Ok(Message::from_bytes(address, [0x00, 0x03, 0x00, preset_id]))
    }

    /// Clear Preet. An error is returned if `preset_id` is 0.
    pub fn clear_preset(address: u8, preset_id: u8) -> Result<Message> {
        validate_preset_id(preset_id)?;
        Ok(Message::from_bytes(address, [0x00, 0x05, 0x00, preset_id]))
    }

    /// Call Preet. An error is returned if `preset_id` is 0.
    pub fn go_to_preset(address: u8, preset_id: u8) -> Result<Message> {
        validate_preset_id(preset_id)?;
        Ok(Message::from_bytes(address, [0x00, 0x07, 0x00, preset_id]))
    }

    /// Call the special preset "rotate 180 degrees".
    pub fn flip_180(address: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x07, 0x00, 0x21]))
    }

    /// Call the special preset "Go To Zero Pan".
    pub fn go_to_zero_pan(address: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x07, 0x00, 0x22]))
    }

    /// Set Auxiliary. No particular check is done on the arguments.
    pub fn set_auxiliary(address: u8, sub_opcode: u8, aux_id: u8) -> Result<Message> {
        Ok(Message::from_bytes(
            address,
            [sub_opcode, 0x09, 0x00, aux_id],
        ))
    }

    /// Clear Auxiliary. No particular check is done on the arguments.
    pub fn clear_auxiliary(address: u8, sub_opcode: u8, aux_id: u8) -> Result<Message> {
        Ok(Message::from_bytes(
            address,
            [sub_opcode, 0x0B, 0x00, aux_id],
        ))
    }

    /// Reset.
    pub fn remote_reset(address: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x0F, 0x00, 0x00]))
    }

    /// Set Zone Start.
    pub fn set_zone_start(address: u8, zone_id: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x11, 0x00, zone_id]))
    }

    /// Set Zone End.
    pub fn set_zone_end(address: u8, zone_id: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x13, 0x00, zone_id]))
    }

    /// Write Character To Screen.
    pub fn write_char_to_screen(address: u8, column: u8, character: char) -> Result<Message> {
        if character.is_ascii() {
            let ascii = character as u8;
            Ok(Message::from_bytes(address, [0x00, 0x15, column, ascii]))
        } else {
            Err(arg_error("Invalid ASCII character"))
        }
    }

    /// Clear Screen.
    pub fn clear_screen(address: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x17, 0x00, 0x00]))
    }

    /// Alarm Acknowledge.
    pub fn alarm_acknowledge(address: u8, alarm_no: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x19, 0x00, alarm_no]))
    }

    /// Zone Scan On.
    pub fn zone_scan_on(address: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x1B, 0x00, 0x00]))
    }

    /// Zone Scan Off.
    pub fn zone_scan_off(address: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x1D, 0x00, 0x00]))
    }

    /// Record Pattern Start.
    pub fn set_pattern_start(address: u8, pattern_id: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x1F, 0x00, pattern_id]))
    }

    /// Record Pattern End.
    pub fn set_pattern_stop(address: u8, pattern_id: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x21, 0x00, pattern_id]))
    }

    /// Run Pattern.
    pub fn run_pattern(address: u8, pattern_id: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x23, 0x00, pattern_id]))
    }

    /// Set Zoom Speed.
    pub fn set_zoom_speed(address: u8, speed: ZoomSpeed) -> Result<Message> {
        Ok(Message::from_bytes(
            address,
            [0x00, 0x25, 0x00, speed as u8],
        ))
    }

    /// Set Focus Speed.
    pub fn set_focus_speed(address: u8, speed: FocusSpeed) -> Result<Message> {
        Ok(Message::from_bytes(
            address,
            [0x00, 0x27, 0x00, speed as u8],
        ))
    }

    /// Reset Camera to Defaults.
    pub fn reset_camera_to_defaults(address: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x29, 0x00, 0x00]))
    }

    /// Auto Focus.
    pub fn auto_focus(address: u8, ctrl: AutoCtrl) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x2B, 0x00, ctrl as u8]))
    }

    /// Auto Iris.
    pub fn auto_iris(address: u8, cmd: AutoCtrl) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x2D, 0x00, cmd as u8]))
    }

    /// AGC.
    pub fn agc(address: u8, cmd: AutoCtrl) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x2F, 0x00, cmd as u8]))
    }

    /// Backlight Compensation.
    pub fn backlight_compensation(address: u8, ctrl: OnOff) -> Result<Message> {
        let ctrl = match ctrl {
            OnOff::On => 2,
            OnOff::Off => 1,
            OnOff::Value(x) => x,
        };
        Ok(Message::from_bytes(address, [0x00, 0x31, 0x00, ctrl]))
    }

    /// Auto White Balance.
    pub fn auto_white_balance(address: u8, ctrl: OnOff) -> Result<Message> {
        let ctrl = match ctrl {
            OnOff::On => 1,
            OnOff::Off => 2,
            OnOff::Value(x) => x,
        };
        Ok(Message::from_bytes(address, [0x00, 0x33, 0x00, ctrl]))
    }

    /// Enable Device Phase Delay Mode.
    pub fn enable_device_phase_delay_mode(address: u8) -> Result<Message> {
        Ok(Message::from_bytes(address, [0x00, 0x35, 0x00, 0x00]))
    }

    /// Set Shutter Speed.
    pub fn set_shutter_speed(address: u8, ctrl: ShutterSpeed) -> Result<Message> {
        let data: [u8; 2] = match ctrl {
            ShutterSpeed::Bytes(d1, d2) => [d1, d2],
            ShutterSpeed::DefaultValue => [0, 0],
            ShutterSpeed::Increment => [0, 1],
            ShutterSpeed::Decrement => [0, 2],
            ShutterSpeed::PAL => [0, 50],
            ShutterSpeed::NTSC => [0, 60],
            ShutterSpeed::Value(secs) => secs.to_be_bytes(),
            ShutterSpeed::AutoShutter => [0, 0],
            ShutterSpeed::Index(idx) => [0, idx],
        };
        Ok(Message::from_bytes(address, [0x00, 0x37, data[0], data[1]]))
    }

    fn adjust(address: u8, opcode: u8, ctrl: AdjustmentValue) -> Result<Message> {
        let (cmd, data): (u8, [u8; 2]) = match ctrl {
            AdjustmentValue::New(value) => (0, value.to_be_bytes()),
            AdjustmentValue::Delta(value) => (1, value.to_be_bytes()),
        };
        Ok(Message::from_bytes(
            address,
            [cmd, opcode, data[0], data[1]],
        ))
    }

    /// Adjust Line Lock Phase Delay.
    pub fn adjust_line_lock_phase_delay(address: u8, ctrl: AdjustmentValue) -> Result<Message> {
        Message::adjust(address, 0x39, ctrl)
    }

    /// Adjust White Balance (R-B)
    pub fn adjust_white_balance_rb(address: u8, ctrl: AdjustmentValue) -> Result<Message> {
        Message::adjust(address, 0x3B, ctrl)
    }

    /// Adjust White Balance (M-G)
    pub fn adjust_white_balance_mg(address: u8, ctrl: AdjustmentValue) -> Result<Message> {
        Message::adjust(address, 0x3D, ctrl)
    }

    /// Adjust Gain.
    pub fn adjust_gain(address: u8, ctrl: AdjustmentValue) -> Result<Message> {
        Message::adjust(address, 0x3F, ctrl)
    }

    /// Adjust Auto-Iris Level
    pub fn adjust_auto_iris_level(address: u8, ctrl: AdjustmentValue) -> Result<Message> {
        Message::adjust(address, 0x41, ctrl)
    }

    /// Adjust Auto-Iris Peak Value.
    pub fn adjust_auto_iris_peak(address: u8, ctrl: AdjustmentValue) -> Result<Message> {
        Message::adjust(address, 0x43, ctrl)
    }

    /// Query.
    pub fn query() -> Result<Message> {
        Ok(Message::from_bytes(0, [0x00, 0x45, 0x00, 0x00]))
    }
}

impl AsRef<[u8]> for Message {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; MESSAGE_SIZE]> for Message {
    /// Create a Message by specifying all the bytes manually:
    ///
    /// ```rust
    /// # use pelcodrs::Message;
    /// let msg = Message::from([11, 22, 33, 44, 55, 66, 77]);
    /// assert_eq!(&[11, 22, 33, 44, 55, 66, 77], msg.as_ref());
    /// ```
    fn from(array: [u8; MESSAGE_SIZE]) -> Self {
        Message(array)
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

impl TryFrom<&[u8]> for Message {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if value.len() == MESSAGE_SIZE {
            let mut msg = [0u8; MESSAGE_SIZE];
            msg.copy_from_slice(value);
            Ok(Message(msg))
        } else {
            Err("The slice must contain exactly 7 bytes")
        }
    }
}

fn arg_error(description: &str) -> Error {
    Error::new(ErrorKind::InvalidValue, description)
}

fn validate_preset_id(idx: u8) -> Result<()> {
    if idx != 0x00 {
        Ok(())
    } else {
        Err(arg_error("Invalid Present ID"))
    }
}

/// Builder of [Message](struct.Message.html) (standard) instances.
///
/// Provided methods fill the message with corresponding bits, then
/// [MessageBuilder::finalize()](struct.MessageBuilder.html#method.finalize)
/// produces the final Message.
///
/// Please note that currently no logical validation is done. For example the
/// "sense" bit can be overwritten, and thus invalidate previous method call.
///
/// # Example
///
/// ```rust
/// # use pelcodrs::*;
/// # use pelcodrs::*;
/// # fn example() -> Result<()> {
/// let msg = MessageBuilder::new(10)
///     .camera_on()
///     .focus_far()
///     .direction(Direction::DOWN)
///     .tilt(Speed::Range(0.5))
///     .finalize()?;
///
/// assert_eq!(&[0xFF, 0x0A, 0x88, 0x90, 0x00, 0x20, 0x42], msg.as_ref());
/// # Ok(())}
/// # example().expect("Could not finalize message");
///
/// ```
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

/// Checksum algorithm used by Pelco D.
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
