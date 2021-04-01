use std::net::Ipv4Addr;

use super::header::Header;
use super::packet_buffer::PacketBuffer;
use super::query::Query;
use super::resource_record::ResourceRecord;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

    pub fn from_buffer(buffer: &mut PacketBuffer) -> Result<Packet> {
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

    pub fn write_to_buffer(&mut self, buffer: &mut PacketBuffer) -> Result<()> {
        self.header.queries_total = self.queries.len() as u16;
        self.header.answer_rr_total = self.answer_records.len() as u16;
        self.header.authoritative_rr_total = self.authoritative_records.len() as u16;
        self.header.additional_rr_total = self.additional_records.len() as u16;

        self.header.write_to_buffer(buffer)?;

        for query in &self.queries {
            query.write_to_buffer(buffer)?;
        }
        for rec in &self.answer_records {
            rec.write_to_buffer(buffer)?;
        }
        for rec in &self.authoritative_records {
            rec.write_to_buffer(buffer)?;
        }
        for rec in &self.additional_records {
            rec.write_to_buffer(buffer)?;
        }

        Ok(())
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
