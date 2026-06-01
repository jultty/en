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

    pub fn add_row(&mut self, row: Vec<String>) { self.contents.push(row); }

    pub fn add_cell(&mut self, content: &str) {
        if let Some(last) = self.contents.last_mut() {
            last.push(content.trim().to_string());
        } else {
            self.contents.push(vec![content.trim().to_string()]);
        }
    }

    /// Counts the number of cells in the last row.
    pub fn last_row_count(&self) -> usize {
        if let Some(last) = self.contents.last() {
            last.len()
        } else {
            0
        }
    }
}

impl Parseable for Table {
    fn probe(lexeme: &Lexeme) -> bool { lexeme.match_char_sequence('%', '\n') }

    fn lex(_lexeme: &Lexeme) -> Table {
        panic!("Attempt to lex a table directly from a lexeme")
    }

    fn render(&self) -> String {
        let mut xml = String::from("\n<table>\n");
        let tab = "    ";

        if !self.headers.is_empty() {
            xml.push_str(format!("{tab}<tr>\n").as_str());
            for header in &self.headers {
                xml.push_str(format!("{tab}{tab}<th>{header}</th>\n").as_str());
            }
            xml.push_str(format!("{tab}</tr>\n").as_str());
        }

        for row in &self.contents {
            if !row.is_empty() && row.iter().any(|cell| !cell.is_empty()) {
                xml.push_str(format!("{tab}<tr>\n").as_str());
                for cell in row {
                    xml.push_str(
                        format!("{tab}{tab}<td>{cell}</td>\n").as_str(),
                    );
                }
                xml.push_str(format!("{tab}</tr>\n").as_str());
            }
        }

        xml.push_str("</table>\n");
        xml
    }

    fn flatten(&self) -> String { String::from("[Table]") }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let headers_width = self.headers.len();
        let contents_height = self.contents.len();
        let contents_width = self.last_row_count();

        let mut extra = String::default();
        if headers_width > 0 && contents_height > 0 {
            extra = format!(
                " [{contents_width}x{contents_height} +{headers_width} headers]"
            );
        } else if headers_width > 0 {
            extra = format!(" [+{headers_width} headers]");
        } else if contents_height > 0 {
            extra = format!(" [{contents_width}x{contents_height}]");
        }

        write!(f, "Table{extra}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::content::parser::Token;

    #[test]
    #[should_panic(expected = "Attempt to lex a table directly from a lexeme")]
    fn lex() {
        let lexeme = Lexeme::new("tp0h", "rrFt", "Qouf");
        Table::lex(&lexeme);
    }

    #[test]
    fn flatten() {
        assert_eq!(Table::default().flatten(), "[Table]");
        assert_eq!(Token::Table(Table::default()).flatten(), "[Table]");
    }

    #[test]
    fn display() {
        use std::string::ToString;

        let mut table = Table::default();
        table.add_header("A");
        table.add_header("B");
        table.add_header("C");

        let table_token = Token::Table(table.clone());
        assert_eq!(format!("{table}"), "Table [+3 headers]");
        assert_eq!(format!("{table_token}"), "Tk:Table [+3 headers]");

        table.add_row(
            ["1", "2", "3"]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );
        table.add_row(
            ["4", "5", "6"]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );
        table.add_row(
            ["7", "8", "9"]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );

        let table_token2 = Token::Table(table.clone());
        assert_eq!(format!("{table}"), "Table [3x3 +3 headers]");
        assert_eq!(format!("{table_token2}"), "Tk:Table [3x3 +3 headers]");

        let mut table2 = Table::default();
        table2.add_row(
            ["1", "2", "3"]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );
        table2.add_row(
            ["2", "4", "6"]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );

        let table2_token = Token::Table(table2.clone());
        assert_eq!(format!("{table2}"), "Table [3x2]");
        assert_eq!(format!("{table2_token}"), "Tk:Table [3x2]");
    }
}
