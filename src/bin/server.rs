use iris::resolver::Resolver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = 2053;
    let resolver = Resolver::new("127.0.0.1", port)?;

    println!("=== DNS server listening on port {} ===\n", port);

    loop {
        match resolver.handle_query() {
            Ok(_) => {}
            Err(e) => eprintln!("An error occurred: {}", e),
        }
    }
}
