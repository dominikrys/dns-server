// NOTE: ideally we would use the num-derive crate and we'd provide explicit discriminants
// to the enum from nightly Rust (RFC 2363), but the features don't inter-work. Using nested
// enums and num-derive also ended up being quite convoluted, so the code is a bit verbose.

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u16)]
pub enum QueryType {
    UNKNOWN(u16),
    A,
    NS,
    CNAME,
    MX,
    AAAA,
}

impl QueryType {
    pub fn to_num(&self) -> u16 {
        match *self {
            Self::A => 1,
            Self::NS => 2,
            Self::CNAME => 5,
            Self::MX => 15,
            Self::AAAA => 28,
            Self::UNKNOWN(num) => num,
        }
    }

    pub fn from_num(num: u16) -> Self {
        match num {
            1 => Self::A,
            2 => Self::NS,
            5 => Self::CNAME,
            15 => Self::MX,
            28 => Self::AAAA,
            _ => Self::UNKNOWN(num),
        }
    }
}
