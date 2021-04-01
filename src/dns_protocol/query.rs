use super::packet_buffer::PacketBuffer;
use super::query_type::QueryType;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub const INTERNET_CLASS: u16 = 1;

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
            class: INTERNET_CLASS,
        }
    }

    pub fn from_buffer(buffer: &mut PacketBuffer) -> Result<Query> {
        let qname = buffer.read_compressed_name()?;
        let qtype = QueryType::from_num(buffer.read_u16()?);
        let _class = buffer.read_u16()?;

        Ok(Query::new(qname, qtype))
    }

    pub fn write_to_buffer(&self, buffer: &mut PacketBuffer) -> Result<()> {
        buffer.write_compressed_name(&self.qname)?;
        buffer.write_u16(self.qtype.to_num())?;
        buffer.write_u16(self.class)?;

        Ok(())
    }
}
