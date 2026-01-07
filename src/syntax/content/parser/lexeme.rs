use std::fmt;

use crate::{prelude::*, syntax::content::parser::segment::delimiter::Delimiters};

#[derive(Clone, Debug, Default)]
pub struct Lexeme {
    text: String,
    next: String,
    third: String,
    last: bool,
}

impl Lexeme {
    pub fn new(raw: &str, next: &str, third: &str) -> Lexeme {
        Lexeme {
            text: raw.to_owned(),
            next: next.to_owned(),
            third: third.to_owned(),
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

    pub fn third_as_char(&self) -> Option<char> {
        if self.third.chars().count() == 1 {
            self.third.chars().nth(0)
        } else {
            None
        }
    }

    pub fn match_as_char(&self, c: char) -> bool {
        self.as_char().is_some_and(|as_char| as_char == c)
    }

    pub fn match_next_as_char(&self, c: char) -> bool {
        self.next_as_char().is_some_and(|next| next == c)
    }

    pub fn match_third_as_char(&self, c: char) -> bool {
        self.third_as_char().is_some_and(|third| third == c)
    }

    pub fn match_triple_as_char(&self, t: (char, char, char)) -> bool {
        self.match_as_char(t.0)
        && self.match_next_as_char(t.1)
        && self.match_third_as_char(t.2)
    }

    pub fn contains_as_char(&self, slice: &[char]) -> bool {
        self.as_char().is_some_and(|c| slice.contains(&c))
    }

    pub fn contains_next_as_char(&self, slice: &[char]) -> bool {
        self.next_as_char().is_some_and(|c| slice.contains(&c))
    }

    pub fn is_punctuation(&self) -> bool {
        self.contains_as_char(&Delimiters::default().punctuation)
    }

    pub fn is_whitespace(&self) -> bool {
        self.contains_as_char(&Delimiters::default().whitespace)
    }

    pub fn is_next_whitespace(&self) -> bool {
        self.contains_next_as_char(&Delimiters::default().whitespace)
    }

    pub fn is_next_punctuation(&self) -> bool {
        self.contains_next_as_char(&Delimiters::default().punctuation)
    }

    pub fn is_next_boundary(&self) -> bool {
        let delimiters = Delimiters::default();
        self.last
            || self
                .next_as_char()
                .is_some_and(|c| delimiters.is_boundary(c))
    }

    pub fn is_delimiter(&self) -> bool {
        let delimiters = Delimiters::default();
        self.as_char().is_some_and(|c| delimiters.is_delimiter(c))
    }

    pub fn is_next_delimiter(&self) -> bool {
        let delimiters = Delimiters::default();
        self.last
            || self
                .next_as_char()
                .is_some_and(|c| delimiters.is_delimiter(c))
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

    pub fn collect(segments: &[String]) -> Vec<Lexeme> {
        let mut out_vector = Vec::with_capacity(segments.len());
        let mut vec = segments.to_vec();

        let Some(mut third) = vec.pop() else { return vec![] };
        let last_lexeme = Lexeme {
            text: third.clone(),
            next: String::default(),
            third: String::default(),
            last: true,
        };

        let Some(mut next) = vec.pop() else { return vec![last_lexeme] };
        let penultimate_lexeme = Lexeme {
            text: next.clone(),
            next: third.clone(),
            third: String::default(),
            last: false,
        };

        for current in vec.iter().rev() {
            out_vector.push(Lexeme {
                text: current.to_owned(),
                next: next.clone(),
                third: third.clone(),
                last: false,
            });

            third.clone_from(&next);
            next.clone_from(current);
        }

        out_vector.reverse();
        out_vector.push(penultimate_lexeme);
        out_vector.push(last_lexeme);
        out_vector
    }
}

impl fmt::Display for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::dev::wrap;

        let next_display = if self.last() {
            " <EOI>"
        } else if self.third.is_empty() {
            &format!("-> {} -! EOI", wrap(&self.next))
        } else {
            &format!("-> {} -> {}", wrap(&self.next), wrap(&self.third))
        };
        write!(f, "{} {}", wrap(&self.text), next_display)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_lexeme() {
        let raw = "3PKK4RzfGgUL58rU2NZbAiGN1o5dOfNu";
        let next = "wAcZe8iVEEcZLp20PP9KKf07zJbeZafa";
        let third = "K0QTlujGjL2qxBzs16g8oyiCYSuQaRVE";
        let lexeme = Lexeme::new(raw, next, third);
        assert_eq!(lexeme.text, raw);
        assert_eq!(lexeme.next, next);
        assert_eq!(lexeme.third, third);
    }

    #[test]
    fn next_first_char() {
        let payload = "4IU";
        let lexeme = Lexeme::new("", payload, "");
        assert_eq!(lexeme.next_first_char().unwrap(), '4');
    }

    #[test]
    fn match_first_char() {
        let payload = "MKY";
        let lexeme = Lexeme::new(payload, "", "");
        assert!(lexeme.match_first_char('M'));
    }

    #[test]
    fn match_absent_first_char() {
        let lexeme = Lexeme::new("", "", "");
        assert!(!lexeme.match_first_char('x'));
    }

    #[test]
    fn first_word() {
        let payload = "nhNc fGev QnGW E4hj ExyZ";
        let lexeme = Lexeme::new(payload, "", "");
        assert_eq!(lexeme.first(), Some(String::from("nhNc")));
    }

    #[test]
    fn count_char() {
        let payload = "6Ur3UjnndhENjFNSYWF7bhej2NZKLwdY";
        let lexeme = Lexeme::new(payload, "", "");
        assert_eq!(lexeme.count_char('j'), 3);
    }
}
