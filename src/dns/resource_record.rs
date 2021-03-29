use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use super::byte_packet_buffer::BytePacketBuffer;
use super::question_type::QuestionType;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: see if all these traits are needed
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceRecord {
    // TODO: do we need an unknown record?
    // TODO: can we include the integer within these?
    // TODO: do we need a new enum for every record type?
    // TODO: maybe implement this as a trait? Something to store the common names
    // These are pretty generic and the header is the same for all: http://www.networksorcery.com/enp/protocol/dns.htm#Answer%20RRs
    UNKNOWN {
        domain: String,
        qtype: u16,
        data_len: u16,
        ttl: u32,
    }, // 0
    A {
        domain: String,
        ip_addr: Ipv4Addr,
        ttl: u32,
    }, // 1
    NS {
        domain: String,
        host: String,
        ttl: u32,
    }, // 2,
    CNAME {
        domain: String,
        cname: String,
        ttl: u32,
    }, // 5
    MX {
        domain: String,
        priority: u16,
        exchange: String,
        ttl: u32,
    }, // 15
    AAAA {
        domain: String,
        ip_addr: Ipv6Addr,
        ttl: u32,
    }, // 28
}

impl ResourceRecord {
    pub fn read(buffer: &mut BytePacketBuffer) -> Result<ResourceRecord> {
        // TODO: this assumes that the bytepacketbuffer is at the start. Maybe reset it?
        let mut domain = String::new();
        buffer.read_qname(&mut domain)?;

        let qtype_num = buffer.read_u16()?;
        let qtype = QuestionType::from_num(qtype_num); // TODO: maybe combine these two in one
        let _ = buffer.read_u16()?; // Class
        let ttl = buffer.read_u32()?;
        let data_len = buffer.read_u16()?;

        // TODO: use the data_len in here somehow, maybe check for limit
        match qtype {
            QuestionType::A => {
                let raw_ip_addr = buffer.read_u32()?;
                let ip_addr = Ipv4Addr::new(
                    ((raw_ip_addr >> 24) & 0xFF) as u8,
                    ((raw_ip_addr >> 16) & 0xFF) as u8,
                    ((raw_ip_addr >> 8) & 0xFF) as u8,
                    ((raw_ip_addr >> 0) & 0xFF) as u8,
                );

                // TODO: can we not repeat this
                Ok(ResourceRecord::A {
                    domain,
                    ip_addr,
                    ttl,
                })
            }
            QuestionType::AAAA => {
                let raw_ip_addr1 = buffer.read_u32()?;
                let raw_ip_addr2 = buffer.read_u32()?;
                let raw_ip_addr3 = buffer.read_u32()?;
                let raw_ip_addr4 = buffer.read_u32()?;

                let ip_addr = Ipv6Addr::new(
                    ((raw_ip_addr1 >> 16) & 0xFFFF) as u16,
                    ((raw_ip_addr1 >> 0) & 0xFFFF) as u16,
                    ((raw_ip_addr2 >> 16) & 0xFFFF) as u16,
                    ((raw_ip_addr2 >> 0) & 0xFFFF) as u16,
                    ((raw_ip_addr3 >> 16) & 0xFFFF) as u16,
                    ((raw_ip_addr3 >> 0) & 0xFFFF) as u16,
                    ((raw_ip_addr4 >> 16) & 0xFFFF) as u16,
                    ((raw_ip_addr4 >> 0) & 0xFFFF) as u16,
                );

                Ok(ResourceRecord::AAAA {
                    domain,
                    ip_addr,
                    ttl,
                })
            }
            QuestionType::NS => {
                let mut host = String::new();
                buffer.read_qname(&mut host)?;

                Ok(ResourceRecord::NS { domain, host, ttl })
            }
            QuestionType::CNAME => {
                let mut cname = String::new();
                buffer.read_qname(&mut cname)?;

                Ok(ResourceRecord::CNAME { domain, cname, ttl })
            }
            QuestionType::MX => {
                let priority = buffer.read_u16()?;
                // TODO read_qname can definitely return a string
                let mut exchange = String::new();
                buffer.read_qname(&mut exchange)?;

                Ok(ResourceRecord::MX {
                    domain,
                    priority,
                    exchange,
                    ttl,
                })
            }
            QuestionType::UNKNOWN(_) => {
                buffer.step(data_len as usize)?; // TODO: what's the point of this? To see if it returns a negative result?

                Ok(ResourceRecord::UNKNOWN {
                    domain: domain,
                    qtype: qtype_num,
                    data_len: data_len,
                    ttl: ttl,
                })
            }
        }
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<usize> {
        let start_pos = buffer.pos();

        // TODO: see if i can tidy this a bit
        match *self {
            ResourceRecord::A {
                ref domain,
                ref ip_addr,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QuestionType::A.to_num())?;
                buffer.write_u16(1)?; // Class
                buffer.write_u32(ttl)?;
                buffer.write_u16(4)?; // TODO: 4 parts of IP?

                let octets = ip_addr.octets();
                buffer.write_u8(octets[0])?;
                buffer.write_u8(octets[1])?;
                buffer.write_u8(octets[2])?;
                buffer.write_u8(octets[3])?;
            }
            ResourceRecord::NS {
                ref domain,
                ref host,
                ttl,
            } => {
                // TODO: lots of this is common. Can we compress it?
                buffer.write_qname(domain)?;
                buffer.write_u16(QuestionType::NS.to_num())?;
                buffer.write_u16(1)?; // Class
                buffer.write_u32(ttl)?;

                // TODO: can we do pos after writing 0 so we don't need to -2?
                // TODO: can this code be extracted since it;s common?
                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_qname(host)?;

                let size = buffer.pos() - (pos - 2);
                buffer.set_u16(pos, size as u16)?;
            }
            ResourceRecord::CNAME {
                ref domain,
                ref cname,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QuestionType::CNAME.to_num())?;
                buffer.write_u16(1)?; // Class
                buffer.write_u32(ttl)?;

                // TODO: can we do pos after writing 0 so we don't need to -2?
                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_qname(cname)?;

                let size = buffer.pos() - (pos + 2);
                buffer.set_u16(pos, size as u16)?;
            }
            ResourceRecord::MX {
                ref domain,
                priority,
                ref exchange,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QuestionType::MX.to_num())?;
                buffer.write_u16(1)?; // Class
                buffer.write_u32(ttl)?;

                // TODO: can we do pos after writing 0 so we don't need to -2?
                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_u16(priority)?;
                buffer.write_qname(exchange)?;

                let size = buffer.pos() - (pos + 2);
                buffer.set_u16(pos, size as u16)?;
            }
            ResourceRecord::AAAA {
                ref domain,
                ref ip_addr,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QuestionType::AAAA.to_num())?;
                buffer.write_u16(1)?; // Class
                buffer.write_u32(ttl)?;
                buffer.write_u16(16)?; // 16 as in 16 octets?

                for octet in &ip_addr.segments() {
                    buffer.write_u16(*octet)?;
                }
            }
            ResourceRecord::UNKNOWN { .. } => {
                println!("Skipping unknown record: {:?}", self);
            }
        }

        // TODO: why are we returning the size?
        Ok(buffer.pos() - start_pos)
    }
}
