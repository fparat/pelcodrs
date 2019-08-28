use pelcodrs::{Command1, Command2, Message};

#[test]
fn test_message_build() {
    let msg = Message::new(10)
        .with_command1(Command1::SENSE | Command1::CAMERA_ON_OFF)
        .with_command2(Command2::FOCUS_FAR | Command2::DOWN)
        .with_data1(0x00)
        .with_data2(0x40)
        .finalize()
        .unwrap();
    assert_eq!(&[0xFF, 0x0A, 0x88, 0x90, 0x00, 0x40, 0x62], msg.as_ref());
}
