use crate::syntax::content::Parseable as _;

pub mod anchor;
pub mod bold;
pub mod checkbox;
pub mod code;
pub mod header;
pub mod item;
pub mod linebreak;
pub mod list;
pub mod literal;
pub mod oblique;
pub mod paragraph;
pub mod preformat;
pub mod quote;
pub mod strike;
pub mod table;
pub mod underline;
pub mod verse;

pub use anchor::Anchor;
pub use bold::Bold;
pub use checkbox::CheckBox;
pub use code::Code;
pub use header::Header;
pub use item::Item;
pub use linebreak::LineBreak;
pub use list::List;
pub use literal::Literal;
pub use oblique::Oblique;
pub use paragraph::Paragraph;
pub use preformat::PreFormat;
pub use quote::Quote;
pub use strike::Strike;
pub use table::Table;
pub use underline::Underline;
pub use verse::Verse;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    Anchor(Box<Anchor>),
    Bold(Bold),
    CheckBox(CheckBox),
    Code(Code),
    Strike(Strike),
    Header(Header),
    Item(Item),
    LineBreak(LineBreak),
    List(List),
    Literal(Literal),
    Oblique(Oblique),
    Paragraph(Paragraph),
    PreFormat(PreFormat),
    Quote(Quote),
    Table(Table),
    Underline(Underline),
    Verse(Verse),
}

impl Token {
    pub fn render(&self) -> String {
        match self {
            Token::Anchor(d) => d.render(),
            Token::Bold(d) => d.render(),
            Token::CheckBox(d) => d.render(),
            Token::Code(d) => d.render(),
            Token::Strike(d) => d.render(),
            Token::Header(d) => d.render(),
            Token::Item(d) => d.render(),
            Token::LineBreak(d) => d.render(),
            Token::List(d) => d.render(),
            Token::Literal(d) => d.render(),
            Token::Oblique(d) => d.render(),
            Token::Paragraph(d) => d.render(),
            Token::PreFormat(d) => d.render(),
            Token::Quote(d) => d.render(),
            Token::Table(d) => d.render(),
            Token::Underline(d) => d.render(),
            Token::Verse(d) => d.render(),
        }
    }

    pub fn flatten(&self) -> String {
        match self {
            Token::Anchor(d) => d.flatten(),
            Token::Bold(d) => d.flatten(),
            Token::CheckBox(d) => d.flatten(),
            Token::Code(d) => d.flatten(),
            Token::Strike(d) => d.flatten(),
            Token::Header(d) => d.flatten(),
            Token::Item(d) => d.flatten(),
            Token::LineBreak(d) => d.flatten(),
            Token::List(d) => d.flatten(),
            Token::Literal(d) => d.flatten(),
            Token::Oblique(d) => d.flatten(),
            Token::Paragraph(d) => d.flatten(),
            Token::PreFormat(d) => d.flatten(),
            Token::Quote(d) => d.flatten(),
            Token::Table(d) => d.flatten(),
            Token::Underline(d) => d.flatten(),
            Token::Verse(d) => d.flatten(),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let data = match self {
            Token::Anchor(d) => format!("{d}"),
            Token::Bold(d) => format!("{d}"),
            Token::CheckBox(d) => format!("{d}"),
            Token::Code(d) => format!("{d}"),
            Token::Strike(d) => format!("{d}"),
            Token::Header(d) => format!("{d}"),
            Token::Item(d) => format!("{d}"),
            Token::LineBreak(d) => format!("{d}"),
            Token::List(d) => format!("{d}"),
            Token::Literal(d) => format!("{d}"),
            Token::Oblique(d) => format!("{d}"),
            Token::Paragraph(d) => format!("{d}"),
            Token::PreFormat(d) => format!("{d}"),
            Token::Quote(d) => format!("{d}"),
            Token::Table(d) => format!("{d}"),
            Token::Underline(d) => format!("{d}"),
            Token::Verse(d) => format!("{d}"),
        };

        write!(f, "Tk:{data}")
    }
}
