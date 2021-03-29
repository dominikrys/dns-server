use super::byte_packet_buffer::PacketBuffer;
use super::question_type::QuestionType;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

    pub fn read_u8(&mut self, buffer: &mut PacketBuffer) -> Result<()> {
        // TODO: this assumes that the buffer position is at the start. Maybe we should set it explicitly
        buffer.read_qname(&mut self.name)?;

        self.qtype = QuestionType::from_num(buffer.read_u16()?);
        let _ = buffer.read_u16()?; // class
                                    // TODO: do we keep the class?

        Ok(())
    }

    // TODO: return the buffer
    pub fn write(&self, buffer: &mut PacketBuffer) -> Result<()> {
        buffer.write_qname(&self.name)?;

        let type_num = self.qtype.to_num();
        buffer.write_u16(type_num)?;
        buffer.write_u16(1)?; // Class

        Ok(())
    }
}
