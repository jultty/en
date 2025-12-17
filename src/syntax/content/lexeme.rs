use super::parsers::{line::Line, word::Word};

#[derive(Clone)]
pub enum Lexeme {
    Line(Line),
    Word(Word),
}

impl Lexeme {
    pub fn to_raw(&self) -> String {
        match *self {
            Lexeme::Line(ref d) => d.raw.clone(),
            Lexeme::Word(ref d) => d.raw.clone(),
        }
    }

    pub fn to_vec(self) -> Vec<String> {
        self.to_raw().split(' ').map(str::to_string).collect()
    }

    pub fn first(self) -> Option<String> {
        self.to_vec().first().map(String::to_owned)
    }
}
