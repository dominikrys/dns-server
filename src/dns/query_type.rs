#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum QueryType {
    // TODO: can we assign values to all of these?
    UNKNOWN(u16), // TODO: does unknown need to take a value?
    A,            // 1
    NS,           // 2
    CNAME,        // 5
    MX,           // 15
    AAAA,         // 28
}

impl QueryType {
    pub fn to_num(&self) -> u16 {
        // TODO: can we get rid of the *?
        // TODO: do we even need this?
        match *self {
            QueryType::UNKNOWN(x) => x,
            QueryType::A => 1,
            QueryType::NS => 2,
            QueryType::CNAME => 5,
            QueryType::MX => 15,
            QueryType::AAAA => 28,
        }
    }

    // TODO: maybe use num-derive crate instead
    // https://enodev.fr/posts/rusticity-convert-an-integer-to-an-enum.html
    pub fn from_num(num: u16) -> Self {
        match num {
            1 => QueryType::A,
            2 => QueryType::NS,
            5 => QueryType::CNAME,
            15 => QueryType::MX,
            28 => QueryType::AAAA,
            _ => QueryType::UNKNOWN(num),
        }
    }
}
