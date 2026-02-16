use crate::syntax::content::parser::{
    State, Token,
    token::{Header, Paragraph, PreFormat, Verse},
};

pub mod anchor;
pub mod block;
pub mod inline;
pub mod list;
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
/// Panics if there is an open header or list at end of input.
pub fn close(state: &State, tokens: &mut Vec<Token>) {
    match state.context.block {
        Block::PreFormat => {
            tokens.push(Token::PreFormat(PreFormat::new(false)));
        },
        Block::Paragraph => {
            tokens.push(Token::Paragraph(Paragraph::new(false)));
        },
        Block::List => {
            panic!("End of input with open list")
        },
        Block::Header(level) => {
            tokens.push(Token::Header(Header::from_u8(level, false, None)));
        },
        Block::Quote => {
            panic!("End of input with open quote")
        },
        Block::Table => {
            panic!("End of input with open table")
        },
        Block::Verse => {
            tokens.push(Token::Verse(Verse::new(false)));
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
}
