# DNS Server

[![Build Status](https://img.shields.io/github/workflow/status/dominikrys/dns-server/ci?style=flat-square)](https://github.com/dominikrys/dns-server/actions)

Rudimentary DNS server in Rust. Based off [Emil Hernvall's DNS guide](https://github.com/EmilHernvall/dnsguide).

## Build Instructions

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

## TODO

- refactor code
  - sort out directory structure
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
