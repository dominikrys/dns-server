use std::net::Ipv4Addr;

use super::packet_buffer::PacketBuffer;
use super::dns_header::DnsHeader;
use super::question::Question;
use super::question_type::QuestionType;
use super::resource_record::ResourceRecord;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: do we need both these traits?
#[derive(Clone, Debug)]
// TODO: maybe rename to just "packet"? and the other _dns stuff. Redundant since inside "DNS" module?
pub struct DnsPacket {
    // TODO: should these all be pub?
    pub header: DnsHeader,
    pub questions: Vec<Question>,
    pub answer_records: Vec<ResourceRecord>,
    pub authoritative_records: Vec<ResourceRecord>,
    pub additional_records: Vec<ResourceRecord>,
}

impl DnsPacket {
    pub fn new() -> Self {
        DnsPacket {
            header: DnsHeader::new(),
            questions: Vec::new(),
            answer_records: Vec::new(),
            authoritative_records: Vec::new(),
            additional_records: Vec::new(),
        }
    }

    pub fn from_buffer(buffer: &mut PacketBuffer) -> Result<DnsPacket> {
        let mut result = DnsPacket::new();
        result.header.read_u8(buffer)?;

        // TODO: can we tidy this repetition?
        for _ in 0..result.header.questions_total {
            let mut question = Question::new("".to_string(), QuestionType::UNKNOWN(0));
            question.read_u8(buffer)?;
            result.questions.push(question);
        }

        for _ in 0..result.header.answer_rr_total {
            let record = ResourceRecord::read_u8(buffer)?;
            result.answer_records.push(record);
        }
        for _ in 0..result.header.authoritative_rr_total {
            let record = ResourceRecord::read_u8(buffer)?;
            result.authoritative_records.push(record);
        }
        for _ in 0..result.header.additional_rr_total {
            let record = ResourceRecord::read_u8(buffer)?;
            result.additional_records.push(record);
        }

        Ok(result)
    }

    // TODO: does self need to be mut?
    // TODO: should this return buffer ot take it in as a reference?
    pub fn write(&mut self, buffer: &mut PacketBuffer) -> Result<()> {
        self.header.questions_total = self.questions.len() as u16;
        self.header.answer_rr_total = self.answer_records.len() as u16;
        self.header.authoritative_rr_total = self.authoritative_records.len() as u16;
        self.header.additional_rr_total = self.additional_records.len() as u16;

        self.header.write(buffer)?;

        for question in &self.questions {
            question.write(buffer)?;
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
