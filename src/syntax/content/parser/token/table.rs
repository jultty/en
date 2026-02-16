use crate::syntax::content::{Parseable, parser::Lexeme};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Table {
    pub headers: Vec<String>,
    pub contents: Vec<Vec<String>>,
}

impl Table {
    pub fn probe_end(lexeme: &Lexeme) -> bool {
        lexeme.match_char_triple('\n', '%', '\n') || lexeme.last()
    }

    pub fn add_header(&mut self, header: &str) {
        self.headers.push(header.trim().to_string());
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.contents.push(row);
    }

    pub fn add_cell(&mut self, content: &str) {
        if let Some(last) = self.contents.last_mut() {
            last.push(content.trim().to_string());
        }
    }
}

impl Parseable for Table {
    fn probe(lexeme: &Lexeme) -> bool {
        lexeme.match_char_triple('\n', '%', '\n')
    }

    fn lex(_lexeme: &Lexeme) -> Table {
        Table::default()
    }

    fn render(&self) -> String {
        let mut xml = String::from("\n<table>\n");

        if !self.headers.is_empty() {
            xml.push_str("<tr>\n");
            for header in &self.headers {
                xml.push_str(format!("<th>{header}</th>\n").as_str());
            }
            xml.push_str("\n</tr>\n");
        }

        for row in &self.contents {
            if !row.is_empty() {
                xml.push_str("<tr>\n");
                for cell in row {
                    xml.push_str(format!("<td>{cell}</td>\n").as_str());
                }
                xml.push_str("\n</tr>\n");
            }
        }

        xml.push_str("\n</table>\n");
        xml
    }

    fn flatten(&self) -> String {
        String::default()
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Table")
    }
}
