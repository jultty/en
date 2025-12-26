use parser::{token::Token, lexeme::Lexeme};

pub mod parser;

pub trait Parseable {
    fn probe(lexeme: &Lexeme) -> bool;
    fn lex(lexeme: &Lexeme) -> Self;
    fn render(&self) -> String;
}

type Probe = fn(&Lexeme) -> bool;
type Lexer = fn(&Lexeme) -> Token;
type LexMap<'lm> = &'lm [(Probe, Lexer)];

pub fn parse(text: &str) -> String {
    parser::read(text)
}
