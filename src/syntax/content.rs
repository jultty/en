use token::{Token};
use parsers::{line::Line, word::Word};
use lexeme::Lexeme;

mod token;
pub mod lexeme;
pub mod parsers;

pub trait Parseable: Into<Token> {
    fn probe(lexeme: &Lexeme) -> bool;
    fn lex(lexeme: &Lexeme) -> Self;
    fn render(&self) -> String;
}

type Probe = fn(&Lexeme) -> bool;
type Lexer = fn(&Lexeme) -> Token;
type LexMap<'lm> = &'lm [(Probe, Lexer)];

fn make_lexmap<DefaultToken: Parseable>(base: LexMap) -> Vec<(Probe, Lexer)> {
    let mut vector: Vec<(Probe, Lexer)> = base.to_vec();

    fn adapter<D: Parseable>(lex: &Lexeme) -> Token {
        D::lex(lex).into()
    }

    vector.push((DefaultToken::probe, adapter::<DefaultToken>));
    vector
}

pub fn parse<DefaultLineToken: Parseable, DefaultWordToken: Parseable>(
    text: &str,
) -> String {
    let escaped_text = tera::escape_html(text);
    parsers::line::parser::read::<DefaultLineToken>(
        &parsers::word::parser::read::<DefaultWordToken>(&escaped_text),
    )
}
