// TODO: see if all these traits are needed
#[derive(PartialEq, Eq, Debug, Clone, Hash, Copy)]
pub enum QuestionType {
    // TODO: can we assign values to all of these?
    UNKNOWN(u16), // TODO: does unknown need to take a value?
    A,            // 1
}

impl QuestionType {
    pub fn to_num(&self) -> u16 {
        // TODO: can we get rid of the *?
        // TODO: do we even need this?
        match *self {
            QuestionType::UNKNOWN(x) => x,
            QuestionType::A => 1,
        }
    }

    // TODO: maybe use num-derive crate instead
    // https://enodev.fr/posts/rusticity-convert-an-integer-to-an-enum.html
    pub fn from_num(num: u16) -> Self {
        match num {
            1 => QuestionType::A,
            _ => QuestionType::UNKNOWN(num),
        }
    }
}
