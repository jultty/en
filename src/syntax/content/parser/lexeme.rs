#[derive(Clone, Debug)]
pub struct Lexeme {
    text: String,
    pub next: String,
}

impl Lexeme {
    pub fn new(raw: &str, next: &str) -> Lexeme {
        Lexeme {
            text: raw.to_owned(),
            next: next.to_owned(),
        }
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// # Panics
    /// Panics if number of chars for a single lexeme exceeds `i2::MAX`
    pub fn count_char(&self, c: char) -> i32 {
        let count = self.text().chars().filter(|&n| n == c).count();
        match i32::try_from(count) {
            Ok(i) => i,
            Err(e) => {
                panic!("Wild char number {count} is a bit much: {e:#?}");
            },
        }
    }

    pub fn split_chars(&self) -> Vec<char> {
        let vector: Vec<char> = self.text().chars().collect();
        vector
    }

    pub fn split_words(self) -> Vec<String> {
        self.text().split(' ').map(str::to_string).collect()
    }

    pub fn first(self) -> Option<String> {
        self.split_words().first().map(String::to_owned)
    }

    pub fn collect(raw_strings: &[String]) -> Vec<Lexeme> {
        let mut out_vector = Vec::with_capacity(raw_strings.len());
        let mut iterator = raw_strings.iter().peekable();

        while let Some(raw) = iterator.next() {
            let next =
                iterator.peek().map(|s| (*s).clone()).unwrap_or_default();
            out_vector.push(Lexeme {
                text: raw.to_owned(),
                next,
            });
        }

        out_vector
    }
}
