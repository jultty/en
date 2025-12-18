#[derive(Clone)]
pub enum Lexeme {
    Compound(compound::Compound),
}

pub mod compound;

impl Lexeme {
    pub fn to_raw(&self) -> String {
        match *self {
            Lexeme::Compound(ref d) => d.raw.clone(),
        }
    }

    /// # Panics
    /// Panics if number of chars for a single lexeme exceeds `i2::MAX`
    pub fn count_char(&self, c: char) -> i32 {
        let count = self.to_raw().chars().filter(|&n| n == c).count();
        match i32::try_from(count) {
            Ok(i) => i,
            Err(e) => {
                panic!("Wild char number {count} is a bit much: {e:#?}");
            },
        }
    }

    pub fn split_chars(&self) -> Vec<char> {
        let vector: Vec<char> = self.to_raw().chars().collect();
        vector
    }

    pub fn split_words(self) -> Vec<String> {
        self.to_raw().split(' ').map(str::to_string).collect()
    }

    pub fn first(self) -> Option<String> {
        self.split_words().first().map(String::to_owned)
    }
}
