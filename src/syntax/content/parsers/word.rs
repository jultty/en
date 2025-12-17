pub mod parser;
pub mod elements;

#[derive(Clone)]
pub struct Word {
    pub raw: String,
}

impl Word {
    pub fn new(text: &str) -> Word {
        Word {
            raw: text.to_owned(),
        }
    }
}
