use crate::syntax::content::parser::{
    state::State,
    token::{
        Token, paragraph::Paragraph, preformat::PreFormat, list::List,
        item::Item,
    },
};

pub mod anchor;
pub mod block;
pub mod inline;

#[derive(Clone, Debug)]
pub struct Context {
    pub block: Block,
    pub inline: Inline,
}

#[derive(Clone, Debug)]
pub enum Block {
    Paragraph,
    Header(u8),
    Item(bool),
    List(bool),
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
        Block::Item(ordered) => {
            tokens.push(Token::Item(Item::new(false)));
            tokens.push(Token::List(List::new(false, ordered)));
        },
        Block::List(ordered) => {
            tokens.push(Token::List(List::new(false, ordered)));
        },
        Block::Header(_) => panic!("End of input with open header"),
        Block::None => (),
    }
}
