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
            header: DnsJeader::new(),
            questions: Vec::new(),
            answer_records: Vec::new(),
            authoritative_records: Vec::new(),
            additional_records: Vec::new(),
        }
    }

    pub fn from_buffer(buffer: &mut BytePacketBuffer) -> Result<DnsPacket> {
        let mut result = DnsPacket::new();
        result.header.read(buffer)?;

        for _ in 0..result.header.questions {
            let mut question = Question::new("".to_string(), QuestionType::UNKNOWN(0));
            question.read(buffer)?;
            result.questions.push(question);
        }

        for _ in 0..result.header.answers {
            let record = ResourceRecord::read(buffer)?;
            result.answers.push(record);
        }
        for _ in 0..result.header.authoritative_entries {
            let record = ResourceRecord::read(buffer)?;
            result.authorities.push(record);
        }
        for _ in 0..result.header.resource_entries {
            let record = ResourceRecord::read(buffer)?;
            result.resources.push(record);
        }

        Ok(result)
    }
}
