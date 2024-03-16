# caddy-pretty-print

[![Latest Version][crates-badge]][crates.io]
![Apache 2.0 OR MIT licensed][license-badge]

[crates.io]: https://crates.io/crates/caddy-pretty-print
[crates-badge]: https://img.shields.io/crates/v/caddy-pretty-print.svg
[license-badge]: https://img.shields.io/badge/license-Apache2.0%2FMIT-blue.svg

A CLI tool to pretty print [Caddy](https://github.com/caddyserver/caddy) json logs.

## Install

If cargo is installed, caddy-pretty-print can be installed with it:

```
$ cargo install caddy-pretty-print
```

Alternatively, you can download a pre-built binary for your operating system
from the [latest release](https://github.com/mattiapenati/caddy-pretty-print/releases).

## How to use

You can pipe the log directly into it, from a file:

```bash
cat caddy.log | caddy-pretty-print
```

Or from the output of any other job

```bash
sudo journalctl -u caddy.service --output cat - | caddy-pretty-print
```


## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT
license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

