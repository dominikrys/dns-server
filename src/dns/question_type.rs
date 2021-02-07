// TODO: see if all these traits are needed
#[derive(PartialEq, Eq, Debug, Clone, Hash, Copy)]
pub enum QuestionType {
    // TODO: can we assign values to all of these?
    UNKNOWN(u16), // TODO: does unknown need to take a value?
    A,            // 1
    NS,           // 2
    CNAME,        // 5
    MX,           // 15
    AAAA,         // 28
}

impl QuestionType {
    pub fn to_num(&self) -> u16 {
        // TODO: can we get rid of the *?
        // TODO: do we even need this?
        match *self {
            QuestionType::UNKNOWN(x) => x,
            QuestionType::A => 1,
            QuestionType::NS => 2,
            QuestionType::CNAME => 5,
            QuestionType::MX => 15,
            QuestionType::AAAA => 28,
        }
    }

    // TODO: maybe use num-derive crate instead
    // https://enodev.fr/posts/rusticity-convert-an-integer-to-an-enum.html
    pub fn from_num(num: u16) -> Self {
        match num {
            1 => QuestionType::A,
            2 => QuestionType::NS,
            5 => QuestionType::CNAME,
            15 => QuestionType::MX,
            28 => QuestionType::AAAA,
            _ => QuestionType::UNKNOWN(num),
        }
    }
}
