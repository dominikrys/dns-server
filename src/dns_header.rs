// Reference: http://www.networksorcery.com/enp/protocol/dns.htm
// TODO: maybe rename to "header"
#[derive(Clone, Debug)]
pub struct DnsHeader {
    pub id: u16, // 16 bits

    pub response: bool,             // 1 bit
    pub opcode: u8,                 // 4 bits
    pub authoritative_answer: bool, // 1 bit
    pub truncated_message: bool,    // 1 bit
    pub recursion_desired: bool,    // 1 bit

    pub recursion_available: bool, // 1 bit
    pub z: bool,                   // 1 bit
    pub authenticated_data: bool,  // 1 bit
    pub checking_disabled: bool,   // 1 bit
    pub return_code: ReturnCode,   // 4 bits

    pub questions_total: u16,        // 16 bits
    pub answer_rr_total: u16,        // 16 bits
    pub authoritative_rr_total: u16, // 16 bits
    pub additional_rr_total: u16,    // 16 bits
}

impl DnsHeader {
    pub fn new() -> Self {
        DnsHeader {
            response: false,
            opcode: 0,
            authoritative_answer: false,
            truncated_message: false,
            recursion_desired: false,

            return_code: ReturnCode::NOERROR,
            checking_disabled: false,
            authenticated_data: false,
            z: false,
            recursion_available: false,

            recursion_available: false,
            z: false,
            authenticated_data: false,
            checking_disabled: false,
            return_code: ReturnCode::NOERROR,
        }
    }

    // TODO: rename this - this is more like creation from buffer
    // TODO: have a constructor for this, and not return an input
    pub fn read(&mut self, &mut buffer: BytePacketBuffer) -> Result<()> {
        // TODO: reset buffer pos?
        self.id = buffer.read_u16()?;

        let flags = buffer.read_u16()?;
        let a = (flags >> 8) as u8; // TODO: rename to higher bits
        let b = (flags & 0xFF) as u8; // TODO: Rename to lower bits

        self.response = (a & (1 << 7)) > 0;
        self.opcode = (a >> 3) & 0x0F; // TODO: can we instead shift 0x0F?
        self.authoritative_answer = (a & (1 << 2)) > 0;
        self.truncated_message = (a & (1 << 1)) > 0;
        self.recursion_desired = (a & 1) > 0;

        self.recursion_available = (b & (1 << 7)) > 0;
        self.z = (b & (1 << 6)) > 0;
        self.authenticated_data = (b & (1 << 5)) > 0;
        self.checking_disabled = (b & (1 << 4)) > 0;
        self.return_code = ReturnCode::from_num(b & 0x0F);

        self.questions_total = buffer.read_u16()?;
        self.answer_rr_total = buffer.read_u16()?;
        self.authoritative_rr_total = buffer.read_u16()?;
        self.additional_rr_total = buffer.read_u16()?;

        Ok(())
    }
}
