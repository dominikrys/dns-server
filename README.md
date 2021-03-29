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

qname qtype

- re-read all code and go through TODOs
- make into lib and not bin?
- refactor + naming
  - no side effects
  - unambiguous and consistent names e.g. domain and host
  - add comments if really necessary
  - remove dns\_\* from files. We know it's DNS.
  - utilise the result type for returning values and errors
  - sort out directory structure
    - are all the includes pragmatic? can we reuse `type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;`? Or is this not good practise at all?
    - sort out scope - go through the "make this private" todos for these especially
    - should we include more stuff in one file?
    - alternatively, is there a way to reduce the amount of `use super::*`?
    - maybe use [`pub use`](https://www.reddit.com/r/rust/comments/6x49mu/what_are_some_rules_of_thumb_for_use/dmd07yr?utm_source=share&utm_medium=web2x&context=3)?
  - are the file names correct?
- add tests?
  - maybe just important ones if necessary
- Extra nice stuff:
  - Concurrency
  - TCP connections
  - Host our own zones? Allow to act a authoritative server?
  - DNSSEC?
  - Add SOA records?
- make public, put featured on profile, make sure badge shows up
