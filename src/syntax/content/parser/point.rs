use std::{iter::Peekable, slice::Iter};

use crate::{
    prelude::*,
    syntax::content::{
        Parseable as _,
        parser::{
            Lexeme, State, Token,
            token::{Bold, CheckBox, Oblique, Strike, Underline},
        },
    },
};

pub fn parse(
    lexeme: &Lexeme,
    state: &mut State,
    tokens: &mut Vec<Token>,
    iterator: &mut Peekable<Iter<'_, Lexeme>>,
) -> bool {
    if matches!(state.context.block, super::context::Block::PreFormat)
        || matches!(state.context.inline, super::context::Inline::Code)
    {
        return false;
    }

    if Underline::probe(lexeme) {
        log!(VERBOSE, "Underline probed: {lexeme}");
        tokens
            .push(Token::Underline(Underline::new(!state.switches.underline)));
        state.switches.underline = !state.switches.underline;
        iterator.next();
        return true;
    } else if Oblique::probe(lexeme) {
        log!(VERBOSE, "Oblique probed: {lexeme}");
        tokens.push(Token::Oblique(Oblique::new(!state.switches.oblique)));
        state.switches.oblique = !state.switches.oblique;
        return true;
    } else if Strike::probe(lexeme) {
        log!(VERBOSE, "Strike probed: {lexeme}");
        tokens.push(Token::Strike(Strike::new(!state.switches.crossout)));
        state.switches.crossout = !state.switches.crossout;
        iterator.next();
        return true;
    } else if Bold::probe(lexeme) {
        log!(VERBOSE, "Bold probed: {lexeme}");
        tokens.push(Token::Bold(Bold::new(!state.switches.bold)));
        state.switches.bold = !state.switches.bold;
        return true;
    } else if CheckBox::probe(lexeme) {
        log!(VERBOSE, "CheckBox probed: {lexeme}");
        tokens.push(Token::CheckBox(CheckBox::lex(lexeme)));
        iterator.next();
        iterator.next();
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::{graph::Graph, syntax::content::parser};

    fn read(input: &str) -> String { parser::read(input, &Graph::default()) }

    #[test]
    fn oblique_anchor() {
        assert_eq!(
            read("w _|S|_ w"),
            concat!(
                r#"<p>w <em><a class="detached" title="" "#,
                r#"href="/node/S">S</a></em> w</p>"#,
            )
        );
    }

    #[test]
    fn oblique_anchor_with_trailing_comma() {
        assert_eq!(
            read("w _|S|_, w"),
            concat!(r#"<p>w <em><a class="detached" title="" "#,
            r#"href="/node/S">S</a></em>, w</p>"#)
        );
    }

    #[test]
    fn oblique() {
        assert_eq!(
            read(
                "_|this anchor is oblique|o as are these literals_ but not \
                    these _just these_, not this _and these with an |anc80r| \
                    again_"
            ),
            concat!(
                r#"<p><em><a class="detached" title="" href="/node/o">"#,
                r#"this anchor is oblique</a> as are these literals</em> "#,
                r#"but not these <em>just these</em>, not this <em>and these "#,
                r#"with an <a class="detached" title="" "#,
                r#"href="/node/anc80r">anc80r</a> again</em></p>"#,
            )
        );
    }

    #[test]
    fn trailing_oblique() {
        assert_eq!(read("see _acks_"), "<p>see <em>acks</em></p>");
    }

    #[test]
    fn trailing_oblique_with_newline() {
        assert_eq!(read("see _acks_\n"), "<p>see <em>acks</em></p>");
    }

    #[test]
    fn underline() {
        assert_eq!(read("__u__"), "<p><u>u</u></p>");
    }

    #[test]
    fn checkbox() {
        assert_eq!(read("[ ]"), r#"<p><input type="checkbox"/></p>"#);
        assert_eq!(read("[x]"), r#"<p><input type="checkbox" checked /></p>"#);
    }
}
