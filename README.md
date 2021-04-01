# DNS Server

[![Build Status](https://img.shields.io/github/workflow/status/dominikrys/dns-server/ci?style=flat-square)](https://github.com/dominikrys/dns-server/actions)

Rudimentary DNS server in Rust, with support for the most common record types. Loosely based off [Emil Hernvall's DNS guide](https://github.com/EmilHernvall/dnsguide).

The aim of this project was to learn about Rust and DNS protocols.

## Build Instructions

Rust (stable or nightly) needs to be installed. Then run:

```bash
cargo build
```

## Run Instructions

Run the server:

```bash
cargo run
```

Send DNS query to port 2053:

```bash
dig @127.0.0.1 -p 2053 www.google.com
```

## Links

- [DNS packet structure reference](http://www.networksorcery.com/enp/protocol/dns.htm)
- [Domain name compression explanation](https://docstore.mik.ua/orelly/networking_2ndEd/dns/ch15_02.htm)

## TODO

- refactor code
  - should we include more stuff in one file? protocol, buffer, server
  - alternatively, is there a way to reduce the amount of `use super::*`?
  - maybe use [`pub use`](https://www.reddit.com/r/rust/comments/6x49mu/what_are_some_rules_of_thumb_for_use/dmd07yr?utm_source=share&utm_medium=web2x&context=3)?
  - are the file names correct?
- add important tests
- Make `write_to_buffer` and `from_buffer` methods not rely on the current buffer position
- Extra functionality:
  - Concurrency
  - TCP connections
  - Support replies larger than 512B
  - Resolve using other servers than a root servers
  - Host our own zones? Allow to act a authoritative server?
  - DNSSEC?
  - Add SOA records?
- make public, put featured on profile, make sure badge shows up
