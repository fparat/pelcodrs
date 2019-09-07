use pelcodrs::*;
use std::convert::TryFrom;

#[test]
fn test_message_new() {
    let msg = Message::new(
        10,
        Command1::SENSE | Command1::CAMERA_ON_OFF,
        Command2::FOCUS_FAR | Command2::DOWN,
        0x00,
        0x20,
    );
    assert_eq!(&[0xFF, 0x0A, 0x88, 0x90, 0x00, 0x20, 0x42], msg.as_ref());

    let msg2 = MessageBuilder::new(10)
        .camera_on()
        .focus_far()
        .down()
        .tilt(Speed::Range(0.5))
        .finalize()
        .unwrap();

    assert_eq!(msg, msg2);
}

#[test]
fn test_new_message_with_try_from() {
    let bytes = [11, 22, 33, 44, 55, 66, 77];
    let msg = Message::try_from(&bytes[..]).expect("Failed creating Message from bytes slice");
    assert_eq!(&bytes, msg.as_ref());

    let few_bytes = [11, 22, 33, 44, 55, 66];
    let _ = Message::try_from(&few_bytes[..])
        .expect_err("Message instantiation with too few bytes should fail");

    let many_bytes = [11, 22, 33, 44, 55, 66, 77, 88];
    let _ = Message::try_from(&many_bytes[..])
        .expect_err("Message instantiation with too many bytes should fail");
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

#[test]
fn test_message_from_words() {
    let msg = Message::from_bytes(23, [200, 9, 145, 17]);
    assert_eq!([0xFF, 23, 200, 9, 145, 17, 138], msg.as_ref());
}

#[test]
fn test_invalid_preset() {
    let _ = Message::set_preset(12, 0).expect_err("Preset 0 should fail");
    let _ = Message::clear_preset(12, 0).expect_err("Preset 0 should fail");
    let _ = Message::go_to_preset(12, 0).expect_err("Preset 0 should fail");
}

#[test]
fn test_preset() {
    let msg = Message::set_preset(1, 1).unwrap();
    assert_eq!(&[0xFF, 1, 0, 0x03, 0, 1, 5], msg.as_ref());

    let msg = Message::clear_preset(12, 15).unwrap();
    assert_eq!(&[0xFF, 12, 0, 0x05, 0, 15, 32], msg.as_ref());

    let msg = Message::go_to_preset(255, 255).unwrap();
    assert_eq!(&[0xFF, 255, 0, 0x07, 0, 255, 5], msg.as_ref());

    assert_eq!(
        Message::go_to_preset(34, 0x21).unwrap(),
        Message::flip_180(34).unwrap()
    );
    assert_eq!(
        Message::go_to_preset(84, 0x22).unwrap(),
        Message::go_to_zero_pan(84).unwrap()
    );
}

#[test]
fn test_write_char_to_screen() {
    let msg = Message::write_char_to_screen(11, 32, 'F').unwrap();
    assert_eq!(&[0xFF, 11, 0, 0x15, 32, b'F', 134], msg.as_ref());

    let _ = Message::write_char_to_screen(11, 32, '好').expect_err("Should fail for non-ASCII");
}

#[test]
fn test_zoom_focus_speed() {
    let msg = Message::set_zoom_speed(12, ZoomSpeed::Medium).unwrap();
    assert_eq!(&[0xFF, 12, 0, 0x25, 0, 1, 50], msg.as_ref());

    let msg = Message::set_focus_speed(12, FocusSpeed::Highest).unwrap();
    assert_eq!(&[0xFF, 12, 0, 0x27, 0, 3, 54], msg.as_ref());
}
