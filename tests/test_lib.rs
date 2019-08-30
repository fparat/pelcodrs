use pelcodrs::message::{Command1, Command2, Direction, Message, MessageBuilder, Speed};

#[test]
fn test_message_new() {
    let msg = Message::new(
        10,
        Command1::SENSE | Command1::CAMERA_ON_OFF,
        Command2::FOCUS_FAR | Command2::DOWN,
        0x00,
        0x40,
    );
    assert_eq!(&[0xFF, 0x0A, 0x88, 0x90, 0x00, 0x40, 0x62], msg.as_ref());
}

#[test]
fn test_message_build() {
    let msg = MessageBuilder::new(1).camera_on().finalize().unwrap();
    assert_eq!(&[0xFF, 0x01, 0x88, 0x00, 0x00, 0x00, 0x89], msg.as_ref());

    let msg = MessageBuilder::new(1).camera_off().finalize().unwrap();
    assert_eq!(&[0xFF, 0x01, 0x08, 0x00, 0x00, 0x00, 0x09], msg.as_ref());

    let msg = MessageBuilder::new(2)
        .left()
        .pan(Speed::Range(0.5))
        .finalize()
        .unwrap();
    assert_eq!(&[0xFF, 0x02, 0x00, 0x04, 0x20, 0x00, 0x26], msg.as_ref());

    let msg = MessageBuilder::new(2).stop().finalize().unwrap();
    assert_eq!(&[0xFF, 0x02, 0x00, 0x00, 0x00, 0x00, 0x02], msg.as_ref());

    let msg = MessageBuilder::new(10)
        .camera_on()
        .focus_far()
        .direction(Direction::DOWN)
        .tilt(Speed::Turbo)
        .finalize()
        .expect("Could not finalize message");
    assert_eq!(&[0xFF, 0x0A, 0x88, 0x90, 0x00, 0xFF, 0x21], msg.as_ref());
}
