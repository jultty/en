use std::{iter::Peekable, slice::Iter};

use crate::{
    prelude::*,
    syntax::content::{
        Parseable as _,
        parser::{
            lexeme::Lexeme,
            state::State,
            token::{
                Token, bold::Bold, checkbox::CheckBox, oblique::Oblique,
                strike::Strike, underline::Underline,
            },
        },
    },
};

pub fn parse(
    lexeme: &Lexeme,
    state: &mut State,
    tokens: &mut Vec<Token>,
    iterator: &mut Peekable<Iter<'_, Lexeme>>,
) -> bool {
    if let super::context::Block::PreFormat = state.context.block {
        return false;
    } else if let super::context::Inline::Code = state.context.inline {
        return false;
    }

    if Underline::probe(lexeme) {
        log!("Underline probed: {lexeme}");
        tokens
            .push(Token::Underline(Underline::new(!state.switches.underline)));
        state.switches.underline = !state.switches.underline;
        iterator.next();
        return true;
    } else if Oblique::probe(lexeme) {
        log!("Oblique probed: {lexeme}");
        tokens.push(Token::Oblique(Oblique::new(!state.switches.oblique)));
        state.switches.oblique = !state.switches.oblique;
        return true;
    } else if Strike::probe(lexeme) {
        log!("Strike probed: {lexeme}");
        tokens.push(Token::Strike(Strike::new(!state.switches.crossout)));
        state.switches.crossout = !state.switches.crossout;
        iterator.next();
        return true;
    } else if Bold::probe(lexeme) {
        log!("Bold probed: {lexeme}");
        tokens.push(Token::Bold(Bold::new(!state.switches.bold)));
        state.switches.bold = !state.switches.bold;
        return true;
    } else if CheckBox::probe(lexeme) {
        log!("CheckBox probed: {lexeme}");
        tokens.push(Token::CheckBox(CheckBox::lex(lexeme)));
        iterator.next();
        iterator.next();
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::{syntax::content::parser, types::Graph};

    fn read(input: &str) -> String {
        parser::read(input, &Graph::new(None).meta.config)
    }

    #[test]
    fn oblique_anchor() {
        assert_eq!(
            read("w _|S|_ w"),
            r#"<p>w <em><a href="/node/S">S</a></em> w</p>"#
        );
    }

    #[test]
    fn oblique_anchor_with_trailing_comma() {
        assert_eq!(
            read("w _|S|_, w"),
            r#"<p>w <em><a href="/node/S">S</a></em>, w</p>"#
        );
    }

    #[test]
    fn oblique() {
        assert_eq!(
            read(
                "_|this anchor is oblique|o as are these literals_ but not these _just these_, not this _and these with an |anchor| again_"
            ),
            r#"<p><em><a href="/node/o">this anchor is oblique</a> as are these literals</em> but not these <em>just these</em>, not this <em>and these with an <a href="/node/anchor">anchor</a> again</em></p>"#
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
}
