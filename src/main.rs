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
    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;

    let mut packet = Packet::new();
    packet.header.id = 1234;
    packet.header.queries_total = 1;
    packet.header.recursion_desired = true;
    packet.queries.push(Query::new(qname.to_string(), qtype));

    let mut req_buffer = PacketBuffer::new();
    packet.write_to_buffer(&mut req_buffer)?;
    socket.send_to(req_buffer.get_range(0, req_buffer.pos())?, server)?;

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

        if let Some(new_ns) = response.get_ns_from_additional_records(qname) {
            ns = new_ns;
            continue;
        }

        // Resolve IP of an NS record
        // TODO: is this broken? Try to comment the previous part and see if this still works
        let new_ns_host = match response.get_some_ns_host(qname) {
            Some(x) => x,
            None => return Ok(response),
        };

        // TODO: resolve queries other than A?
        let recursive_response = recursive_lookup(&new_ns_host, QueryType::A)?;

        // Finally, we pick a random ip from the result, and restart the loop. If no such
        // record is available, we again return the last result we got.
        if let Some(new_ns) = recursive_response.get_first_a_record() {
            ns = new_ns;
        } else {
            return Ok(response);
        }
    }
}

fn handle_query(socket: &UdpSocket) -> Result<()> {
    let mut raw_request_buffer: [u8; 512] = [0; 512];
    let (_, src) = socket.recv_from(&mut raw_request_buffer)?;
    let mut request_buffer = PacketBuffer::from_u8_array(raw_request_buffer);
    let request = Packet::from_buffer(&mut request_buffer)?;

    let mut reply_packet = Packet::new();
    reply_packet.header.id = request.header.id;
    reply_packet.header.recursion_desired = true;
    reply_packet.header.recursion_available = true;
    reply_packet.header.response = true;

    if request.queries.is_empty() {
        reply_packet.header.return_code = ReturnCode::FORMERR;
    }

    for query in request.queries.iter() {
        println!("Received query: {:?}", query);

        if let Ok(result) = recursive_lookup(&query.qname, query.qtype) {
            reply_packet.queries.push(query.clone());
            reply_packet.header.return_code = result.header.return_code;

            for rec in result.answer_records {
                println!("Answer: {:?}", rec);
                reply_packet.answer_records.push(rec);
            }
            for rec in result.authoritative_records {
                println!("Authority: {:?}", rec);
                reply_packet.authoritative_records.push(rec);
            }
            for rec in result.additional_records {
                println!("Resource: {:?}", rec);
                reply_packet.additional_records.push(rec);
            }
        } else {
            reply_packet.header.return_code = ReturnCode::SERVFAIL;
        }
    }

    let mut reply_buffer = PacketBuffer::new();
    reply_packet.write_to_buffer(&mut reply_buffer)?;

    let len = reply_buffer.pos();
    let data = reply_buffer.get_range(0, len)?;
    socket.send_to(data, src)?;

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
