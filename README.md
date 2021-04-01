# DNS Server

[![Build Status](https://img.shields.io/github/workflow/status/dominikrys/dns-server/Continuous%20Integration?style=flat-square)](https://github.com/dominikrys/dns-server/actions)

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

- Complete tests (check `TODO`s in code)
- Make `write_to_buffer` and `from_buffer` methods not rely on the current buffer position
- Support replies larger than 512B (TCP connections)
- Concurrency
- Start resolving using other servers than the A root server
- Add support for hosting zones
- Allow acting as an authoritative server
- DNSSEC
- Add support for more records (SOA, TXT, SRV, OPT)
