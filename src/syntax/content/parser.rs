use crate::{prelude::*, types::Config};
use super::{Parseable as _, Token, LexMap};
use token::{
    anchor::Anchor, linebreak::LineBreak, paragraph::Paragraph, header::Header,
    preformat::PreFormat, literal::Literal, code::Code,
};
use lexeme::Lexeme;
use context::{Block, Inline};

pub mod token;
pub mod lexeme;
pub mod segment;
pub mod context;
pub mod point;
pub mod state;

const LEXMAP: LexMap = &[
    (LineBreak::probe, |lexeme| {
        Token::LineBreak(LineBreak::lex(lexeme))
    }),
    (Literal::probe, |lexeme| {
        Token::Literal(Literal::lex(lexeme))
    }),
];

fn lex(text: &str, map: LexMap, config: &Config) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::default();
    let mut state = state::State::default();

    let segments = segment::segment(text);
    let lexemes = Lexeme::collect(&segments);

    log!("Segments: {segments:?}");

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
                    log!("Block Context: None -> Paragraph on {lexeme}");
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
                if Paragraph::probe_end(lexeme) {
                    log!("Block Context: Paragraph -> None on {lexeme}");
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

        if point::puncture(lexeme, &mut state, &mut tokens) {
            continue;
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

                    if lexeme.match_as_char('|') {
                        state.buffers.anchor.candidate.leading = true;
                    } else {
                        state.buffers.anchor.candidate.text = lexeme.text();
                        // because we probed positively and this is not a pipe,
                        // the next lexeme must be and so it was now parsed
                        iterator.next();
                    }
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
            Inline::Anchor => {
                if context::anchor::parse(lexeme, &mut state, &mut tokens) {
                    continue;
                }
            },
        }

        for &(ref probe, lex) in map {
            if probe(lexeme) {
                let token = lex(lexeme);
                log!("Lexmap lexed {lexeme} into {token}");
                tokens.push(token);
                break;
            }
        }
    }

    context::close(&state, &mut tokens);
    tokens
}

fn parse(tokens: &[Token]) -> String {
    tokens.iter().map(Token::render).collect::<String>()
}

pub(super) fn read(text: &str, config: &Config) -> String {
    parse(&lex(text, LEXMAP, config))
}

#[cfg(test)]
mod tests {
    use crate::{
        types::Graph,
        syntax::content::parser::{state::State, token::header::Level},
    };

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
    fn flanking_with_trailing_comma() {
        assert_eq!(
            read_noconfig("|Node|,"),
            r#"<p><a href="/node/Node">Node</a>,</p>"#
        );
    }

    #[test]
    fn flanking_with_trailing_comma_and_space() {
        assert_eq!(
            read_noconfig("|Node|, at"),
            r#"<p><a href="/node/Node">Node</a>, at</p>"#
        );
    }

    #[test]
    fn flanking_at_eoi() {
        assert_eq!(
            read_noconfig("|Node|"),
            r#"<p><a href="/node/Node">Node</a></p>"#
        );
    }

    #[test]
    fn needless_three_pipe_anchor() {
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
    fn nonleading_plural_anchor_at_eoi() {
        assert_eq!(
            read_noconfig("element|s"),
            r#"<p><a href="/node/element">elements</a></p>"#
        );
    }

    #[test]
    fn leading_plural_anchor_at_eoi() {
        assert_eq!(
            read_noconfig("|element|s"),
            r#"<p><a href="/node/element">elements</a></p>"#
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
    fn http_external_anchor_leading_no_third_then_newline() {
        assert_eq!(
            read_noconfig(concat!(
                "|Rust toolchain|https://rustup.rs/",
                "\n",
                "at rustup.rs",
            )),
            concat!(
                r#"<p><a href="https://rustup.rs/">Rust toolchain</a>"#,
                "\n",
                "at rustup.rs</p>",
            )
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_space() {
        assert_eq!(
            read_noconfig("|Rust toolchain|https://rustup.rs/ at rustup.rs"),
            r#"<p><a href="https://rustup.rs/">Rust toolchain</a> at rustup.rs</p>"#
        );
    }

    #[test]
    fn http_external_anchor_leading_no_third_then_eoi() {
        assert_eq!(
            read_noconfig("|Rust toolchain|https://rustup.rs/"),
            r#"<p><a href="https://rustup.rs/">Rust toolchain</a></p>"#
        );
    }

    #[test]
    fn newline_wrapped_anchor() {
        assert_eq!(
            read_noconfig("\n|SomeAnchor|\n"),
            concat!(
                "\n",
                r#"<p><a href="/node/SomeAnchor">SomeAnchor</a></p>"#
            ),
        );
    }

    #[test]
    fn newline_separated_anchors() {
        assert_eq!(
            read_noconfig("|SomeAnchor|\n|SomeOtherAnchor|\n"),
            concat!(
                r#"<p><a href="/node/SomeAnchor">SomeAnchor</a>"#,
                "\n",
                r#"<a href="/node/SomeOtherAnchor">SomeOtherAnchor</a></p>"#
            )
        );
    }

    #[test]
    fn empty_line_separated_anchors() {
        assert_eq!(
            read_noconfig("|SomeAnchor|\n\n|SomeOtherAnchor|\n"),
            concat!(
                r#"<p><a href="/node/SomeAnchor">SomeAnchor</a></p>"#,
                "\n",
                "\n",
                r#"<p><a href="/node/SomeOtherAnchor">SomeOtherAnchor</a></p>"#
            ),
        );
    }

    #[test]
    fn homepage_footer() {
        assert_eq!(
            read_noconfig(
                "made by jutty|https://jutty.dev • acknowledgments|Acknowledgments • |source code|https://codeberg.org/jutty/en"
            ),
            r#"<p>made by <a href="https://jutty.dev">jutty</a> • <a href="/node/Acknowledgments">acknowledgments</a> • <a href="https://codeberg.org/jutty/en">source code</a></p>"#
        );
    }

    #[test]
    fn trailing_anchor() {
        assert_eq!(
            read_noconfig("see acks|acks"),
            r#"<p>see <a href="/node/acks">acks</a></p>"#
        );
    }

    #[test]
    fn trailing_anchor_with_newline() {
        assert_eq!(
            read_noconfig("\nsee acks|acks\n"),
            concat!("\n", r#"<p>see <a href="/node/acks">acks</a></p>"#)
        );
    }

    #[test]
    fn trailing_oblique() {
        assert_eq!(read_noconfig("see _acks_"), "<p>see <em>acks</em></p>");
    }

    #[test]
    fn trailing_oblique_with_newline() {
        assert_eq!(read_noconfig("see _acks_\n"), "<p>see <em>acks</em></p>");
    }

    #[test]
    fn pre() {
        let payload = "D0qdJ184f3q1okbYu3Xm1d93jj6jy615";
        assert_eq!(
            read_noconfig(&format!("`\n{payload}\n`\n")),
            format!("<pre>\n{payload}\n</pre>"),
        );
    }

    #[test]
    fn eoi_pre() {
        let payload = "Jp8INpWzsQmk20jpIhBFCfMUXOztxv0w";
        assert_eq!(
            read_noconfig(&format!("`\n{payload}\n`")),
            format!("<pre>\n{payload}\n</pre>"),
        );
    }

    #[test]
    #[should_panic(expected = "End of input with open header")]
    fn end_with_open_header() {
        let mut state = State::default();
        state.context.block = Block::Header(1);

        context::close(&state, &mut vec![]);
    }

    #[test]
    fn end_with_open_preformat() {
        let mut state = State::default();
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
