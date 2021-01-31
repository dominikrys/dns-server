// TODO: see if all these traits are needed
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(dead_code)] // Maybe remove this
pub enum ResourceRecord {
    // TODO: do we need an unknown record?
    // TODO: can we include the integer within these?
    // TODO: do we need a new enum for every record type?
    // These are pretty generic and the header is the same for all: http://www.networksorcery.com/enp/protocol/dns.htm#Answer%20RRs
    UNKNOWN {
        domain: String,
        qtype: u16,
        data_len: u16,
        ttl: u32,
    }, // 0
    A {
        domain: String,
        addr: Ipv4Addr,
        ttl: u32,
    }, // 1
}

impl ResourceRecord {
    pub fn read(buffer: &mut BytePacketBuffer) -> Result<DnsRecord> {
        // TODO: this assumes that the bytepacketbuffer is at the start. Maybe reset it?
        let mut domain = String::new();
        buffer.read_qname(&mut domain)?;

        let qtype_num = buffer.read_u16()?;
        let qtype = QueryType::from_num(qtype_num); // TODO: maybe combine these two in one
        let _ = buffer.read_u16()?; // Class
        let ttl = buffer.read_u32()?;
        let data_len = buffer.read_u16()?;

        // TODO: use the data_len in here somehow, maybe check for limit
        // TODO: support IPv6? Or mention that it's IPv4 only and that it will break otherwise
        match qtype {
            QueryType::A => {
                let raw_ip_addr = buffer.read_u32()?;
                let ip_addr = Ipv4Addr::new(
                    ((raw_ip_addr >> 24) & 0xFF) as u8,
                    ((raw_ip_addr >> 16) & 0xFF) as u8,
                    ((raw_ip_addr >> 8) & 0xFF) as u8,
                    ((raw_ip_addr >> 0) & 0xFF) as u8,
                );

                // TODO: can we not repeat this
                Ok(DnsRecord::A {
                    domain: domain,
                    addr: addr,
                    ttl: ttl,
                })
            }
            QueryType::UNKNOWN(_) => {
                buffer.step(data_len as usize)?; // TODO: what's the point of this? To see if it returns a negative result?

                Ok(DnsRecord::UNKNOWN {
                    domain: domain,
                    qtype: qtype_num,
                    data_len: data_len,
                    ttl: ttl,
                })
            }
        }
    }
}
