use std::io::{Read, Write};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use pelcodrs::message::Message;
use pelcodrs::port::*;
use std::convert::TryFrom;

struct FakeDevice {
    tx: Sender<u8>,
    rx: Receiver<u8>,
}

impl FakeDevice {
    pub fn from_channels(tx: Sender<u8>, rx: Receiver<u8>) -> FakeDevice {
        FakeDevice { tx, rx }
    }

    pub fn received(&self) -> Vec<u8> {
        self.rx.try_iter().collect()
    }
}

impl Read for FakeDevice {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        let mut count = 0;
        for (b, rx) in buf.iter_mut().zip(self.rx.try_iter()) {
            *b = rx;
            count += 1;
        }
        Ok(count)
    }
}

impl Write for FakeDevice {
    fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
        Ok(buf.iter().filter_map(|b| self.tx.send(*b).ok()).count())
    }

    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        Ok(())
    }
}

type StubPort = FakeDevice;
type StubDevice = FakeDevice;

fn new_stub_port_and_device() -> (PelcoDPort<StubPort>, StubDevice) {
    let (mosi_tx, mosi_rx) = mpsc::channel();
    let (miso_tx, miso_rx) = mpsc::channel();
    let stubport = StubPort::from_channels(mosi_tx, miso_rx);
    let stubdev = StubDevice::from_channels(miso_tx, mosi_rx);
    (PelcoDPort::new(stubport), stubdev)
}

#[test]
fn test_create_port_and_stub_and_write_and_read() {
    let (mut pelcoport, mut stubdev) = new_stub_port_and_device();

    pelcoport.write_all(b"hello").unwrap();
    assert_eq!(b"hello".to_vec(), stubdev.received());

    let mut buf = [0u8; 32];
    stubdev
        .write_all(b"bye")
        .expect("Error when sending data through stub");
    assert_eq!(3, pelcoport.read(&mut buf).unwrap());
    assert_eq!(b"bye", &buf[..3]);
}

#[test]
fn test_send_message_with_port() {
    let (mut pelcoport, stubdev) = new_stub_port_and_device();

    let msg = Message::from([1, 2, 3, 4, 5, 6, 7]);
    pelcoport.send_message(msg).expect("Failed sending message");
    let received = stubdev.received();
    assert_eq!(msg, Message::try_from(&received[..]).unwrap());
}
