use super::{Parseable as _, Token, Lexeme, LEXMAP};

pub fn read(text: &str) -> String {
    parse(&lex(text))
}

fn lex(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for line in text
        .lines()
        .filter(|x| !x.is_empty())
        .filter(|x| !x.replace(" ", "").is_empty())
    {
        let lexeme = Lexeme::new(line);

        for &(ref matcher, lexer) in LEXMAP {
            if matcher(&lexeme) {
                tokens.push(lexer(&lexeme));
                break;
            }
        }
    }

    tokens
}

fn parse(tokens: &[Token]) -> String {
    let mut out_text: Vec<String> = Vec::new();
    for token in tokens {
        out_text.push(match *token {
            Token::Paragraph(ref d) => d.render(),
            Token::Header(ref d) => d.render(),
        });
    }

    out_text.join("\n")
}
