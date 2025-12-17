use crate::syntax::content::parsers::word::elements::literal::Literal;
use crate::syntax::content::{Parseable, Token, Word, LexMap, make_lexmap};
use crate::syntax::content::lexeme::Lexeme;

const LEXMAP: LexMap =
    &[(Literal::probe, |line| Token::Literal(Literal::lex(line)))];

pub(in crate::syntax::content) fn read<DefaultToken: Parseable>(
    text: &str,
) -> String {
    parse(&lex(text, &make_lexmap::<DefaultToken>(LEXMAP)))
}

fn lex(text: &str, map: LexMap) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    for raw_word in text.split(" ") {
        let word = Lexeme::Word(Word::new(raw_word));

        for &(ref probe, lex) in map {
            if probe(&word) {
                tokens.push(lex(&word));
                break;
            }
        }
    }

    tokens
}

fn parse(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(Token::render)
        .collect::<Vec<_>>()
        .join(" ")
}
