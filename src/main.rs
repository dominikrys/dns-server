use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::net::UdpSocket;

mod dns;

use dns::byte_packet_buffer::BytePacketBuffer;
use dns::dns_packet::DnsPacket;
use dns::question::Question;
use dns::question_type::QuestionType;
fn main() -> Result<(), Box<dyn Error>> {
    // Connect to Google's DNS
    let server = ("8.8.8.8", 53);
    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;

    // Query A record for google.com
    let qname = "google.com";
    let qtype = QuestionType::A;

    // Configure request packet
    let mut packet = DnsPacket::new();

    packet.header.id = 1234;
    packet.header.questions_total = 1;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(Question::new(qname.to_string(), qtype));

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer)?;

    // Perform the query
    socket.send_to(&req_buffer.buf[0..req_buffer.pos], server)?;

    // Get the answer
    let mut answer_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut answer_buffer.buf)?;

    let answer_packet = DnsPacket::from_buffer(&mut answer_buffer)?;
    println!("{:#?}", answer_packet.header);

    for q in answer_packet.questions {
        println!("{:#?}", q);
    }
    for rec in answer_packet.answer_records {
        println!("{:#?}", rec);
    }
    for rec in answer_packet.authoritative_records {
        println!("{:#?}", rec);
    }
    for rec in answer_packet.additional_records {
        println!("{:#?}", rec);
    }

    Ok(())
}
