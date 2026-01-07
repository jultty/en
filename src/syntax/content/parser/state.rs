use std::collections::HashMap;

use crate::syntax::content::parser::{
    context::{Block, Context, Inline},
    token::{anchor::Anchor, item::Item, list::List},
};

#[derive(Clone, Debug)]
pub struct State {
    pub context: Context,
    pub dom_ids: HashMap<String, Vec<String>>,
    pub switches: Switches,
    pub buffers: Buffers,
}

#[derive(Clone, Debug)]
pub struct Switches {
    pub bold: bool,
    pub oblique: bool,
    pub crossout: bool,
    pub underline: bool,
}

#[derive(Clone, Debug)]
pub struct Buffers {
    pub anchor: AnchorBuffer,
    pub list: ListBuffer,
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

impl Default for State {
    fn default() -> State {
        State {
            context: Context {
                inline: Inline::None,
                block: Block::None,
            },
            dom_ids: HashMap::default(),
            switches: Switches {
                bold: false,
                crossout: false,
                oblique: false,
                underline: false,
            },
            buffers: Buffers {
                anchor: AnchorBuffer {
                    candidate: Anchor::default(),
                    text: String::default(),
                    destination: String::default(),
                },
                list: ListBuffer {
                    candidate: List::default(),
                    item_candidate: Item::default(),
                    depth: 0,
                },
            },
        }
    }
}
