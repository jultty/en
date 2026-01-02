use crate::{prelude::*, syntax::content::parser::segment::delimiter::Delimiters};

#[derive(Clone, Debug)]
pub struct Lexeme {
    text: String,
    next: String,
    last: bool,
}

impl Lexeme {
    pub fn new(raw: &str, next: &str) -> Lexeme {
        Lexeme {
            text: raw.to_owned(),
            next: next.to_owned(),
            last: false,
        }
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn next(&self) -> String {
        if self.next.is_empty() && !self.last {
            log!("Returning an empty string for next of non-last {self:?}");
        }
        self.next.clone()
    }

    pub fn last(&self) -> bool {
        self.last
    }

    pub fn mutate_text(&mut self, new: &str) {
        self.text = new.to_string();
    }

    pub fn as_char(&self) -> Option<char> {
        if self.text.chars().count() == 1 {
            self.text.chars().nth(0)
        } else {
            None
        }
    }

    pub fn next_as_char(&self) -> Option<char> {
        if self.next.chars().count() == 1 {
            self.next.chars().nth(0)
        } else {
            None
        }
    }

    pub fn match_as_char(&self, c: char) -> bool {
        self.as_char().is_some_and(|as_char| as_char == c)
    }

    pub fn is_punctuation(&self) -> bool {
        let punctuation = Delimiters::default().punctuation;
        self.as_char().is_some_and(|c| punctuation.contains(&c))
    }

    pub fn is_whitespace(&self) -> bool {
        let delimiters = Delimiters::default();
        self.as_char()
            .is_some_and(|c| delimiters.whitespace.contains(&c))
    }

    pub fn is_next_whitespace(&self) -> bool {
        let delimiters = Delimiters::default();
        self.next_as_char()
            .is_some_and(|c| delimiters.whitespace.contains(&c))
    }

    pub fn is_next_punctuation(&self) -> bool {
        let delimiters = Delimiters::default();
        self.next_as_char()
            .is_some_and(|c| delimiters.punctuation.contains(&c))
    }

    pub fn is_next_boundary(&self) -> bool {
        let delimiters = Delimiters::default();
        self.next_as_char()
            .is_some_and(|c| delimiters.is_boundary(c))
    }

    pub fn next_first_char(&self) -> Option<char> {
        self.next.chars().nth(0)
    }

    pub fn match_first_char(&self, query: char) -> bool {
        self.text.chars().nth(0).is_some_and(|c| c == query)
    }

    pub fn match_last_char(&self, query: char) -> bool {
        self.text.chars().last().is_some_and(|c| c == query)
    }

    pub fn match_next_first_char(&self, query: char) -> bool {
        self.next.chars().nth(0).is_some_and(|c| c == query)
    }

    /// # Panics
    /// Panics if number of chars for a single lexeme exceeds `i32::MAX`
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
            let mut next = String::new();
            let mut last = false;
            if let Some(peeked) = iterator.peek() {
                next.clone_from(*peeked);
            } else {
                last = true;
            }
            out_vector.push(Lexeme {
                text: raw.to_owned(),
                next,
                last,
            });
        }

        out_vector
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_lexeme() {
        let raw = "3PKK4RzfGgUL58rU2NZbAiGN1o5dOfNu";
        let next = "wAcZe8iVEEcZLp20PP9KKf07zJbeZafa";
        let lexeme = Lexeme::new(raw, next);
        assert_eq!(lexeme.text, raw);
        assert_eq!(lexeme.next, next);
    }

    #[test]
    fn next_first_char() {
        let payload = "4IU";
        let lexeme = Lexeme::new(payload, payload);
        assert_eq!(lexeme.next_first_char().unwrap(), '4');
    }

    #[test]
    fn match_first_char() {
        let payload = "MKY";
        let lexeme = Lexeme::new(payload, payload);
        assert!(lexeme.match_first_char('M'));
    }

    #[test]
    fn match_absent_first_char() {
        let payload = "";
        let lexeme = Lexeme::new(payload, payload);
        assert!(!lexeme.match_first_char('x'));
    }

    #[test]
    fn first_word() {
        let payload = "nhNc fGev QnGW E4hj ExyZ";
        let lexeme = Lexeme::new(payload, payload);
        assert_eq!(lexeme.first(), Some(String::from("nhNc")));
    }

    #[test]
    fn count_char() {
        let payload = "6Ur3UjnndhENjFNSYWF7bhej2NZKLwdY";
        let lexeme = Lexeme::new(payload, payload);
        assert_eq!(lexeme.count_char('j'), 3);
    }

    #[test]
    fn count_char_huge_number() {
        let payload = "6Ur3UjnndhENjFNSYWF7bhej2NZKLwdY";
        let lexeme = Lexeme::new(payload, payload);
        assert_eq!(lexeme.count_char('j'), 3);
    }
}
