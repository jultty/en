use crate::syntax::content::parser::{
    state::State,
    token::{Token, paragraph::Paragraph, preformat::PreFormat},
};

pub mod block;
pub mod inline;
pub mod anchor;
pub mod list;

#[derive(Clone, Debug)]
pub struct Context {
    pub block: Block,
    pub inline: Inline,
}

#[derive(Clone, Debug)]
pub enum Block {
    Paragraph,
    Header(u8), // level
    List,
    PreFormat,
    None,
}

#[derive(Clone, Debug)]
pub enum Inline {
    Anchor,
    Code,
    None,
}

/// # Panics
/// Panics if there is an open header at end of input.
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
        Block::Header(_) => panic!("End of input with open header"),
        Block::None => (),
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::content::parser::{context::Block, state::State};

    #[test]
    #[should_panic(expected = "End of input with open list")]
    fn open_list_eoi() {
        let mut state = State::default();
        state.context.block = Block::List;
        super::close(&state, &mut vec![]);
    }
}
