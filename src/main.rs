use std::error::Error;
use std::net::UdpSocket;

mod dns;

use dns::byte_packet_buffer::BytePacketBuffer;
use dns::dns_packet::DnsPacket;
use dns::question::Question;
use dns::question_type::QuestionType;
use dns::return_code::ReturnCode;

fn lookup(qname: &str, qtype: QuestionType) -> Result<DnsPacket, Box<dyn Error>> {
    // Forward queries to Google's public DNS
    let server = ("8.8.8.8", 53);
    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;

    let mut packet = DnsPacket::new();

    packet.header.id = 1234;
    packet.header.questions_total = 1;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(Question::new(qname.to_string(), qtype));

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer)?;
    socket.send_to(&req_buffer.buf[0..req_buffer.pos], server)?;

    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buf)?;

    DnsPacket::from_buffer(&mut res_buffer)
}

fn handle_query(socket: &UdpSocket) -> Result<(), Box<dyn Error>> {
    // Blocks until a reply is received
    let mut req_buffer = BytePacketBuffer::new();

    let (_, src) = socket.recv_from(&mut req_buffer.buf)?;
    let mut request = DnsPacket::from_buffer(&mut req_buffer)?;

    let mut packet = DnsPacket::new();
    packet.header.id = request.header.id;
    packet.header.recursion_desired = true;
    packet.header.recursion_available = true;
    packet.header.response = true;

    // Only get one question TODO: support more
    if let Some(question) = request.questions.pop() {
        println!("Received query: {:?}", question);

        if let Ok(result) = lookup(&question.name, question.qtype) {
            packet.questions.push(question);
            packet.header.return_code = result.header.return_code;

            for rec in result.answer_records {
                println!("Answer: {:?}", rec);
                packet.answer_records.push(rec);
            }
            for rec in result.authoritative_records {
                println!("Authority: {:?}", rec);
                packet.authoritative_records.push(rec);
            }
            for rec in result.additional_records {
                println!("Resource: {:?}", rec);
                packet.additional_records.push(rec);
            }
        } else {
            packet.header.return_code = ReturnCode::SERVFAIL;
        }
    } else {
        packet.header.return_code = ReturnCode::FORMERR;
    }

    let mut res_buffer = BytePacketBuffer::new();
    packet.write(&mut res_buffer)?;

    // TODO: maybe we can add a "get_size" function?
    let len = res_buffer.pos();
    let data = res_buffer.get_range(0, len)?;

    socket.send_to(data, src)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind(("0.0.0.0", 2053))?;

    // TODO: don't make an infinite loop
    loop {
        match handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("An error occurred: {}", e),
        }
    }
}
