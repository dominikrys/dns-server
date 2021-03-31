use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use super::packet_buffer::PacketBuffer;
use super::query::INTERNET_CLASS;
use super::query_type::QueryType;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
        cname: String,
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
    pub fn from_buffer(buffer: &mut PacketBuffer) -> Result<ResourceRecord> {
        // NOTE: buffer pos must be at the start of a resource record
        let domain = buffer.read_qname()?;

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
                let host = buffer.read_qname()?;

                Ok(ResourceRecord::NS { domain, host, ttl })
            }
            QueryType::CNAME => {
                let cname = buffer.read_qname()?;

                Ok(ResourceRecord::CNAME { domain, cname, ttl })
            }
            QueryType::MX => {
                let priority = buffer.read_u16()?;
                let exchange = buffer.read_qname()?;

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

    fn write_common_fields(
        &self,
        buffer: &mut PacketBuffer,
        domain: &str,
        qtype: QueryType,
        ttl: u32,
    ) -> Result<()> {
        buffer.write_qname(domain)?;
        buffer.write_u16(qtype.to_num())?;
        buffer.write_u16(INTERNET_CLASS)?;
        buffer.write_u32(ttl)?;

        Ok(())
    }

    pub fn write_to_buffer(&self, buffer: &mut PacketBuffer) -> Result<usize> {
        // NOTE: this method will write at the current buffer position
        let start_pos = buffer.pos();

        match *self {
            ResourceRecord::A {
                ref domain,
                ref ip_addr,
                ttl,
            } => {
                self.write_common_fields(buffer, domain, QueryType::A, ttl)?;

                buffer.write_u16(4)?; // 4 IPv4 octets

                // TODO: do a loop for this
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
                self.write_common_fields(buffer, domain, QueryType::NS, ttl)?;

                // TODO: can we do pos after writing 0 so we don't need to -2?
                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_qname(host)?;

                let size = buffer.pos() - (pos - 2);
                buffer.seek(pos);
                buffer.write_u16(size as u16)?;
            }
            ResourceRecord::CNAME {
                ref domain,
                ref cname,
                ttl,
            } => {
                self.write_common_fields(buffer, domain, QueryType::CNAME, ttl)?;

                // TODO: can we do pos after writing 0 so we don't need to -2?
                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_qname(cname)?;

                let size = buffer.pos() - (pos + 2);
                buffer.seek(pos);
                buffer.write_u16(size as u16)?;
            }
            ResourceRecord::MX {
                ref domain,
                priority,
                ref exchange,
                ttl,
            } => {
                self.write_common_fields(buffer, domain, QueryType::MX, ttl)?;

                // TODO: can we do pos after writing 0 so we don't need to -2?
                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_u16(priority)?;
                buffer.write_qname(exchange)?;

                let size = buffer.pos() - (pos + 2);
                buffer.seek(pos);
                buffer.write_u16(size as u16)?;
            }
            ResourceRecord::AAAA {
                ref domain,
                ref ip_addr,
                ttl,
            } => {
                self.write_common_fields(buffer, domain, QueryType::AAAA, ttl)?;

                buffer.write_u16(16)?; // 16 IPv6 octets

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
