use crate::syntax::content::Parseable as _;

pub mod literal;
pub mod anchor;
pub mod linebreak;
pub mod paragraph;
pub mod span;
pub mod header;
pub mod preformat;
pub mod code;

pub enum Token {
    Anchor(anchor::Anchor),
    Code(code::Code),
    Header(header::Header),
    LineBreak(linebreak::LineBreak),
    Literal(literal::Literal),
    Paragraph(paragraph::Paragraph),
    PreFormat(preformat::PreFormat),
    Span(span::Span),
}

impl Token {
    pub fn render(&self) -> String {
        match *self {
            Token::Anchor(ref d) => d.render(),
            Token::Code(ref d) => d.render(),
            Token::Header(ref d) => d.render(),
            Token::LineBreak(ref d) => d.render(),
            Token::Literal(ref d) => d.render(),
            Token::Paragraph(ref d) => d.render(),
            Token::PreFormat(ref d) => d.render(),
            Token::Span(ref d) => d.render(),
        }
    }
}

impl From<paragraph::Paragraph> for Token {
    fn from(d: paragraph::Paragraph) -> Token {
        Token::Paragraph(d)
    }
}

impl From<header::Header> for Token {
    fn from(d: header::Header) -> Token {
        Token::Header(d)
    }
}

impl From<span::Span> for Token {
    fn from(d: span::Span) -> Token {
        Token::Span(d)
    }
}

impl From<literal::Literal> for Token {
    fn from(d: literal::Literal) -> Token {
        Token::Literal(d)
    }
}

impl From<anchor::Anchor> for Token {
    fn from(d: anchor::Anchor) -> Token {
        Token::Anchor(d)
    }
}

impl From<linebreak::LineBreak> for Token {
    fn from(d: linebreak::LineBreak) -> Token {
        Token::LineBreak(d)
    }
}

impl From<preformat::PreFormat> for Token {
    fn from(d: preformat::PreFormat) -> Token {
        Token::PreFormat(d)
    }
}

impl From<code::Code> for Token {
    fn from(d: code::Code) -> Token {
        Token::Code(d)
    }
}
