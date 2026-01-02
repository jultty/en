use std::collections::{HashMap};

use crate::types::Config;
use super::{Parseable as _, Token, LexMap};
use token::{
    anchor::Anchor, linebreak::LineBreak, paragraph::Paragraph, header::Header,
    preformat::PreFormat, literal::Literal, code::Code, oblique::Oblique,
};
use lexeme::Lexeme;

pub mod token;
pub mod lexeme;
pub mod segment;

const LEXMAP: LexMap = &[
    (LineBreak::probe, |word| {
        Token::LineBreak(LineBreak::lex(word))
    }),
    (Literal::probe, |word| Token::Literal(Literal::lex(word))),
];

fn lex(text: &str, map: LexMap, config: &Config) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut state = State::new();

    let segments = segment::segment(text);
    let lexemes = Lexeme::collect(&segments);

    let mut iterator = lexemes.iter().peekable();
    while let Some(lexeme) = iterator.next() {
        match state.context.block {
            BlockContext::None => {
                if PreFormat::probe(lexeme) {
                    state.context.block = BlockContext::PreFormat;
                    tokens.push(Token::PreFormat(PreFormat::new(true)));
                    continue;
                } else if Header::probe(lexeme) {
                    let mut header = Header::lex(lexeme);
                    header.dom_id = Some(Header::make_id(
                        config,
                        iterator.peek().map_or(&Lexeme::new("", ""), |l| l),
                        &mut state.dom_ids,
                    ));
                    state.context.block = BlockContext::Header(header.level());
                    tokens.push(Token::Header(header));
                    continue;
                } else if Paragraph::probe(lexeme) {
                    state.context.block = BlockContext::Paragraph;
                    tokens.push(Token::Paragraph(Paragraph::new(true)));
                }
            },
            BlockContext::PreFormat => {
                if PreFormat::probe(lexeme) {
                    tokens.push(Token::PreFormat(PreFormat::new(false)));
                    state.context.block = BlockContext::None;
                } else {
                    tokens.push(Token::Literal(Literal::lex(lexeme)));
                }
                continue;
            },
            BlockContext::Paragraph => {
                if lexeme.text() == "\n" {
                    tokens.push(Token::Paragraph(Paragraph::new(false)));
                    state.context.block = BlockContext::None;
                }
            },
            BlockContext::Header(n) => {
                if lexeme.text() == "\n" {
                    tokens.push(Token::Header(Header::from_u8(n, false, None)));
                    state.context.block = BlockContext::None;
                }
            },
        }

        match state.context.inline {
            InlineContext::None => {
                if Code::probe(lexeme) {
                    state.context.inline = InlineContext::Code;
                    tokens.push(Token::Code(Code::new(true)));
                    continue;
                } else if Oblique::probe(lexeme) {
                    state.context.inline = InlineContext::Oblique;
                    tokens.push(Token::Oblique(Oblique::new(true)));
                    continue;
                } else if Anchor::probe(lexeme) {
                    state.context.inline = InlineContext::Anchor;
                    state.buffers.anchor.clear();

                    if lexeme.match_first_char('|') {
                        state.buffers.anchor.candidate.leading = true;
                    } else {
                        state.buffers.anchor.candidate.text = lexeme.text();
                    }
                    continue;
                }
            },
            InlineContext::Code => {
                if Code::probe(lexeme) {
                    state.context.inline = InlineContext::None;
                    tokens.push(Token::Code(Code::new(false)));
                    continue;
                }
            },
            InlineContext::Oblique => {
                if Oblique::probe(lexeme) {
                    state.context.inline = InlineContext::None;
                    tokens.push(Token::Oblique(Oblique::new(false)));
                    continue;
                }
            },
            InlineContext::Anchor => {
                let buffer = &mut state.buffers.anchor;
                let candidate = &mut buffer.candidate;
                if candidate.text.is_empty() {
                    if lexeme.next() == "|" {
                        buffer.text.push_str(&lexeme.text());
                        candidate.text.clone_from(&buffer.text);
                    } else {
                        buffer.text.push_str(&lexeme.text());
                    }
                    continue;
                } else if candidate.destination.is_none() {
                    // candidate is leading and we found the second pipe
                    if candidate.leading && lexeme.text() == "|" {
                        // third pipe immediately after second: forcing flanking
                        if lexeme.match_next_first_char('|') {
                            candidate.destination =
                                Some(candidate.text.clone());
                            let token = Token::Anchor(candidate.clone());
                            tokens.push(token);
                            state.context.inline = InlineContext::None;
                            iterator.next();
                            continue;
                        // whitespace or punctuation after pipe: flanking anchor
                        } else if lexeme.is_next_whitespace()
                            || lexeme.is_next_punctuation()
                        {
                            candidate.destination =
                                Some(candidate.text.clone());
                            let token = Token::Anchor(candidate.clone());
                            tokens.push(token);
                            state.context.inline = InlineContext::None;
                        // non-whitespace after pipe is the destination
                        } else {
                            candidate.destination = Some(lexeme.next().clone());
                            let token = Token::Anchor(candidate.clone());
                            tokens.push(token);
                            state.context.inline = InlineContext::None;
                            // if there is a trailing pipe, consume it
                            if let Some(next) = iterator.next()
                                && next.next() == "|"
                            {
                                iterator.next();
                            }
                        }
                    // candidate is nonleading and we found a second pipe
                    } else if !candidate.leading && lexeme.next() == "|" {
                        candidate.destination = Some(lexeme.text());
                        tokens.push(Token::Anchor(candidate.clone()));
                        state.context.inline = InlineContext::None;
                        iterator.next();
                    // candidate is nonleading and we found whitespace
                    } else if lexeme.is_next_whitespace() {
                        candidate.destination = Some(lexeme.text());
                        let token = Token::Anchor(candidate.clone());
                        tokens.push(token);
                        state.context.inline = InlineContext::None;
                    // candidate is nonleading and we haven't found whitespace
                    } else {
                        buffer.destination.push_str(&lexeme.text());
                    }
                    continue;
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

    close(&state, &mut tokens);
    tokens
}

fn close(state: &State, tokens: &mut Vec<Token>) {
    match state.context.block {
        BlockContext::PreFormat => {
            tokens.push(Token::PreFormat(PreFormat::new(false)));
        },
        BlockContext::Paragraph => {
            tokens.push(Token::Paragraph(Paragraph::new(false)));
        },
        BlockContext::Header(_) => panic!("End of file with open header"),
        BlockContext::None => (),
    }
}

enum BlockContext {
    Paragraph,
    Header(u8),
    PreFormat,
    None,
}

enum InlineContext {
    Anchor,
    Code,
    Oblique,
    None,
}

struct State {
    context: Context,
    dom_ids: HashMap<String, Vec<String>>,
    buffers: Buffers,
}

struct Context {
    block: BlockContext,
    inline: InlineContext,
}

struct Buffers {
    anchor: AnchorBuffer,
}

#[derive(Debug)]
struct AnchorBuffer {
    candidate: Anchor,
    text: String,
    destination: String,
}

impl AnchorBuffer {
    fn clear(&mut self) {
        self.candidate = Anchor::empty();
        self.text = String::new();
        self.destination = String::new();
    }
}

impl State {
    fn new() -> State {
        State {
            context: Context {
                inline: InlineContext::None,
                block: BlockContext::None,
            },
            dom_ids: HashMap::new(),
            buffers: Buffers {
                anchor: AnchorBuffer {
                    candidate: Anchor::empty(),
                    text: String::new(),
                    destination: String::new(),
                },
            },
        }
    }
}

fn parse(tokens: &[Token]) -> String {
    tokens.iter().map(Token::render).collect::<String>()
}

pub(super) fn read(text: &str, config: &Config) -> String {
    parse(&lex(text, LEXMAP, config))
}

#[cfg(test)]
mod tests {
    use crate::{types::Graph, syntax::content::parser::token::header::Level};

    use super::*;

    fn read_noconfig(input: &str) -> String {
        read(input, &Graph::new(None).meta.config)
    }

    #[test]
    fn empty_render_is_empty() {
        assert_eq!(read_noconfig(""), "");
    }

    #[test]
    fn mixed_sample() {
        let en = "`this |test|` tries ## to |brea|k|: things";
        let html = r#"<p><code>this |test|</code> tries ## to <a href="/node/k">brea</a>: things</p>"#;

        assert_eq!(read_noconfig(en), html);
    }

    #[test]
    fn force_flanking() {
        assert_eq!(
            read_noconfig("|Node||"),
            r#"<p><a href="/node/Node">Node</a></p>"#
        );
    }

    #[test]
    fn force_flanking_with_trailing_letter() {
        assert_eq!(
            read_noconfig("|Node||s"),
            r#"<p><a href="/node/Node">Node</a>s</p>"#
        );
    }

    #[test]
    fn flanking_with_trailing_pipe() {
        assert_eq!(
            read_noconfig("|Node|Destination|"),
            r#"<p><a href="/node/Destination">Node</a></p>"#
        );
    }

    #[test]
    fn nonleading_second_pipe() {
        assert_eq!(
            read_noconfig("Go to Node|Destination|, here"),
            r#"<p>Go to <a href="/node/Destination">Node</a>, here</p>"#,
        );
    }

    #[test]
    fn clear_anchor_buffer() {
        assert_eq!(
            read_noconfig("|SomeAnchor|\n|SomeOtherAnchor|"),
            concat!(
                r#"<p><a href="/node/SomeAnchor">SomeAnchor</a></p>"#,
                "\n",
                r#"<p><a href="/node/SomeOtherAnchor">SomeOtherAnchor</a></p>"#
            ),
        );
    }

    #[test]
    fn pre() {
        let payload = "D0qdJ184f3q1okbYu3Xm1d93jj6jy615";
        assert_eq!(
            read_noconfig(&format!("`\n{payload}\n`\n")),
            format!("<pre>\n{payload}\n</pre>\n"),
        );
    }

    #[test]
    fn eof_pre() {
        let payload = "Jp8INpWzsQmk20jpIhBFCfMUXOztxv0w";
        assert_eq!(
            read_noconfig(&format!("`\n{payload}\n`")),
            format!("<pre>\n{payload}\n</pre>"),
        );
    }

    #[test]
    #[should_panic(expected = "End of file with open header")]
    fn end_with_open_header() {
        let default_state = State::new();
        let state = State {
            context: Context {
                block: BlockContext::Header(1),
                ..default_state.context
            },
            ..default_state
        };

        close(&state, &mut vec![]);
    }

    #[test]
    fn end_with_open_preformat() {
        let mut state = State::new();
        state.context.block = BlockContext::PreFormat;

        let mut vec: Vec<Token> = vec![];
        close(&state, &mut vec);
        assert_eq!(vec, vec![Token::PreFormat(PreFormat::new(false))]);
    }

    #[test]
    fn truncated_header_level() {
        let u: usize = 999;
        let level = Level::from(u);
        assert_eq!(level.to_string(), "6");
    }

    #[test]
    fn display_level() {
        assert_eq!(format!("{}", Level::One), "1");
        assert_eq!(format!("{}", Level::Two), "2");
        assert_eq!(format!("{}", Level::Three), "3");
        assert_eq!(format!("{}", Level::Four), "4");
        assert_eq!(format!("{}", Level::Five), "5");
        assert_eq!(format!("{}", Level::Six), "6");
    }
}
