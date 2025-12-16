use crate::syntax::content::Parseable as _;
use crate::syntax::content::elements::{
    paragraph::Paragraph, header::Header, span::Span,
};

pub enum Token {
    Paragraph(Paragraph),
    Header(Header),
    Span(Span),
}

impl Token {
    pub fn render(&self) -> String {
        match *self {
            Token::Paragraph(ref d) => d.render(),
            Token::Header(ref d) => d.render(),
            Token::Span(ref d) => d.render(),
        }
    }
}

impl From<Paragraph> for Token {
    fn from(d: Paragraph) -> Self {
        Token::Paragraph(d)
    }
}

impl From<Header> for Token {
    fn from(d: Header) -> Self {
        Token::Header(d)
    }
}

impl From<Span> for Token {
    fn from(d: Span) -> Self {
        Token::Span(d)
    }
}

pub struct Lexeme<'l> {
    pub raw: &'l str,
    pub first: &'l str,
}

impl<'l> Lexeme<'l> {
    pub fn new(text: &'l str) -> Lexeme<'l> {
        let vec: Vec<&'l str> = text.split(" ").collect();

        Self {
            raw: text,
            first: vec.first().unwrap_or_else(|| unreachable!()),
        }
    }
}
