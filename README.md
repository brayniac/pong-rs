# pong-rs - a UDP ASCII ping server with userspace networking

pong-rs is a UDP ASCII ping server which makes use of [librips](https://github.com/faern/librips) for userspace networking. It provides a basic server for demonstration and benchmarking.

## Usage

To use `pong-rs`, first clone the repo:

With stable rust, just build and run (note: you must change the parameters to reflect your environment):
```shell
git clone https://github.com/brayniac/pong-rs
cargo build --release
sudo ./target/release/pong-rs --ip 10.138.0.3/32 --gateway 10.138.0.1 eth0
```

Optimized event timestamping. WARNING: this adds platform specific optimizations!!! Only tested on Intel x86_64. This assumes the constant_tsc feature is available on your particular CPU. Beware of potential for incorrect metrics if the assumptions do not hold.
```shell
git clone https://github.com/brayniac/pong-rs
cargo build --release --features asm
sudo ./target/release/pong-rs --ip 10.138.0.3/32 --gateway 10.138.0.1 eth0
```

Once launched, you may use [ping-rs](https://github.com/brayniac/ping-rs) for benchmarking the server

## Features

* over-engineered ASCII ping server
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
