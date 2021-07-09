# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- example for Source engine games (tested against Counter Strike: Global Offensive). [@jenrik](https://github.com/jenrik)

## [0.5.0] - 2021-07-10

### Added
- support for running on an `async-std` executer. It should now be possible to use `rcon` with both, `tokio` and `async-std`. [@jenrik](https://github.com/jenrik)

## [0.4.0] - 2020-12-26

### Breaking
- upgraded to tokio 1.0
