use std::error::Error;
use std::fs::File;
use std::io::Read;

mod dns;

use dns::byte_packet_buffer::BytePacketBuffer;
use dns::dns_packet::DnsPacket;

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("response_packet.txt")?;
    let mut buffer = BytePacketBuffer::new();
    f.read(&mut buffer.buf)?;

    let packet = DnsPacket::from_buffer(&mut buffer)?;
    println!("{:#?}", packet.header);

    for q in packet.questions {
        println!("{:#?}", q);
    }
    for rec in packet.answer_records {
        println!("{:#?}", rec);
    }
    for rec in packet.authoritative_records {
        println!("{:#?}", rec);
    }
    for rec in packet.additional_records {
        println!("{:#?}", rec);
    }

    Ok(())
}
