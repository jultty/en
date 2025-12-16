use elements::{paragraph::Paragraph, header::Header};

mod elements;
pub mod parser;

enum Token {
    Paragraph(Paragraph),
    Header(Header),
}

struct Lexeme<'l> {
    pub raw: &'l str,
    pub first: &'l str,
}

impl<'l> Lexeme<'l> {
    pub fn new(text: &'l str) -> Lexeme<'l> {
        let vec: Vec<&'l str> = text.split(" ").collect();

        Self {
            raw: text,
            first: vec.first().unwrap_or_else(|| unreachable!()),
        }
    }
}

trait Parseable {
    fn probe(lexeme: &Lexeme) -> bool;
    fn lex(lexeme: &Lexeme) -> Self
    where
        Self: Sized;
    fn render(&self) -> String;
}

type Matcher = fn(&Lexeme) -> bool;
type Constructor = fn(&Lexeme) -> Token;

static LEXMAP: &[(Matcher, Constructor)] = &[
    (Header::probe, |lexeme| Token::Header(Header::lex(lexeme))),
    (Paragraph::probe, |lexeme| {
        Token::Paragraph(Paragraph::lex(lexeme))
    }),
];
