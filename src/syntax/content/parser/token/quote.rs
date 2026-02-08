use crate::syntax::content::{Parseable, parser::Lexeme};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Quote {
    pub text: String,
    pub citation: Option<String>,
    pub url: Option<String>,
}

impl Quote {
    pub fn probe_end(lexeme: &Lexeme) -> bool {
        lexeme.match_char_sequence('\n', '\n')
    }

    pub fn extend_citation(&mut self, s: &str) {
        if let Some(current) = &self.citation {
            self.citation = Some(format!("{current}{s}"));
        } else {
            self.citation = Some(String::from(s));
        }
    }
}

impl Parseable for Quote {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_char('>') && lexeme.match_next_char(' ')
    }

    fn lex(_lexeme: &Lexeme) -> Quote {
        Quote::default()
    }

    fn render(&self) -> String {
        let opening = if let Some(url) = &self.url {
            format!(r#"<blockquote cite="{url}">"#)
        } else {
            String::from("<blockquote>")
        };

        let content = if let Some(citation) = &self.citation {
            format!(
                r#"{}<br/><p class="quote-citation">{citation}</p>"#,
                &self.text
            )
        } else {
            String::from(&self.text)
        };

        format!("\n{opening}\n{content}\n</blockquote>\n")
    }

    fn flatten(&self) -> String {
        String::default()
    }
}

impl std::fmt::Display for Quote {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut meta = String::default();
        if self.url.is_some() {
            meta.push_str("+url ");
        }
        if self.citation.is_some() {
            meta.push_str("+citation ");
        }

        write!(f, "Quote [{}]", meta.trim())
    }
}
