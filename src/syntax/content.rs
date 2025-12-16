use elements::{header::Header};
use units::{Token, Lexeme};

mod units;
pub mod elements;
pub mod parser;

pub trait Parseable: Into<Token> {
    fn probe(lexeme: &Lexeme) -> bool;
    fn lex(lexeme: &Lexeme) -> Self;
    fn render(&self) -> String;
}

type Probe = fn(&Lexeme) -> bool;
type Lexer = fn(&Lexeme) -> Token;
type LexEntry = (Probe, Lexer);
type LexMap<'lm> = &'lm [LexEntry];

const LEXMAP: LexMap =
    &[(Header::probe, |lexeme| Token::Header(Header::lex(lexeme)))];

fn make_lexmap<DefaultToken: Parseable>() -> Vec<LexEntry> {
    let mut vector: Vec<(Probe, Lexer)> = LEXMAP.to_vec();

    fn adapter<D: Parseable>(lex: &Lexeme) -> Token {
        D::lex(lex).into()
    }

    vector.push((DefaultToken::probe, adapter::<DefaultToken>));
    vector
}
