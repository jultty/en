use crate::syntax::content::parser::{
    State, Token,
    token::{Header, Paragraph, Verse},
};

pub mod anchor;
pub mod block;
pub mod inline;
pub mod list;
pub mod preformat;
pub mod quote;
pub mod table;

#[derive(Clone, Default, Debug)]
pub struct Context {
    pub block: Block,
    pub inline: Inline,
}

#[derive(Clone, Default, Debug)]
pub enum Block {
    Paragraph,
    Header(u8), // level
    List,
    PreFormat,
    Quote,
    Table,
    Verse,
    #[default]
    None,
}

#[derive(Clone, Default, Debug)]
pub enum Inline {
    Anchor,
    Code,
    #[default]
    None,
}

/// # Panics
/// Panics if there is an open token at end of input that can't be easily
/// closed by simply adding a matching closing token. This normally is handled
/// by context parsers and probably indicates an error in one of them.
pub fn close(state: &State, tokens: &mut Vec<Token>) {
    match state.context.block {
        Block::Paragraph => {
            tokens.push(Token::Paragraph(Paragraph::new(false)));
        },
        Block::Header(level) => {
            tokens.push(Token::Header(Header::from_u8(level, false, None)));
        },
        Block::Verse => {
            tokens.push(Token::Verse(Verse::new(false)));
        },
        Block::PreFormat => {
            panic!("End of input with open preformat: {tokens:#?}")
        },
        Block::List => {
            panic!("End of input with open list: {tokens:#?}")
        },
        Block::Quote => {
            panic!("End of input with open quote: {tokens:#?}")
        },
        Block::Table => {
            panic!("End of input with open table: {tokens:#?}")
        },
        Block::None => (),
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::content::parser::{State, context::Block};

    #[test]
    #[should_panic(expected = "End of input with open list")]
    fn open_list_eoi() {
        let mut state = State::default();
        state.context.block = Block::List;
        super::close(&state, &mut vec![]);
    }

    #[test]
    #[should_panic(expected = "End of input with open quote")]
    fn open_quote_eoi() {
        let mut state = State::default();
        state.context.block = Block::Quote;
        super::close(&state, &mut vec![]);
    }

    #[test]
    #[should_panic(expected = "End of input with open table")]
    fn open_table_eoi() {
        let mut state = State::default();
        state.context.block = Block::Table;
        super::close(&state, &mut vec![]);
    }

    #[test]
    fn open_verse_eoi() {
        let mut state = State::default();
        state.context.block = Block::Verse;
        super::close(&state, &mut vec![]);
    }
}
