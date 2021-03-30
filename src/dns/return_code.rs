#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReturnCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
}

// TODO: maybe use num-derive crate instead
// https://enodev.fr/posts/rusticity-convert-an-integer-to-an-enum.html
impl ReturnCode {
    pub fn from_num(num: u8) -> Self {
        match num {
            1 => ReturnCode::FORMERR,
            2 => ReturnCode::SERVFAIL,
            3 => ReturnCode::NXDOMAIN,
            4 => ReturnCode::NOTIMP,
            5 => ReturnCode::REFUSED,
            _ => ReturnCode::NOERROR,
        }
    }
}
