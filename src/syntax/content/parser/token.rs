use crate::syntax::content::Parseable as _;

pub mod literal;
pub mod anchor;
pub mod linebreak;
pub mod paragraph;
pub mod span;
pub mod header;
pub mod preformat;
pub mod code;

#[derive(Debug)]
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

#[cfg(test)]
mod tests {

    #[test]
    fn smoke() {}
}
