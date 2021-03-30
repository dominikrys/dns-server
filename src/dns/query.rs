use super::packet_buffer::PacketBuffer;
use super::query_type::QueryType;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
            class: 1, // IN (Internet)
        }
    }

    pub fn from_buffer(buffer: &mut PacketBuffer) -> Result<Query> {
        // NOTE: buffer pos must be at the start of a query
        let qname = buffer.read_qname()?;
        let qtype = QueryType::from_num(buffer.read_u16()?);
        let _class = buffer.read_u16()?;

        Ok(Query::new(qname, qtype))
    }

    pub fn write_to_buffer(&self, buffer: &mut PacketBuffer) -> Result<()> {
        // NOTE: this method will write at the current buffer position
        buffer.write_qname(&self.qname)?;
        buffer.write_u16(self.qtype.to_num())?;
        buffer.write_u16(self.class)?;

        Ok(())
    }
}
