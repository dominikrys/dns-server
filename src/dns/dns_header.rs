use super::packet_buffer::PacketBuffer;
use super::return_code::ReturnCode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Reference: http://www.networksorcery.com/enp/protocol/dns.htm
// TODO: maybe rename to "header"
#[derive(Clone, Debug)]
pub struct DnsHeader {
    // TODO: do all these need to be pub?
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

    pub queries_total: u16,          // 16 bits
    pub answer_rr_total: u16,        // 16 bits
    pub authoritative_rr_total: u16, // 16 bits
    pub additional_rr_total: u16,    // 16 bits
}

impl DnsHeader {
    pub fn new() -> Self {
        // TODO: maybe change to default instead: https://users.rust-lang.org/t/default-and-optional-parameter/27693/4
        // TODO: Also check if this would be useful anywhere else in the code
        DnsHeader {
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

    // TODO: rename this - this is more like creation from buffer
    // TODO: have flags_half_1 constructor for this, and not return an input
    pub fn read_u8(&mut self, buffer: &mut PacketBuffer) -> Result<()> {
        // TODO: reset buffer pos?
        self.id = buffer.read_u16()?;

        let flags = buffer.read_u16()?;
        let flags_half_1 = (flags >> 8) as u8;
        let flags_half_2 = (flags & 0xFF) as u8;

        self.response = (flags_half_1 & (1 << 7)) > 0;
        self.opcode = (flags_half_1 >> 3) & 0x0F; // TODO: can we instead shift 0x0F?
        self.authoritative_answer = (flags_half_1 & (1 << 2)) > 0;
        self.truncated_message = (flags_half_1 & (1 << 1)) > 0;
        self.recursion_desired = (flags_half_1 & 1) > 0;

        self.recursion_available = (flags_half_2 & (1 << 7)) > 0;
        self.z = (flags_half_2 & (1 << 6)) > 0;
        self.authenticated_data = (flags_half_2 & (1 << 5)) > 0;
        self.checking_disabled = (flags_half_2 & (1 << 4)) > 0;
        self.return_code = ReturnCode::from_num(flags_half_2 & 0x0F);

        self.queries_total = buffer.read_u16()?;
        self.answer_rr_total = buffer.read_u16()?;
        self.authoritative_rr_total = buffer.read_u16()?;
        self.additional_rr_total = buffer.read_u16()?;

        Ok(())
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
