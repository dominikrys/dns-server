#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    // TODO: do all these need to be pub?
    pub name: String,
    pub qtype: QuestionType,
}

impl Question {
    pub fn new(name: String, qtype: QuestionType) -> Self {
        Self { name, qtype }
    }

    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        // TODO: this assumes that the buffer position is at the start. Maybe we should set it explicitly
        buffer.read_qname(&mut self.name)?;

        self.qtype = QuestionType::from_num(buffer.read_u16()?);
        let _ = buffer.read_u16()?; // class
                                    // TODO: do we keep the class?

        Ok(())
    }
}
