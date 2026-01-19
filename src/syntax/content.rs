use std::mem::discriminant;

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

impl TokenOutput {
    pub fn only(&self, kind: &Token) -> Vec<Token> {
        let filter = |tokens: &[Token], k: &Token| -> Vec<Token> {
            tokens
                .iter()
                .filter(|&t| discriminant(t) == discriminant(k))
                .cloned()
                .collect::<Vec<Token>>()
        };

        let filtered_tokens = filter(&self.tokens, kind);
        let filtered_format_tokens = filter(&self.format_tokens, kind);

        [filtered_tokens, filtered_format_tokens]
            .into_iter()
            .flatten()
            .collect::<Vec<Token>>()
    }
}

pub fn parse(text: &str, graph: &Graph) -> String {
    parser::read(text, graph)
}

pub fn rich_parse(text: &str, graph: &Graph) -> TokenOutput {
    parser::rich_read(text, graph)
}

#[cfg(test)]
mod tests {
    use crate::syntax::content::parser::token::{Bold, Oblique};

    use super::*;

    #[test]
    fn only() {
        let graph = Graph::default();
        let output = rich_parse("*four* *bold* and _two_ italic", &graph);
        let bold_tokens = output.only(&Token::Bold(Bold::new(true)));
        let italic_tokens = output.only(&Token::Oblique(Oblique::new(true)));
        println!("{bold_tokens:?}");
        assert_eq!(bold_tokens.len(), 4);
        assert_eq!(italic_tokens.len(), 2);
    }
}
