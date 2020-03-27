rust-rcon [![Build Status](https://travis-ci.org/panicbit/rust-rcon.svg?branch=master)](https://travis-ci.org/panicbit/rust-rcon)
=========

An RCON implementation in the Rust programming language.

This project aims to at least work with the Minecraft implementation of RCON.

## Status
- basic rcon sessions work
- multi-packet responses are not implemented

## How to install

Add this your Cargo.toml:
```toml
[dependencies]
rcon = "0"
```


## How to use
```rust
extern crate rcon;
```


## Examples

See the examples in [the examples folder](https://github.com/panicbit/rust-rcon/tree/master/examples)

## Features

 * `delay` Adds a 3 millisecond delay to each command for compatiblity with the offical Minecraft server implmentation

The `delay` feature is enabled by default.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

