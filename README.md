# unix\_mode Rust library

[![crates.io](https://img.shields.io/crates/v/unix_mode.svg)](https://crates.io/crates/unix_mode)
[![docs.rs](https://docs.rs/unix_mode/badge.svg)](https://docs.rs/unix_mode)

This library provides functions to decode and print Unix mode bits /
permissions, even on non-Unix platforms.

On Unix, decoding is supported by `std::os::unix::fs` in the standard library,
but this crate adds a function to print them in the format used by `ls -l`.

## License

Apache-2.0.

## Contributing

Patches are very welcome.

Please read the
[contribution guidelines](CONTRIBUTING.md) and
[code of conduct](CODE_OF_CONDUCT.md).

## Disclaimer

This is not an official Google project. It is not supported by Google, and
Google specifically disclaims all warranties as to its quality, merchantability,
or fitness for a particular purpose.
