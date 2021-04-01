use std::net::UdpSocket;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = 2053;
    let socket = UdpSocket::bind(("127.0.0.1", port))?;

    println!("=== DNS server listening on port {} ===\n", port);

    loop {
        match dns_server::client::handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("An error occurred: {}", e),
        }
    }
}
