# ESP32C3 Captive Portal Example

## Flashing firmware

* [Install Rust](https://rustup.rs)
* [Setup ESP32 RISC-V target](https://esp-rs.github.io/book/installation/riscv.html)
* Install espflash: `cargo install espflash`
* Flash firmware: `cargo run --release`

## Wi-Fi

Default SSID name and password:
* SSID: `ESP32Portal`
* Password: `PortalPassword`

Compile firmware with custom Wi-Fi config:
* `SSID="name" SSID_PASSWORD="password" cargo build --release`

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
