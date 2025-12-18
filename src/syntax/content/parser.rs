use std::slice::Iter;

use crate::prelude::*;
use super::{Parseable as _, Token, LexMap};
use token::{
    anchor::Anchor, linebreak::LineBreak, paragraph::Paragraph, header::Header,
    preformat::PreFormat, literal::Literal,
};
use lexeme::{Lexeme, compound::Compound};

pub mod token;
pub mod lexeme;

const LEXMAP: LexMap = &[
    (Anchor::probe, |word| Token::Anchor(Anchor::lex(word))),
    (LineBreak::probe, |word| {
        Token::LineBreak(LineBreak::lex(word))
    }),
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

    let splits = split(text);
    let mut iter = splits.iter();
    while let Some(word) = iter.next() {
        let compound = cluster(word, &mut iter);
        let lexeme = Lexeme::Compound(compound);

        match state {
            Context::None => {
                if Header::probe(&lexeme) {
                    let header = Header::lex(&lexeme);
                    state = Context::Header(header.get_level());
                    tokens.push(Token::Header(header));
                    continue;
                } else if PreFormat::probe(&lexeme) {
                    tokens.push(Token::PreFormat(PreFormat::new(true)));
                    state = Context::PreFormat;
                    continue;
                } else if Paragraph::probe(&lexeme) {
                    tokens.push(Token::Paragraph(Paragraph::new(true)));
                    state = Context::Paragraph;
                }
            },
            Context::Paragraph => {
                if word == "\n" {
                    tokens.push(Token::Paragraph(Paragraph::new(false)));
                    state = Context::None;
                }
            },
            Context::Header(n) => {
                if word == "\n" {
                    tokens.push(Token::Header(Header::from_u8(n, false)));
                    state = Context::None;
                }
            },
            Context::PreFormat => {
                if PreFormat::probe(&lexeme) {
                    tokens.push(Token::PreFormat(PreFormat::new(false)));
                    state = Context::None;
                    continue;
                }
            },
        }

        for &(ref probe, lex) in map {
            if probe(&lexeme) {
                tokens.push(lex(&lexeme));
                break;
            }
        }
    }

    tokens
}

fn split(text: &str) -> Vec<String> {
    text.replace("\n", " \n ")
        .split(' ')
        .map(str::to_string)
        .collect()
}

// this could be eliminated if space were a token
fn join<'i, Iterator>(rendered_tokens: Iterator) -> String
where
    Iterator: IntoIterator<Item = &'i str>,
{
    fn stick(current: &str, next: &str) -> bool {
        // this could be in a dedicated type
        fn is_tag(s: &str) -> bool {
            s.starts_with("<") && s.ends_with('>')
        }
        fn is_opening(s: &str) -> bool {
            is_tag(s) && !s.contains("</")
        }
        fn is_closing(s: &str) -> bool {
            is_tag(s) && s.contains("</")
        }
        fn is_inline(s: &str) -> bool {
            is_tag(s) && s.starts_with("<a")
        }

        log!("On {current}[?]{next}");
        if is_inline(next) {
            log!("Pushing space because {next} is inline");
            false
        } else if is_closing(next) {
            log!("Not pushing space because {next} is closing");
            true
        } else if is_opening(current) {
            log!("Not pushing space because {current} is opening");
            true
        } else {
            false
        }
    }

    let mut iterator = rendered_tokens.into_iter();
    let mut out_string = String::new();

    if let Some(mut current) = iterator.next() {
        out_string.push_str(current);
        for next in iterator {
            if stick(current, next) {
                out_string.push_str(next);
            } else {
                out_string.push(' ');
                out_string.push_str(next);
            }
            current = next;
        }
    }
    out_string
}

fn parse(tokens: &[Token]) -> String {
    let rendered: Vec<String> = tokens.iter().map(Token::render).collect();

    join(rendered.iter().map(String::as_str))
}

fn cluster<'c>(word: &str, iter: &mut Iter<'c, String>) -> Compound {
    if word.starts_with('|') {
        log!("Found opener {word}");
        let mut parts = vec![word];

        if let Some(first) = parts.first()
            && first.ends_with('|')
        {
            log!("Returning atomic cluster");
            Compound::new(&parts.join(" "))
        } else {
            log!("Seeking a boundary");
            for next_raw in iter {
                if next_raw.contains('|') {
                    log!("Found end of cluster {next_raw:?}");
                    parts.push(next_raw);
                    break;
                } else {
                    parts.push(next_raw);
                    log!("Onto next word from {next_raw}");
                }
            }
            log!("Returning cluster {parts:?}");

            Compound::new(&parts.join(" "))
        }
    } else {
        Compound::new(word)
    }
}

pub(super) fn read(text: &str) -> String {
    parse(&lex(text, LEXMAP))
}
