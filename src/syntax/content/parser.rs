use crate::syntax::content::{Parseable, Token, Lexeme, make_lexmap};

pub fn read<DefaultToken: Parseable>(text: &str) -> String {
    let escaped_text = tera::escape_html(text);
    parse(&lex(&escaped_text, &make_lexmap::<DefaultToken>()))
}

fn lex(text: &str, map: super::LexMap) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    for line in text.lines().filter(|x| !x.trim().is_empty()) {
        let lexeme = Lexeme::new(line);

        for &(ref matcher, lexer) in map {
            if matcher(&lexeme) {
                tokens.push(lexer(&lexeme));
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
