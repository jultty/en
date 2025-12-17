use crate::syntax::content::lexeme::Lexeme;

pub mod parser;
pub mod elements;

#[derive(Clone)]
pub struct Line {
    pub raw: String,
    pub first: String,
}

impl Line {
    pub fn new(text: &str) -> Line {
        let vec: Vec<&str> = text.split(" ").collect();

        Line {
            raw: text.to_owned(),
            first: vec.first().unwrap_or_else(|| unreachable!()).to_string(),
        }
    }
}

impl From<Lexeme> for Line {
    fn from(lexeme: Lexeme) -> Line {
        match lexeme {
            Lexeme::Word(w) => Line {
                raw: w.raw.clone(),
                first: w.raw.split(' ').next().unwrap_or_default().to_owned(),
            },
            Lexeme::Line(l) => l,
        }
    }
}
