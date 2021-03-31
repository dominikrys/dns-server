#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate num_derive;

use std::net::Ipv4Addr;
use std::net::UdpSocket;

mod dns;

use dns::packet::Packet;
use dns::packet_buffer::PacketBuffer;
use dns::query::Query;
use dns::query_type::QueryType;
use dns::return_code::ReturnCode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: remove this stuff from main?
fn lookup(qname: &str, qtype: QueryType, server: (Ipv4Addr, u16)) -> Result<Packet> {
    // Send request to server TODO: refactor this into its own method as it's shared
    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;

    let mut packet = Packet::new();
    packet.header.id = 1234;
    packet.header.queries_total = 1;
    packet.header.recursion_desired = true;
    packet.queries.push(Query::new(qname.to_string(), qtype));

    let mut req_buffer = PacketBuffer::new();
    packet.write_to_buffer(&mut req_buffer)?;
    socket.send_to(req_buffer.get_range(0, req_buffer.pos())?, server)?;

    // Get reply from server TODO: refactor this into its own method as it's shared
    let mut raw_res_buffer: [u8; 512] = [0; 512];
    socket.recv_from(&mut raw_res_buffer)?;
    let mut res_buffer = PacketBuffer::from_u8_array(raw_res_buffer);

    Packet::from_buffer(&mut res_buffer)
}

fn recursive_lookup(qname: &str, qtype: QueryType) -> Result<Packet> {
    let a_root_servers_net_ip = "198.41.0.4";
    let mut ns = a_root_servers_net_ip.parse::<Ipv4Addr>().unwrap();

    loop {
        println!("Performing lookup of {:?} {} with ns {}", qtype, qname, ns);

        let server = (ns, 53);
        let response = lookup(qname, qtype, server)?;

        // TODO: can we combine these into a match?
        if !response.answer_records.is_empty() && response.header.return_code == ReturnCode::NOERROR
        {
            return Ok(response);
        }

        if response.header.return_code == ReturnCode::NXDOMAIN {
            return Ok(response);
        }

        // Try to resolve NS from additional records
        if let Some(new_ns) = response.get_ns_from_additional_records(qname) {
            ns = new_ns;
            continue;
        }

        // Resolve NS ourselves
        // TODO: is this broken? Try to comment the previous part and see if this still works
        // TODO: make the "get_first" methods just reply with the full list
        // TOOD: remove comments from these
        let new_ns_host = match response.get_first_ns_host(qname) {
            Some(x) => x,
            None => return Ok(response),
        };
        let recursive_response = recursive_lookup(&new_ns_host, qtype)?;

        // Pick an IP from the result, and recurse. Otherwise return last result.
        if let Some(new_ns) = recursive_response.get_first_a_record() {
            ns = new_ns;
        } else {
            return Ok(response);
        }
    }
}

fn handle_query(socket: &UdpSocket) -> Result<()> {
    // Get request TODO: refactor this into its own method as it's shared
    let mut raw_req_buffer: [u8; 512] = [0; 512];
    let (_, req_src_ip) = socket.recv_from(&mut raw_req_buffer)?;
    let mut req_buffer = PacketBuffer::from_u8_array(raw_req_buffer);
    let request = Packet::from_buffer(&mut req_buffer)?;

    // Prepare reply
    let mut res_packet = Packet::new();
    res_packet.header.id = request.header.id;
    res_packet.header.recursion_desired = true;
    res_packet.header.recursion_available = true;
    res_packet.header.response = true;

    // Resolve request
    if request.queries.is_empty() {
        res_packet.header.return_code = ReturnCode::FORMERR;
    }

    for query in request.queries.iter() {
        println!("Received query: {:?}", query);

        if let Ok(result) = recursive_lookup(&query.qname, query.qtype) {
            res_packet.queries.push(query.clone());
            res_packet.header.return_code = result.header.return_code;

            for rec in result.answer_records {
                println!("Answer rec: {:?}", rec);
                res_packet.answer_records.push(rec);
            }
            for rec in result.authoritative_records {
                println!("Authority rec: {:?}", rec);
                res_packet.authoritative_records.push(rec);
            }
            for rec in result.additional_records {
                println!("Additional rec: {:?}", rec);
                res_packet.additional_records.push(rec);
            }
        } else {
            res_packet.header.return_code = ReturnCode::SERVFAIL;
        }
    }

    // Send reply TODO: refactor this into its own method as it's shared
    let mut res_buffer = PacketBuffer::new();
    res_packet.write_to_buffer(&mut res_buffer)?;

    let res_len = res_buffer.pos();
    let res_data = res_buffer.get_range(0, res_len)?;
    socket.send_to(res_data, req_src_ip)?;

    Ok(())
}

fn main() -> Result<()> {
    let port = 2053;
    let socket = UdpSocket::bind(("127.0.0.1", port))?;

    println!("=== DNS server listening on port {} ===\n", port);

    loop {
        match handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("An error occurred: {}", e),
        }
    }
}
