#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub name: String,
    pub qtype: QueryType,
}

impl Query {
    pub fn new(name: String, qtype: QueryType) -> Self {
        Self { name, qtype }
    }

    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        // TODO: this assumes that the buffer position is at the start. Maybe we should set it explicitly
        buffer.read_qname(&mut self.name)?;

        self.qtype = QueryType::from_num(buffer.read_u16()?);
        let _ = buffer.read_u16()?; // class
                                    // TODO: do we keep the class?

        Ok(())
    }
}
