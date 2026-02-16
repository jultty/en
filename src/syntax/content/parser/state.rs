use std::collections::HashMap;

use crate::syntax::content::parser::{
    Token,
    context::Context,
    token::{Anchor, Item, List, Quote, Table},
};

#[derive(Clone, Default, Debug)]
pub struct State {
    pub context: Context,
    pub dom_ids: HashMap<String, Vec<String>>,
    pub switches: Switches,
    pub buffers: Buffers,
    pub format_tokens: Vec<Token>,
}

#[derive(Clone, Default, Debug)]
pub struct Switches {
    pub bold: bool,
    pub oblique: bool,
    pub crossout: bool,
    pub underline: bool,
}

#[derive(Clone, Default, Debug)]
pub struct Buffers {
    pub anchor: AnchorBuffer,
    pub list: ListBuffer,
    pub quote: QuoteBuffer,
    pub table: TableBuffer,
}

#[derive(Default, Clone, Debug)]
pub struct ListBuffer {
    pub candidate: List,
    pub item_candidate: Item,
    pub depth: u8,
}

#[derive(Default, Clone, Debug)]
pub struct AnchorBuffer {
    pub candidate: Anchor,
    pub text: String,
    pub destination: String,
}

#[derive(Default, Clone, Debug)]
pub struct QuoteBuffer {
    pub candidate: Quote,
    pub in_citation: bool,
}

#[derive(Default, Clone, Debug)]
pub struct TableBuffer {
    pub candidate: Table,
    pub cell: String,
    pub in_cell: bool,
    pub in_header: bool,
}

impl std::fmt::Display for AnchorBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_text = if self.text.is_empty() {
            String::default()
        } else {
            format!("text: {:?}", self.text)
        };
        let display_destination = if self.destination.is_empty() {
            String::default()
        } else {
            format!(", dest: {:?}", self.destination)
        };

        let display_text_and_destination =
            format!("{display_text}{display_destination}");

        write!(
            f,
            "AnchorBuffer [{display_text_and_destination}] >> {}",
            self.candidate,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_buffer_display() {
        let buffer = AnchorBuffer::default();
        println!("{buffer:#?}");
        println!("{buffer}");
        assert_eq!(
            format!("{buffer}"),
            "AnchorBuffer [] >> Anchor <empty> -> <unknown>"
        );
    }

    #[test]
    fn anchor_buffer_display_with_text_set() {
        let mut buffer = AnchorBuffer::default();
        buffer.text = String::from("mX8Z7yWmsK");
        println!("{buffer:#?}");
        println!("{buffer}");
        assert_eq!(
            format!("{buffer}"),
            r#"AnchorBuffer [text: "mX8Z7yWmsK"] >> Anchor <empty> -> <unknown>"#
        );
    }

    #[test]
    fn anchor_buffer_display_with_destination_set() {
        let mut buffer = AnchorBuffer::default();
        buffer.destination = String::from("VP2aqGngAq");
        println!("{buffer:#?}");
        println!("{buffer}");
        assert_eq!(
            format!("{buffer}"),
            r#"AnchorBuffer [, dest: "VP2aqGngAq"] >> Anchor <empty> -> <unknown>"#
        );
    }

    #[test]
    fn anchor_buffer_display_with_text_and_destination_set() {
        let mut buffer = AnchorBuffer::default();
        buffer.text = String::from("ECJrzgkBHg");
        buffer.destination = String::from("9dy6gQ2g3E");
        println!("{buffer:#?}");
        println!("{buffer}");
        assert_eq!(
            format!("{buffer}"),
            r#"AnchorBuffer [text: "ECJrzgkBHg", dest: "9dy6gQ2g3E"] >> Anchor <empty> -> <unknown>"#
        );
    }
}
