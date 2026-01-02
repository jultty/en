use std::collections::{HashMap};

use crate::types::Config;
use super::{Parseable as _, Token, LexMap};
use token::{
    anchor::Anchor, linebreak::LineBreak, paragraph::Paragraph, header::Header,
    preformat::PreFormat, literal::Literal, code::Code, oblique::Oblique,
};
use lexeme::Lexeme;
use context::{Context, Block, Inline};

pub mod token;
pub mod lexeme;
pub mod segment;
pub mod context;

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
            Block::None => {
                if PreFormat::probe(lexeme) {
                    state.context.block = Block::PreFormat;
                    tokens.push(Token::PreFormat(PreFormat::new(true)));
                    continue;
                } else if Header::probe(lexeme) {
                    let mut header = Header::lex(lexeme);
                    header.dom_id = Some(Header::make_id(
                        config,
                        iterator.peek().map_or(&Lexeme::new("", ""), |l| l),
                        &mut state.dom_ids,
                    ));
                    state.context.block = Block::Header(header.level());
                    tokens.push(Token::Header(header));
                    continue;
                } else if Paragraph::probe(lexeme) {
                    state.context.block = Block::Paragraph;
                    tokens.push(Token::Paragraph(Paragraph::new(true)));
                }
            },
            Block::PreFormat => {
                if PreFormat::probe(lexeme) {
                    tokens.push(Token::PreFormat(PreFormat::new(false)));
                    state.context.block = Block::None;
                } else {
                    tokens.push(Token::Literal(Literal::lex(lexeme)));
                }
                continue;
            },
            Block::Paragraph => {
                if lexeme.text() == "\n" {
                    tokens.push(Token::Paragraph(Paragraph::new(false)));
                    state.context.block = Block::None;
                }
            },
            Block::Header(n) => {
                if lexeme.text() == "\n" {
                    tokens.push(Token::Header(Header::from_u8(n, false, None)));
                    state.context.block = Block::None;
                }
            },
        }

        match state.context.inline {
            Inline::None => {
                if Code::probe(lexeme) {
                    state.context.inline = Inline::Code;
                    tokens.push(Token::Code(Code::new(true)));
                    continue;
                } else if Anchor::probe(lexeme) {
                    state.context.inline = Inline::Anchor;
                    state.buffers.anchor.clear();

                    if lexeme.match_first_char('|') {
                        state.buffers.anchor.candidate.leading = true;
                    } else {
                        state.buffers.anchor.candidate.text = lexeme.text();
                    }
                    continue;
                } else if Oblique::probe(lexeme) {
                    state.context.inline = Inline::Oblique;
                    tokens.push(Token::Oblique(Oblique::new(true)));
                    continue;
                }
            },
            Inline::Code => {
                if Code::probe(lexeme) {
                    state.context.inline = Inline::None;
                    tokens.push(Token::Code(Code::new(false)));
                    continue;
                }
            },
            Inline::Oblique => {
                if Oblique::probe(lexeme) {
                    state.context.inline = Inline::None;
                    tokens.push(Token::Oblique(Oblique::new(false)));
                    continue;
                }
            },
            Inline::Anchor => {
                if context::anchor::parse(
                    lexeme,
                    &mut iterator,
                    &mut state,
                    &mut tokens,
                ) {
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

    context::close(&state, &mut tokens);
    tokens
}

#[derive(Clone, Debug)]
pub struct State {
    context: Context,
    dom_ids: HashMap<String, Vec<String>>,
    buffers: Buffers,
}

#[derive(Clone, Debug)]
struct Buffers {
    anchor: AnchorBuffer,
}

#[derive(Clone, Debug)]
struct AnchorBuffer {
    candidate: Anchor,
    text: String,
    destination: String,
}

impl AnchorBuffer {
    fn clear(&mut self) {
        self.candidate = Anchor::default();
        self.text = String::new();
        self.destination = String::new();
    }
}

impl State {
    fn new() -> State {
        State {
            context: Context {
                inline: Inline::None,
                block: Block::None,
            },
            dom_ids: HashMap::new(),
            buffers: Buffers {
                anchor: AnchorBuffer {
                    candidate: Anchor::default(),
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
    fn anchor_to_node_s() {
        assert_eq!(
            read_noconfig("The |letter s|s|'s node: |s|!"),
            r#"<p>The <a href="/node/s">letter s</a>'s node: <a href="/node/s">s</a>!</p>"#
        );
    }

    #[test]
    fn nonleading_plural_anchor() {
        assert_eq!(
            read_noconfig("The flower|s bloomed"),
            r#"<p>The <a href="/node/flower">flowers</a> bloomed</p>"#
        );
    }

    #[test]
    fn leading_plural_anchor() {
        assert_eq!(
            read_noconfig("Interfaces are |element|s of |system|s."),
            r#"<p>Interfaces are <a href="/node/element">elements</a> of <a href="/node/system">systems</a>.</p>"#
        );
    }

    #[test]
    fn http_external_anchor() {
        assert_eq!(
            read_noconfig(
                "a |false dichotomy|https://en.wikipedia.org/wiki/False_dilemma|."
            ),
            r#"<p>a <a href="https://en.wikipedia.org/wiki/False_dilemma">false dichotomy</a>.</p>"#
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third() {
        assert_eq!(
            read_noconfig("|Rust toolchain|https://rustup.rs/ "),
            r#"<p><a href="https://rustup.rs/">Rust toolchain</a> </p>"#
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_punctuation_then_space() {
        assert_eq!(
            read_noconfig("|Rust toolchain|https://rustup.rs/, "),
            r#"<p><a href="https://rustup.rs/">Rust toolchain</a>, </p>"#
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_punctuation_then_eof() {
        assert_eq!(
            read_noconfig("|Rust toolchain|https://rustup.rs/,"),
            r#"<p><a href="https://rustup.rs/">Rust toolchain</a></p>"#
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_eof() {
        assert_eq!(
            read_noconfig("|Rust toolchain|https://rustup.rs/"),
            r#"<p><a href="https://rustup.rs/">Rust toolchain</a></p>"#
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
    #[should_panic(expected = "End of input with open header")]
    fn end_with_open_header() {
        let default_state = State::new();
        let state = State {
            context: Context {
                block: Block::Header(1),
                ..default_state.context
            },
            ..default_state
        };

        context::close(&state, &mut vec![]);
    }

    #[test]
    fn end_with_open_preformat() {
        let mut state = State::new();
        state.context.block = Block::PreFormat;

        let mut vec: Vec<Token> = vec![];
        context::close(&state, &mut vec);
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
