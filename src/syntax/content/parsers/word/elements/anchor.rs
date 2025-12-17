// use std::fmt::Display;
// use crate::syntax::content::{Parseable, Line};
//
// pub struct Anchor {
//     text: String,
//     destination: String,
// }
//
// impl Parseable for Anchor {
//     fn probe(line: &Line) -> bool {
//         let candidate = line.raw.split(' ');
//         !line.first.trim().is_empty()
//             && line.first.replace("#", "").is_empty()
//             && line.first.len() <= 6
//     }
//
//     fn lex(line: &Line) -> Self {
//         Self {
//             text: line.raw.trim().to_owned(),
//             destination:  t
//         }
//     }
//
//     fn render(&self) -> String {
//         format!(r#"<a href="{}">{}</a>"#, &self.destination, &self.text)
//     }
// }
//
// impl Display for Anchor {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "Anchor: <{}>", &self.text)
//     }
// }
