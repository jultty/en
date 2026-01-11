use parser::{token::Token, lexeme::Lexeme};

use crate::types::Graph;

pub mod parser;

pub trait Parseable: std::fmt::Display {
    fn probe(lexeme: &Lexeme) -> bool;
    fn lex(lexeme: &Lexeme) -> Self;
    fn render(&self) -> String;
    fn flatten(&self) -> String;
}

type Probe = fn(&Lexeme) -> bool;
type Lexer = fn(&Lexeme) -> Token;
type LexMap<'lm> = &'lm [(Probe, Lexer)];

pub fn parse(text: &str, graph: &Graph) -> String {
    parser::read(text, graph)
}
