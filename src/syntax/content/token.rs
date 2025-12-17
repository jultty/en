use super::Parseable as _;
use super::parsers::word::elements::{literal::Literal};
use super::parsers::line::elements::{
    paragraph::Paragraph, header::Header, span::Span,
};

pub enum Token {
    Paragraph(Paragraph),
    Header(Header),
    Span(Span),
    Literal(Literal),
}

impl Token {
    pub fn render(&self) -> String {
        match *self {
            Token::Paragraph(ref d) => d.render(),
            Token::Header(ref d) => d.render(),
            Token::Span(ref d) => d.render(),
            Token::Literal(ref d) => d.render(),
        }
    }
}

impl From<Paragraph> for Token {
    fn from(d: Paragraph) -> Token {
        Token::Paragraph(d)
    }
}

impl From<Header> for Token {
    fn from(d: Header) -> Token {
        Token::Header(d)
    }
}

impl From<Span> for Token {
    fn from(d: Span) -> Token {
        Token::Span(d)
    }
}

impl From<Literal> for Token {
    fn from(d: Literal) -> Token {
        Token::Literal(d)
    }
}
