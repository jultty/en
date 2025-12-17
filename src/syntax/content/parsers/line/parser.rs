use crate::syntax::content::{
    LexMap, Line, Parseable, Token, parsers::line::elements::header::Header,
    make_lexmap, Lexeme,
};

const LEXMAP: LexMap =
    &[(Header::probe, |line| Token::Header(Header::lex(line)))];

pub(in crate::syntax::content) fn read<DefaultToken: Parseable>(
    text: &str,
) -> String {
    parse(&lex(text, &make_lexmap::<DefaultToken>(LEXMAP)))
}

fn lex(text: &str, map: LexMap) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    for raw_line in text.lines() {
        let line = Lexeme::Line(Line::new(raw_line));

        for &(ref probe, lex) in map {
            if probe(&line) {
                tokens.push(lex(&line));
                break;
            }
        }
    }

    tokens
}

fn parse(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(Token::render)
        .collect::<Vec<_>>()
        .join("\n")
}
