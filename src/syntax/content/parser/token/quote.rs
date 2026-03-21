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
        panic!("Attempt to lex a quote directly from a lexeme")
    }

    fn render(&self) -> String {
        let opening = if let Some(url) = &self.url {
            format!(r#"<blockquote cite="{url}">"#)
        } else {
            String::from("<blockquote>")
        };

        let content = if let Some(citation) = &self.citation {
            format!(
                r#"{}<br/><cite class="quote-citation">{citation}</cite>"#,
                &self.text
            )
        } else {
            String::from(&self.text)
        };

        format!("\n{opening}\n{content}\n</blockquote>\n")
    }

    fn flatten(&self) -> String {
        if let Some(citation) = &self.citation {
            format!(r#""{}" -- {}"#, self.text, citation)
        } else {
            format!(r#""{}""#, self.text)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::content::parser::Token;

    #[test]
    fn display() {
        let mut quote_slim = Quote::default();
        quote_slim.text = "iXh0141J7B8P46Gv".to_string();

        println!("{quote_slim}");
        assert!(format!("{quote_slim}").contains("Quote"));
        assert!(!format!("{quote_slim}").contains("+url"));
        assert!(!format!("{quote_slim}").contains("+citation"));
        assert_eq!(format!("{}", Token::Quote(quote_slim)), "Tk:Quote []");

        let mut quote_cited = Quote::default();
        quote_cited.text = "iXh0141J7B8P46Gv".to_string();
        quote_cited.citation = Some("k8Fy7htmvi2NG7yh".to_string());

        println!("{quote_cited}");
        assert!(format!("{quote_cited}").contains("Quote"));
        assert!(!format!("{quote_cited}").contains("+url"));
        assert!(format!("{quote_cited}").contains("+citation"));
        assert_eq!(
            format!("{}", Token::Quote(quote_cited)),
            "Tk:Quote [+citation]",
        );

        let mut quote_with_url = Quote::default();
        quote_with_url.text = "iXh0141J7B8P46Gv".to_string();
        quote_with_url.url = Some("CttVJU2IHDsjSjao".to_string());

        println!("{quote_with_url}");
        assert!(format!("{quote_with_url}").contains("Quote"));
        assert!(format!("{quote_with_url}").contains("+url"));
        assert!(!format!("{quote_with_url}").contains("+citation"));
        assert_eq!(
            format!("{}", Token::Quote(quote_with_url)),
            "Tk:Quote [+url]",
        );

        let mut quote_full = Quote::default();
        quote_full.text = "iXh0141J7B8P46Gv".to_string();
        quote_full.citation = Some("k8Fy7htmvi2NG7yh".to_string());
        quote_full.url = Some("CttVJU2IHDsjSjao".to_string());

        println!("{quote_full}");
        assert!(format!("{quote_full}").contains("Quote"));
        assert!(format!("{quote_full}").contains("+url"));
        assert!(format!("{quote_full}").contains("+citation"));
        assert_eq!(
            format!("{}", Token::Quote(quote_full)),
            "Tk:Quote [+url +citation]",
        );
    }

    #[test]
    fn flatten() {
        assert_eq!(Quote::default().flatten(), r#""""#);

        let mut without_citation = Quote::default();
        let text = "AphyFDQHVbkOeaNw";
        without_citation.text = text.to_string();
        assert_eq!(without_citation.flatten(), format!(r#""{text}""#));

        let without_citation_token = Token::Quote(without_citation);
        assert_eq!(without_citation_token.flatten(), format!(r#""{text}""#));

        let mut with_citation = Quote::default();
        let citation = "B35rcofYM0J7";
        with_citation.text = text.to_string();
        with_citation.citation = Some(citation.to_string());
        assert_eq!(
            with_citation.flatten(),
            format!(r#""{text}" -- {citation}"#)
        );

        let with_citation_token = Token::Quote(with_citation);
        assert_eq!(
            with_citation_token.flatten(),
            format!(r#""{text}" -- {citation}"#)
        );
    }

    #[test]
    #[should_panic(expected = "Attempt to lex a quote directly from a lexeme")]
    fn lex() {
        let lexeme = Lexeme::new("z2UI", "FiCd", "rtq4");
        Quote::lex(&lexeme);
    }
}
