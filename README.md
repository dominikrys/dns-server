# Iris

[![Build Status](https://img.shields.io/github/workflow/status/dominikrys/iris/Continuous%20Integration?style=flat-square)](https://github.com/dominikrys/iris/actions)

Rudimentary DNS server implementation in Rust with support for the most common DNS record types. Written with the help of [Emil Hernvall's DNS guide](https://github.com/EmilHernvall/dnsguide). Non-exhaustive tests are also provided.

The aim of this project was to learn about Rust and DNS protocols.

## Demo

[![asciicast](https://asciinema.org/a/422536.svg)](https://asciinema.org/a/422536)

## Build Instructions

[Rust](https://www.rust-lang.org/) (stable or nightly) needs to be installed. Then, run:

```bash
cargo build
```

For automated pre-commit/pre-push checks, a [Lefthook](https://github.com/evilmartians/lefthook) script is included. Run `lefthook install` to initialize it.

## How to Run

Run the server:

```bash
cargo run
```

Then, while the server is running, send a DNS query to port 2053:

```bash
dig @127.0.0.1 -p 2053 www.google.com
```

To run the provided tests:

```bash
cargo test
```

## Links

- [DNS packet structure reference](http://www.networksorcery.com/enp/protocol/dns.htm)
- [Domain name compression explanation](https://docstore.mik.ua/orelly/networking_2ndEd/dns/ch15_02.htm)

## Further Improvements

I've achieved what I wanted to with this project, but there are some further improvements that could be made:

- Support replies larger than 512B (TCP connection support is necessary for this)
- Concurrency
- Start resolving using other servers than the A root server
- Add support for hosting zones
- Allow acting as an authoritative server
- DNSSEC
- Add support for more records (SOA, TXT, SRV, OPT)
