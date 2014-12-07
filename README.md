rust-rcon [![Build Status](https://travis-ci.org/panicbit/rust-rcon.svg?branch=master)](https://travis-ci.org/panicbit/rust-rcon)
=========

An RCON implementation in the Rust programming language.

This project aims to at least work with the Minecraft implementation of RCON.

### Status
- basic rcon sessions work
- multi-packet responses are not implemented
- the maximum request size of the Minecraft implementation is not respected

### How to install

Add this your Cargo.toml:
```toml
[dependencies.rcon]
git = "https://github.com/panicbit/rust-rcon.git"
```


### How to use
```rust
extern crate rcon;
```


### Examples

See the examples in [the examples folder](https://github.com/panicbit/rust-rcon/tree/master/examples)


### Contributing
Contributions are welcome!
