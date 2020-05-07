# unix\_mode Rust library

This library provides functions to decode and print Unix mode bits /
permissions, even on non-Unix platforms.

On Unix, decoding is supported by `std::os::unix::fs` in the standard library,
but this crate adds a function to print them in the format used by `ls -l`.

## License

Apache-2.0.

## Contributing

I'd love to accept patches to this project. Please read the
[contribution guidelines](CONTRIBUTING.md) and
[code of conduct](CODE_OF_CONDUCT.md).

## Disclaimer

This is not an official Google project. It is not supported by Google, and
Google specifically disclaims all warranties as to its quality, merchantability,
or fitness for a particular purpose.
