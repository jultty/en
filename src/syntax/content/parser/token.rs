use crate::syntax::content::Parseable as _;

pub mod literal;
pub mod anchor;
pub mod linebreak;
pub mod paragraph;
pub mod span;
pub mod header;
pub mod preformat;
pub mod code;
pub mod oblique;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    Anchor(anchor::Anchor),
    Code(code::Code),
    Header(header::Header),
    LineBreak(linebreak::LineBreak),
    Literal(literal::Literal),
    Oblique(oblique::Oblique),
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
            Token::Oblique(ref d) => d.render(),
            Token::Paragraph(ref d) => d.render(),
            Token::PreFormat(ref d) => d.render(),
            Token::Span(ref d) => d.render(),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let data = match *self {
            Token::Anchor(ref d) => format!("{d}"),
            Token::Code(ref d) => format!("{d}"),
            Token::Header(ref d) => format!("{d}"),
            Token::LineBreak(ref d) => format!("{d}"),
            Token::Literal(ref d) => format!("{d}"),
            Token::Oblique(ref d) => format!("{d}"),
            Token::Paragraph(ref d) => format!("{d}"),
            Token::PreFormat(ref d) => format!("{d}"),
            Token::Span(ref d) => format!("{d}"),
        };

        write!(f, "T*{data}")
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn smoke() {}
}
