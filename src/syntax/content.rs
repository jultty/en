struct Lexeme<'l> {
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

pub enum Token {
    Paragraph(paragraph::Paragraph),
    Header(header::Header),
}

pub fn lex(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for line in text
        .lines()
        .filter(|x| !x.is_empty())
        .filter(|x| !x.replace(" ", "").is_empty())
    {
        let lexeme = Lexeme::new(line);
        if header::matches(&lexeme) {
            tokens.push(Token::Header(header::lex(&lexeme)));
        } else if paragraph::matches(&lexeme) {
            tokens.push(Token::Paragraph(paragraph::lex(&lexeme)));
        }
    }

    tokens
}

pub fn parse(tokens: &Vec<Token>) -> String {
    let mut out_text: Vec<String> = Vec::new();
    for token in tokens {
        out_text.push(match token {
            Token::Paragraph(p) => p.to_string(),
            Token::Header(h) => h.to_string(),
        });
    }

    out_text.join("\n")
}

mod paragraph {
    use std::fmt::Display;
    use super::Lexeme;

    pub struct Paragraph {
        text: String,
    }

    impl Display for Paragraph {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "<p>{}</p>", &self.text)
        }
    }

    pub fn matches(lexeme: &Lexeme) -> bool {
        !lexeme.raw.trim().is_empty()
    }

    pub fn lex(lexeme: &Lexeme) -> Paragraph {
        Paragraph {
            text: lexeme.raw.trim().to_owned(),
        }
    }
}

mod header {
    use crate::dev::log;
    use std::fmt::Display;
    use super::Lexeme;

    enum Level {
        One,
        Two,
        Three,
        Four,
        Five,
        Six,
    }

    impl Display for Level {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match *self {
                Level::One => write!(f, "1"),
                Level::Two => write!(f, "2"),
                Level::Three => write!(f, "3"),
                Level::Four => write!(f, "4"),
                Level::Five => write!(f, "5"),
                Level::Six => write!(f, "6"),
            }
        }
    }

    pub struct Header {
        level: Level,
        text: String,
    }

    impl Header {
        fn new(level: usize, text: &str) -> Self {
            Self {
                level: match level {
                    1 => Level::One,
                    2 => Level::Two,
                    3 => Level::Three,
                    4 => Level::Four,
                    5 => Level::Five,
                    6 => Level::Six,
                    _ => panic!(
                        "Attempted to construct a header with invalid level"
                    ),
                },
                text: text.to_owned(),
            }
        }
    }

    impl Display for Header {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "<h{}>{}</h{}>", &self.level, self.text, &self.level)
        }
    }

    pub fn matches(lexeme: &Lexeme) -> bool {
        !lexeme.first.trim().is_empty()
            && lexeme.first.replace("#", "").is_empty()
            && lexeme.first.len() <= 6
    }

    pub fn lex(lexeme: &Lexeme) -> Header {
        let header_level = lexeme.first.len();
        log(&lex, &format!("Header level is {header_level}"));

        let header_text = lexeme.raw.replace(lexeme.first, "");

        Header::new(header_level, &header_text)
    }
}
