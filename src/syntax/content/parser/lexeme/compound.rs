#[derive(Clone)]
pub struct Compound {
    pub raw: String,
}

impl Compound {
    pub fn new(text: &str) -> Compound {
        Compound {
            raw: text.to_owned(),
        }
    }
}
