# pong-rs - a UDP ASCII ping server with userspace networking

pong-rs is a UDP ASCII ping server which makes use of [librips](httos://github.com/faern/librips) for userspace networking. It provides a basic server for demonstration and benchmarking.

## Usage

To use `pong-rs`, first clone the repo:

With stable rust, just build and run (note: you must change the parameters to reflect your environment):
```shell
git clone https://github.com/brayniac/pong-rs
cargo build --release
sudo ./target/release/pong-rs --ip 10.138.0.3/32 --gateway 10.138.0.1 eth0
```

With nightly rust, you may use the 'asm' feature to provide lower-cost timestamping of events:
```shell
git clone https://github.com/brayniac/pong-rs
cargo build --release --features asm
sudo ./target/release/pong-rs --ip 10.138.0.3/32 --gateway 10.138.0.1 eth0
```

Once launched, you may use [ping-rs](https://github.com/brayniac/ping-rs) for benchmarking the server

## Features

* simple ASCII ping server
* userspace UDP implementation

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
