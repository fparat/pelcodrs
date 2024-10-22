# pelcodrs

[![crates.io version badge](https://img.shields.io/crates/v/pelcodrs.svg)](https://crates.io/crates/pelcodrs)
[![Documentation](https://docs.rs/pelcodrs/badge.svg)](https://docs.rs/pelcodrs)
![License](https://img.shields.io/crates/l/pelcodrs)
![CI](https://github.com/fparat/pelcodrs/workflows/Rust/badge.svg)

`pelcodrs` is a Rust library for communications using the Pelco D protocol.

The Pelco D protocol is widely used for controlling PTZ cameras, especially
in the CCTV industry.

## Simple usage

Add this in your application `Cargo.toml`:

```rust
[dependencies]
pelcodrs = "0.2.0"
```

Create message objects to send to the device:

```rust
use pelcodrs::*;

let msg = MessageBuilder::new(10)
    .camera_on()
    .focus_far()
    .down()
    .tilt(Speed::Range(0.5))
    .finalize()?;

assert_eq!(&[0xFF, 0x0A, 0x88, 0x90, 0x00, 0x20, 0x42], msg.as_ref());
```

A port object can be used with any `Read + Write` object for communicating with
the target device. For example, a `SerialPort` object from the crate
[serialport](https://crates.io/crates/serialport) can be used as port:

```rust
use serialport;
use pelcodrs::*;

let dev = PelcoDPort::new(serialport::open("/dev/ttyS0"));
dev.send_message(Message::flip_180(10)?)?;
```


## License

Licensed under either of

* Apache License, Version 2.0
([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.


## Disclaimer

This library is not affiliated or associated in any way with Pelco.

All product and company names are trademarks or registered trademarks of
their respective holders. Use of them does not imply any affiliation with or
endorsement by them.


License: MIT OR Apache-2.0
