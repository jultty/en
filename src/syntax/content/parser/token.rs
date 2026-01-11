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
pub mod span;
pub mod strike;
pub mod underline;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    Anchor(Box<anchor::Anchor>),
    Bold(bold::Bold),
    CheckBox(checkbox::CheckBox),
    Code(code::Code),
    Strike(strike::Strike),
    Header(header::Header),
    Item(item::Item),
    LineBreak(linebreak::LineBreak),
    List(list::List),
    Literal(literal::Literal),
    Oblique(oblique::Oblique),
    Paragraph(paragraph::Paragraph),
    PreFormat(preformat::PreFormat),
    Underline(underline::Underline),
}

impl Token {
    pub fn render(&self) -> String {
        match *self {
            Token::Anchor(ref d) => d.render(),
            Token::Bold(ref d) => d.render(),
            Token::CheckBox(ref d) => d.render(),
            Token::Code(ref d) => d.render(),
            Token::Strike(ref d) => d.render(),
            Token::Header(ref d) => d.render(),
            Token::Item(ref d) => d.render(),
            Token::LineBreak(ref d) => d.render(),
            Token::List(ref d) => d.render(),
            Token::Literal(ref d) => d.render(),
            Token::Oblique(ref d) => d.render(),
            Token::Paragraph(ref d) => d.render(),
            Token::PreFormat(ref d) => d.render(),
            Token::Underline(ref d) => d.render(),
        }
    }

    pub fn flatten(&self) -> String {
        match *self {
            Token::Anchor(ref d) => d.flatten(),
            Token::Bold(ref d) => d.flatten(),
            Token::CheckBox(ref d) => d.flatten(),
            Token::Code(ref d) => d.flatten(),
            Token::Strike(ref d) => d.flatten(),
            Token::Header(ref d) => d.flatten(),
            Token::Item(ref d) => d.flatten(),
            Token::LineBreak(ref d) => d.flatten(),
            Token::List(ref d) => d.flatten(),
            Token::Literal(ref d) => d.flatten(),
            Token::Oblique(ref d) => d.flatten(),
            Token::Paragraph(ref d) => d.flatten(),
            Token::PreFormat(ref d) => d.flatten(),
            Token::Underline(ref d) => d.flatten(),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let data = match *self {
            Token::Anchor(ref d) => format!("{d}"),
            Token::Bold(ref d) => format!("{d}"),
            Token::CheckBox(ref d) => format!("{d}"),
            Token::Code(ref d) => format!("{d}"),
            Token::Strike(ref d) => format!("{d}"),
            Token::Header(ref d) => format!("{d}"),
            Token::Item(ref d) => format!("{d}"),
            Token::LineBreak(ref d) => format!("{d}"),
            Token::List(ref d) => format!("{d}"),
            Token::Literal(ref d) => format!("{d}"),
            Token::Oblique(ref d) => format!("{d}"),
            Token::Paragraph(ref d) => format!("{d}"),
            Token::PreFormat(ref d) => format!("{d}"),
            Token::Underline(ref d) => format!("{d}"),
        };

        write!(f, "Tk:{data}")
    }
}
