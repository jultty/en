use parser::{token::Token, lexeme::Lexeme};

use crate::types::Config;

pub mod parser;

pub trait Parseable: std::fmt::Display {
    fn probe(lexeme: &Lexeme) -> bool;
    fn lex(lexeme: &Lexeme) -> Self;
    fn render(&self) -> String;
}

type Probe = fn(&Lexeme) -> bool;
type Lexer = fn(&Lexeme) -> Token;
type LexMap<'lm> = &'lm [(Probe, Lexer)];

pub fn parse(text: &str, config: &Config) -> String {
    parser::read(text, config)
}
