use std::net::Ipv4Addr;

use super::header::Header;
use super::packet_buffer::PacketBuffer;
use super::query::Query;
use super::resource_record::ResourceRecord;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: do we need both these traits?
#[derive(Clone, Debug)]
// TODO: maybe rename to just "packet"? and the other _dns stuff. Redundant since inside "DNS" module?
pub struct Packet {
    // TODO: should these all be pub?
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
        let mut result = Packet::new();
        result.header = Header::from_buffer(buffer)?;

        // TODO: can we tidy this repetition?
        for _ in 0..result.header.queries_total {
            let query = Query::from_buffer(buffer)?;
            result.queries.push(query);
        }

        for _ in 0..result.header.answer_rr_total {
            let record = ResourceRecord::from_buffer(buffer)?;
            result.answer_records.push(record);
        }
        for _ in 0..result.header.authoritative_rr_total {
            let record = ResourceRecord::from_buffer(buffer)?;
            result.authoritative_records.push(record);
        }
        for _ in 0..result.header.additional_rr_total {
            let record = ResourceRecord::from_buffer(buffer)?;
            result.additional_records.push(record);
        }

        Ok(result)
    }

    // TODO: does self need to be mut?
    // TODO: should this return buffer ot take it in as a reference?
    pub fn write(&mut self, buffer: &mut PacketBuffer) -> Result<()> {
        self.header.queries_total = self.queries.len() as u16;
        self.header.answer_rr_total = self.answer_records.len() as u16;
        self.header.authoritative_rr_total = self.authoritative_records.len() as u16;
        self.header.additional_rr_total = self.additional_records.len() as u16;

        self.header.write(buffer)?;

        for query in &self.queries {
            query.write(buffer)?;
        }
        for rec in &self.answer_records {
            rec.write(buffer)?;
        }
        for rec in &self.authoritative_records {
            rec.write(buffer)?;
        }
        for rec in &self.additional_records {
            rec.write(buffer)?;
        }

        Ok(())
    }

    pub fn get_first_a_record(&self) -> Option<Ipv4Addr> {
        self.answer_records
            .iter()
            .filter_map(|record| match record {
                ResourceRecord::A { ip_addr, .. } => Some(*ip_addr),
                _ => None,
            })
            .next()
    }

    // TODO: read through the next two functions properly: https://github.com/EmilHernvall/dnsguide/blob/master/chapter5.md

    // TODO: make return type obvious that they're a domain and host
    fn get_ns_iter<'a>(&'a self, qname: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.authoritative_records
            .iter()
            .filter_map(|record| match record {
                ResourceRecord::NS { domain, host, .. } => Some((domain.as_str(), host.as_str())),
                _ => None,
            })
            // Discard servers which aren't authoritative to our query
            // TODO: make this cleaner
            .filter(move |(domain, _)| qname.ends_with(*domain))
    }

    pub fn get_ns_from_additional_records(&self, qname: &str) -> Option<Ipv4Addr> {
        // TODO: implement out of bailiwick check: https://www.farsightsecurity.com/blog/txt-record/what-is-a-bailiwick-20170321
        // TODO: otherwise, maybe remove this method. Check what "authoritative" is supposed to mean here
        self.get_ns_iter(qname)
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
            .cloned()
            // TODO: can we use something else than .next()?
            .next()
    }

    pub fn get_ns_host<'a>(&'a self, qname: &'a str) -> Option<&'a str> {
        self.get_ns_iter(qname)
            // TODO: tidy this
            .map(|(_, host)| host)
            .next()
    }
}
