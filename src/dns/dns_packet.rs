use super::byte_packet_buffer::BytePacketBuffer;
use super::dns_header::DnsHeader;
use super::question::Question;
use super::question_type::QuestionType;
use super::resource_record::ResourceRecord;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: do we need both these traits?
#[derive(Clone, Debug)]
// TODO: maybe rename to just "packet"?
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

    pub fn from_buffer(buffer: &mut BytePacketBuffer) -> Result<DnsPacket> {
        let mut result = DnsPacket::new();
        result.header.read(buffer)?;

        // TODO: can we tidy this repetition?
        for _ in 0..result.header.questions_total {
            let mut question = Question::new("".to_string(), QuestionType::UNKNOWN(0));
            question.read(buffer)?;
            result.questions.push(question);
        }

        for _ in 0..result.header.answer_rr_total {
            let record = ResourceRecord::read(buffer)?;
            result.answer_records.push(record);
        }
        for _ in 0..result.header.authoritative_rr_total {
            let record = ResourceRecord::read(buffer)?;
            result.authoritative_records.push(record);
        }
        for _ in 0..result.header.additional_rr_total {
            let record = ResourceRecord::read(buffer)?;
            result.additional_records.push(record);
        }

        Ok(result)
    }

    // TODO: does self need to be mut?
    // TODO: should this return buffer ot take it in as a reference?
    pub fn write(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
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
}
