use super::packet_buffer::PacketBuffer;
use super::return_code::ReturnCode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Reference: http://www.networksorcery.com/enp/protocol/dns.htm

#[derive(Clone, Debug)]
pub struct Header {
    pub id: u16,

    pub response: bool,
    pub opcode: u8, // 4 bits
    pub authoritative_answer: bool,
    pub truncated_message: bool,
    pub recursion_desired: bool,

    pub recursion_available: bool,
    pub z: bool,
    pub authenticated_data: bool,
    pub checking_disabled: bool,
    pub return_code: ReturnCode, // 4 bits

    pub queries_total: u16,
    pub answer_rr_total: u16,
    pub authoritative_rr_total: u16,
    pub additional_rr_total: u16,
}

impl Header {
    pub fn new() -> Self {
        Header {
            id: 0,

            response: false,
            opcode: 0,
            authoritative_answer: false,
            truncated_message: false,
            recursion_desired: false,

            recursion_available: false,
            z: false,
            authenticated_data: false,
            checking_disabled: false,
            return_code: ReturnCode::NOERROR,

            queries_total: 0,
            answer_rr_total: 0,
            authoritative_rr_total: 0,
            additional_rr_total: 0,
        }
    }

    // TODO: have flags_half_1 constructor for this - maybe call b1 and b2
    pub fn from_buffer(buffer: &mut PacketBuffer) -> Result<Self> {
        // NOTE: buffer pos must be at the start of the header

        let mut header = Self::new();

        header.id = buffer.read_u16()?;

        let flags = buffer.read_u16()?;
        let flags_half_1 = (flags >> 8) as u8;
        let flags_half_2 = (flags & 0xFF) as u8;

        header.response = (flags_half_1 & (1 << 7)) > 0;
        header.opcode = (flags_half_1 >> 3) & 0x0F; // TODO: can we instead shift 0x0F?
        header.authoritative_answer = (flags_half_1 & (1 << 2)) > 0;
        header.truncated_message = (flags_half_1 & (1 << 1)) > 0;
        header.recursion_desired = (flags_half_1 & 1) > 0;

        header.recursion_available = (flags_half_2 & (1 << 7)) > 0;
        header.z = (flags_half_2 & (1 << 6)) > 0;
        header.authenticated_data = (flags_half_2 & (1 << 5)) > 0;
        header.checking_disabled = (flags_half_2 & (1 << 4)) > 0;
        header.return_code = ReturnCode::from_num(flags_half_2 & 0x0F);

        header.queries_total = buffer.read_u16()?;
        header.answer_rr_total = buffer.read_u16()?;
        header.authoritative_rr_total = buffer.read_u16()?;
        header.additional_rr_total = buffer.read_u16()?;

        Ok(header)
    }

    // TODO: return the buffer and dont take it as an argument!
    pub fn write(&self, buffer: &mut PacketBuffer) -> Result<()> {
        buffer.write_u16(self.id)?;

        // TODO: make this and the other bit manipulations more legible?
        let flags_half_1 = (((self.response as u8) << 7)
            | (self.opcode << 3)
            | ((self.authoritative_answer as u8) << 2)
            | ((self.truncated_message as u8) << 1)
            | (self.recursion_desired as u8)) as u8;

        buffer.write_u8(flags_half_1)?;

        let flags_half_2 = (((self.recursion_available as u8) << 7)
            | ((self.z as u8) << 6)
            | ((self.authenticated_data as u8) << 5)
            | ((self.checking_disabled as u8) << 4)
            | (self.return_code as u8)) as u8;

        buffer.write_u8(flags_half_2)?;

        buffer.write_u16(self.queries_total)?;
        buffer.write_u16(self.answer_rr_total)?;
        buffer.write_u16(self.authoritative_rr_total)?;
        buffer.write_u16(self.additional_rr_total)?;

        Ok(())
    }
}
