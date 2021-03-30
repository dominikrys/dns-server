use super::packet_buffer::PacketBuffer;
use super::query_type::QueryType;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    // TODO: do all these need to be pub?
    pub name: String,
    pub qtype: QueryType,
    // Class not included as it's constant
}

impl Query {
    pub fn new(name: String, qtype: QueryType) -> Self {
        Self { name, qtype }
    }

    pub fn from_buffer(buffer: &mut PacketBuffer) -> Result<Query> {
        // NOTE: buffer pos must be at the start of a query

        let qname = buffer.read_qname()?;
        let qtype = QueryType::from_num(buffer.read_u16()?);
        let _class = buffer.read_u16()?;

        Ok(Query::new(qname, qtype))
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
