use crate::packet_buffer::PacketBuffer;

use std::net::{Ipv4Addr, Ipv6Addr};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub const INTERNET_CLASS: u16 = 1;

pub trait BufferIO {
    fn from_buffer(buffer: &mut PacketBuffer) -> Result<Self>
    where
        Self: Sized;
    fn write_to_buffer(&mut self, buffer: &mut PacketBuffer) -> Result<()>;
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u16)]
pub enum QueryType {
    /* NOTE: ideally this would use the num-derive crate and explicit discriminants
     * in enums from nightly Rust (RFC 2363), but the features don't inter-work. */
    UNKNOWN(u16),
    A,
    NS,
    CNAME,
    MX,
    AAAA,
}

impl QueryType {
    pub fn to_num(self) -> u16 {
        match self {
            Self::A => 1,
            Self::NS => 2,
            Self::CNAME => 5,
            Self::MX => 15,
            Self::AAAA => 28,
            Self::UNKNOWN(num) => num,
        }
    }

    pub fn from_num(num: u16) -> Self {
        match num {
            1 => Self::A,
            2 => Self::NS,
            5 => Self::CNAME,
            15 => Self::MX,
            28 => Self::AAAA,
            _ => Self::UNKNOWN(num),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub qname: String,
    pub qtype: QueryType,
    class: u16,
}

impl Query {
    pub fn new(qname: String, qtype: QueryType) -> Self {
        Self {
            qname,
            qtype,
            class: INTERNET_CLASS,
        }
    }
}

impl BufferIO for Query {
    fn from_buffer(buffer: &mut PacketBuffer) -> Result<Self> {
        let qname = buffer.read_compressed_name()?;
        let qtype = QueryType::from_num(buffer.read_u16()?);
        let _class = buffer.read_u16()?;

        Ok(Query::new(qname, qtype))
    }

    fn write_to_buffer(&mut self, buffer: &mut PacketBuffer) -> Result<()> {
        buffer.write_compressed_name(&self.qname)?;
        buffer.write_u16(self.qtype.to_num())?;
        buffer.write_u16(self.class)?;

        Ok(())
    }
}
#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
pub enum ReturnCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub id: u16,

    pub response: bool,
    pub opcode: u8, // 4 bits
    pub authoritative_answer: bool,
    pub truncated_message: bool,
    pub recursion_desired: bool,

    pub recursion_available: bool,
    pub z: bool,
    pub authenticated_data: bool,
    pub checking_disabled: bool,
    pub return_code: ReturnCode, // 4 bits

    pub queries_total: u16,
    pub answer_rr_total: u16,
    pub authoritative_rr_total: u16,
    pub additional_rr_total: u16,
}

impl Header {
    pub fn new() -> Self {
        Header {
            id: 0,

            response: false,
            opcode: 0,
            authoritative_answer: false,
            truncated_message: false,
            recursion_desired: false,

            recursion_available: false,
            z: false,
            authenticated_data: false,
            checking_disabled: false,
            return_code: ReturnCode::NOERROR,

            queries_total: 0,
            answer_rr_total: 0,
            authoritative_rr_total: 0,
            additional_rr_total: 0,
        }
    }
}

impl BufferIO for Header {
    fn from_buffer(buffer: &mut PacketBuffer) -> Result<Self> {
        let mut header = Self::new();

        header.id = buffer.read_u16()?;

        let flags = buffer.read_u16()?;
        let flags_b1 = (flags >> 8) as u8;
        let flags_b2 = (flags & 0xFF) as u8;

        header.response = (flags_b1 & (1 << 7)) > 0;
        header.opcode = (flags_b1 >> 3) & 0x0F;
        header.authoritative_answer = (flags_b1 & (1 << 2)) > 0;
        header.truncated_message = (flags_b1 & (1 << 1)) > 0;
        header.recursion_desired = (flags_b1 & 1) > 0;

        header.recursion_available = (flags_b2 & (1 << 7)) > 0;
        header.z = (flags_b2 & (1 << 6)) > 0;
        header.authenticated_data = (flags_b2 & (1 << 5)) > 0;
        header.checking_disabled = (flags_b2 & (1 << 4)) > 0;
        header.return_code = num::FromPrimitive::from_u8(flags_b2 & 0x0F).unwrap();

        header.queries_total = buffer.read_u16()?;
        header.answer_rr_total = buffer.read_u16()?;
        header.authoritative_rr_total = buffer.read_u16()?;
        header.additional_rr_total = buffer.read_u16()?;

        Ok(header)
    }

    fn write_to_buffer(&mut self, buffer: &mut PacketBuffer) -> Result<()> {
        buffer.write_u16(self.id)?;

        let mut flags_b1 = (self.response as u8) << 7;
        flags_b1 |= self.opcode << 3;
        flags_b1 |= (self.authoritative_answer as u8) << 2;
        flags_b1 |= (self.truncated_message as u8) << 1;
        flags_b1 |= (self.recursion_desired as u8) as u8;
        buffer.write_u8(flags_b1)?;

        let mut flags_b2 = (self.recursion_available as u8) << 7;
        flags_b2 |= (self.z as u8) << 6;
        flags_b2 |= (self.authenticated_data as u8) << 5;
        flags_b2 |= (self.checking_disabled as u8) << 4;
        flags_b2 |= self.return_code as u8;
        buffer.write_u8(flags_b2)?;

        buffer.write_u16(self.queries_total)?;
        buffer.write_u16(self.answer_rr_total)?;
        buffer.write_u16(self.authoritative_rr_total)?;
        buffer.write_u16(self.additional_rr_total)?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct Packet {
    pub header: Header,
    pub queries: Vec<Query>,
    pub answer_records: Vec<ResourceRecord>,
    pub authoritative_records: Vec<ResourceRecord>,
    pub additional_records: Vec<ResourceRecord>,
}

impl Packet {
    pub fn new() -> Self {
        Packet {
            header: Header::new(),
            queries: Vec::new(),
            answer_records: Vec::new(),
            authoritative_records: Vec::new(),
            additional_records: Vec::new(),
        }
    }

    pub fn get_answer_a_records(&self) -> Vec<&Ipv4Addr> {
        self.answer_records
            .iter()
            .filter_map(|record| match record {
                ResourceRecord::A { ip_addr, .. } => Some(ip_addr),
                _ => None,
            })
            .collect()
    }

    fn get_ns_domain_host_iter<'a>(
        &'a self,
        qname: &'a str,
    ) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.authoritative_records
            .iter()
            .filter_map(|record| match record {
                ResourceRecord::NS { domain, host, .. } => Some((domain.as_str(), host.as_str())),
                _ => None,
            })
            // Only include domains authoritative to the query
            .filter(move |(domain, _)| qname.ends_with(*domain))
    }

    pub fn get_ns_hosts<'a>(&'a self, qname: &'a str) -> Vec<&'a str> {
        self.get_ns_domain_host_iter(qname)
            .map(|(_, host)| host)
            .collect()
    }

    pub fn get_ns_from_additional_records(&self, qname: &str) -> Vec<&Ipv4Addr> {
        self.get_ns_domain_host_iter(qname)
            .flat_map(|(_, host)| {
                self.additional_records
                    .iter()
                    .filter_map(move |record| match record {
                        ResourceRecord::A {
                            domain, ip_addr, ..
                        } if domain == host => Some(ip_addr),
                        _ => None,
                    })
            })
            .collect()
    }
}

impl BufferIO for Packet {
    fn from_buffer(buffer: &mut PacketBuffer) -> Result<Self> {
        let mut packet = Packet::new();
        packet.header = Header::from_buffer(buffer)?;

        for _ in 0..packet.header.queries_total {
            let query = Query::from_buffer(buffer)?;
            packet.queries.push(query);
        }

        for _ in 0..packet.header.answer_rr_total {
            let record = ResourceRecord::from_buffer(buffer)?;
            packet.answer_records.push(record);
        }
        for _ in 0..packet.header.authoritative_rr_total {
            let record = ResourceRecord::from_buffer(buffer)?;
            packet.authoritative_records.push(record);
        }
        for _ in 0..packet.header.additional_rr_total {
            let record = ResourceRecord::from_buffer(buffer)?;
            packet.additional_records.push(record);
        }

        Ok(packet)
    }

    fn write_to_buffer(&mut self, buffer: &mut PacketBuffer) -> Result<()> {
        self.header.queries_total = self.queries.len() as u16;
        self.header.answer_rr_total = self.answer_records.len() as u16;
        self.header.authoritative_rr_total = self.authoritative_records.len() as u16;
        self.header.additional_rr_total = self.additional_records.len() as u16;

        self.header.write_to_buffer(buffer)?;

        for query in &mut self.queries {
            query.write_to_buffer(buffer)?;
        }
        for rec in &mut self.answer_records {
            rec.write_to_buffer(buffer)?;
        }
        for rec in &mut self.authoritative_records {
            rec.write_to_buffer(buffer)?;
        }
        for rec in &mut self.additional_records {
            rec.write_to_buffer(buffer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ResourceRecord {
    UNKNOWN {
        domain: String,
        qtype: u16,
        data_len: u16,
        ttl: u32,
    },
    A {
        domain: String,
        ip_addr: Ipv4Addr,
        ttl: u32,
    },
    NS {
        domain: String,
        host: String,
        ttl: u32,
    },
    CNAME {
        domain: String,
        host: String,
        ttl: u32,
    },
    MX {
        domain: String,
        priority: u16,
        exchange: String,
        ttl: u32,
    },
    AAAA {
        domain: String,
        ip_addr: Ipv6Addr,
        ttl: u32,
    },
}

impl ResourceRecord {
    fn write_common_fields(
        &self,
        buffer: &mut PacketBuffer,
        domain: &str,
        qtype: QueryType,
        ttl: u32,
    ) -> Result<()> {
        buffer.write_compressed_name(domain)?;
        buffer.write_u16(qtype.to_num())?;
        buffer.write_u16(INTERNET_CLASS)?;
        buffer.write_u32(ttl)?;

        Ok(())
    }

    fn write_compressed_name_with_size(&self, buffer: &mut PacketBuffer, name: &str) -> Result<()> {
        // Skip over size field
        let name_size_field_len = 2;
        buffer.step(name_size_field_len);

        // Write name and get its size
        let name_start_pos = buffer.pos();
        buffer.write_compressed_name(name)?;
        let name_size = buffer.pos() - name_start_pos;

        // Write previously skipped size field
        buffer.set_u16(name_start_pos - name_size_field_len, name_size as u16)?;

        Ok(())
    }
}

impl BufferIO for ResourceRecord {
    fn from_buffer(buffer: &mut PacketBuffer) -> Result<Self> {
        let domain = buffer.read_compressed_name()?;

        let qtype_num = buffer.read_u16()?;
        let qtype = QueryType::from_num(qtype_num);
        let _class = buffer.read_u16()?;
        let ttl = buffer.read_u32()?;
        let data_len = buffer.read_u16()?;

        match qtype {
            QueryType::A => {
                let ip_addr_u32 = buffer.read_u32()?;

                let ip_addr = Ipv4Addr::new(
                    ((ip_addr_u32 >> 24) & 0xFF) as u8,
                    ((ip_addr_u32 >> 16) & 0xFF) as u8,
                    ((ip_addr_u32 >> 8) & 0xFF) as u8,
                    (ip_addr_u32 & 0xFF) as u8,
                );

                Ok(ResourceRecord::A {
                    domain,
                    ip_addr,
                    ttl,
                })
            }
            QueryType::AAAA => {
                let ip_addr_u32_1 = buffer.read_u32()?;
                let ip_addr_u32_2 = buffer.read_u32()?;
                let ip_addr_u32_3 = buffer.read_u32()?;
                let ip_addr_u32_4 = buffer.read_u32()?;

                let ip_addr = Ipv6Addr::new(
                    ((ip_addr_u32_1 >> 16) & 0xFFFF) as u16,
                    (ip_addr_u32_1 & 0xFFFF) as u16,
                    ((ip_addr_u32_2 >> 16) & 0xFFFF) as u16,
                    (ip_addr_u32_2 & 0xFFFF) as u16,
                    ((ip_addr_u32_3 >> 16) & 0xFFFF) as u16,
                    (ip_addr_u32_3 & 0xFFFF) as u16,
                    ((ip_addr_u32_4 >> 16) & 0xFFFF) as u16,
                    (ip_addr_u32_4 & 0xFFFF) as u16,
                );

                Ok(ResourceRecord::AAAA {
                    domain,
                    ip_addr,
                    ttl,
                })
            }
            QueryType::NS => {
                let host = buffer.read_compressed_name()?;

                Ok(ResourceRecord::NS { domain, host, ttl })
            }
            QueryType::CNAME => {
                let host = buffer.read_compressed_name()?;

                Ok(ResourceRecord::CNAME { domain, host, ttl })
            }
            QueryType::MX => {
                let priority = buffer.read_u16()?;
                let exchange = buffer.read_compressed_name()?;

                Ok(ResourceRecord::MX {
                    domain,
                    priority,
                    exchange,
                    ttl,
                })
            }
            QueryType::UNKNOWN(_) => {
                buffer.step(data_len as usize);

                Ok(ResourceRecord::UNKNOWN {
                    domain,
                    qtype: qtype_num,
                    data_len,
                    ttl,
                })
            }
        }
    }

    fn write_to_buffer(&mut self, buffer: &mut PacketBuffer) -> Result<()> {
        match *self {
            ResourceRecord::A {
                ref domain,
                ref ip_addr,
                ttl,
            } => {
                self.write_common_fields(buffer, domain, QueryType::A, ttl)?;

                buffer.write_u16(ip_addr.octets().len() as u16)?;

                for octet in &ip_addr.octets() {
                    buffer.write_u8(*octet)?;
                }
            }
            ResourceRecord::NS {
                ref domain,
                ref host,
                ttl,
            } => {
                self.write_common_fields(buffer, domain, QueryType::NS, ttl)?;

                self.write_compressed_name_with_size(buffer, host)?;
            }
            ResourceRecord::CNAME {
                ref domain,
                ref host,
                ttl,
            } => {
                self.write_common_fields(buffer, domain, QueryType::CNAME, ttl)?;

                self.write_compressed_name_with_size(buffer, host)?;
            }
            ResourceRecord::MX {
                ref domain,
                priority,
                ref exchange,
                ttl,
            } => {
                self.write_common_fields(buffer, domain, QueryType::MX, ttl)?;

                // Skip over size field
                let name_size_field_len = 2;
                buffer.step(name_size_field_len);

                // Write name and get its size
                let name_start_pos = buffer.pos();
                buffer.write_u16(priority)?;
                buffer.write_compressed_name(exchange)?;
                let name_size = buffer.pos() - name_start_pos;

                // Write previously skipped size field
                buffer.set_u16(name_start_pos - name_size_field_len, name_size as u16)?;
            }
            ResourceRecord::AAAA {
                ref domain,
                ref ip_addr,
                ttl,
            } => {
                self.write_common_fields(buffer, domain, QueryType::AAAA, ttl)?;

                buffer.write_u16(ip_addr.octets().len() as u16)?;

                for segment_u16 in &ip_addr.segments() {
                    buffer.write_u16(*segment_u16)?;
                }
            }
            ResourceRecord::UNKNOWN { .. } => {
                println!("Skipping unknown record: {:?}", self);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read_packet() {
        /* Arrange */
        let mut packet = Packet::new();

        packet
            .queries
            .push(Query::new("google.com".to_string(), QueryType::NS));
        packet.answer_records.push(ResourceRecord::NS {
            domain: "google.com".to_string(),
            host: "ns1.google.com".to_string(),
            ttl: 64,
        });
        packet.answer_records.push(ResourceRecord::NS {
            domain: "google.com".to_string(),
            host: "ns2.google.com".to_string(),
            ttl: 64,
        });

        /* Act */
        let mut buffer = PacketBuffer::new();
        packet.write_to_buffer(&mut buffer).unwrap();

        buffer.seek(0);
        let parsed_packet = Packet::from_buffer(&mut buffer).unwrap();

        /* Assert */
        assert_eq!(packet.queries[0], parsed_packet.queries[0]);
        assert_eq!(packet.answer_records[0], parsed_packet.answer_records[0]);
        assert_eq!(packet.answer_records[1], parsed_packet.answer_records[1]);
    }
}
