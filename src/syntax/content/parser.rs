use super::{Parseable as _, Token, LexMap};
use token::{
    anchor::Anchor, linebreak::LineBreak, paragraph::Paragraph, header::Header,
    preformat::PreFormat, literal::Literal, code::Code,
};
use lexeme::Lexeme;

pub mod token;
pub mod lexeme;
pub mod cluster;

const LEXMAP: LexMap = &[
    (LineBreak::probe, |word| {
        Token::LineBreak(LineBreak::lex(word))
    }),
    (Code::probe, |word| Token::Code(Code::lex(word))),
    (Anchor::probe, |word| Token::Anchor(Anchor::lex(word))),
    (Literal::probe, |word| Token::Literal(Literal::lex(word))),
];

enum Context {
    None,
    Paragraph,
    Header(u8),
    PreFormat,
}

fn lex(text: &str, map: LexMap) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut state = Context::None;

    let splits = cluster::cluster(text);
    let lexemes = Lexeme::collect(&splits);
    let iter = lexemes.iter().peekable();
    for lexeme in iter {
        match state {
            Context::None => {
                if PreFormat::probe(lexeme) {
                    tokens.push(Token::PreFormat(PreFormat::new(true)));
                    state = Context::PreFormat;
                    continue;
                } else if Header::probe(lexeme) {
                    let header = Header::lex(lexeme);
                    state = Context::Header(header.get_level());
                    tokens.push(Token::Header(header));
                    continue;
                } else if Paragraph::probe(lexeme) {
                    tokens.push(Token::Paragraph(Paragraph::new(true)));
                    state = Context::Paragraph;
                }
            },
            Context::PreFormat => {
                if PreFormat::probe(lexeme) {
                    tokens.push(Token::PreFormat(PreFormat::new(false)));
                    state = Context::None;
                } else {
                    tokens.push(Token::Literal(Literal::lex(lexeme)));
                }
                continue;
            },
            Context::Paragraph => {
                if lexeme.text() == "\n" {
                    tokens.push(Token::Paragraph(Paragraph::new(false)));
                    state = Context::None;
                }
            },
            Context::Header(n) => {
                if lexeme.text() == "\n" {
                    tokens.push(Token::Header(Header::from_u8(n, false)));
                    state = Context::None;
                }
            },
        }

        for &(ref probe, lex) in map {
            if probe(lexeme) {
                tokens.push(lex(lexeme));
                break;
            }
        }
    }

    tokens
}

fn parse(tokens: &[Token]) -> String {
    tokens.iter().map(Token::render).collect::<String>()
}

pub(super) fn read(text: &str) -> String {
    parse(&lex(text, LEXMAP))
}
