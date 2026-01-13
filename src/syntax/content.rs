use parser::{Token, Lexeme};

use crate::graph::Graph;

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

pub struct TokenOutput {
    pub text: Option<String>,
    pub tokens: Vec<Token>,
    pub format_tokens: Vec<Token>,
}

pub fn parse(text: &str, graph: &Graph) -> String {
    parser::read(text, graph)
}

pub fn rich_parse(text: &str, graph: &Graph) -> TokenOutput {
    parser::rich_read(text, graph)
}
