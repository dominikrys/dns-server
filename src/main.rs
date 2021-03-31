#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate num_derive;

use std::net::UdpSocket;

mod dns;

use dns::resolve;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let port = 2053;
    let socket = UdpSocket::bind(("127.0.0.1", port))?;

    println!("=== DNS server listening on port {} ===\n", port);

    loop {
        match resolve::handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("An error occurred: {}", e),
        }
    }
}
