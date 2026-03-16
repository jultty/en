use std::fmt;

use crate::syntax::content::parser::segment::delimiter::Delimiters;

#[derive(Clone, Debug, Default)]
pub struct Lexeme {
    text: String,
    next: String,
    third: String,
    first: bool,
    last: bool,
}

impl Lexeme {
    pub fn new(raw: &str, next: &str, third: &str) -> Lexeme {
        Lexeme {
            text: raw.to_owned(),
            next: next.to_owned(),
            third: third.to_owned(),
            first: false,
            last: false,
        }
    }

    pub fn text(&self) -> String { self.text.clone() }

    pub fn next(&self) -> String { self.next.clone() }

    pub const fn last(&self) -> bool { self.last }

    pub const fn first(&self) -> bool { self.first }

    pub fn mutate_text(&mut self, new: &str) { self.text = new.to_string(); }

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

    pub fn match_char(&self, c: char) -> bool {
        self.as_char().is_some_and(|as_char| as_char == c)
    }

    pub fn match_next_char(&self, c: char) -> bool {
        self.next_as_char().is_some_and(|next| next == c)
    }

    pub fn match_third_char(&self, c: char) -> bool {
        self.third_as_char().is_some_and(|third| third == c)
    }

    pub fn match_either_char(&self, c1: char, c2: char) -> bool {
        self.as_char().is_some_and(|c| c == c1 || c == c2)
    }

    pub fn match_next_either_char(&self, c1: char, c2: char) -> bool {
        self.next_as_char().is_some_and(|c| c == c1 || c == c2)
    }

    pub fn match_char_sequence(&self, c1: char, c2: char) -> bool {
        self.match_char(c1) && self.match_next_char(c2)
    }

    pub fn match_char_triple(&self, c1: char, c2: char, c3: char) -> bool {
        self.match_char(c1)
            && self.match_next_char(c2)
            && self.match_third_char(c3)
    }

    pub fn match_char_in(&self, slice: &[char]) -> bool {
        self.as_char().is_some_and(|c| slice.contains(&c))
    }

    pub fn match_next_char_in(&self, slice: &[char]) -> bool {
        self.next_as_char().is_some_and(|c| slice.contains(&c))
    }

    pub fn is_punctuation(&self) -> bool {
        self.match_char_in(&Delimiters::default().punctuation)
    }

    pub fn is_whitespace(&self) -> bool {
        self.match_char_in(&Delimiters::default().whitespace)
    }

    pub fn is_next_whitespace(&self) -> bool {
        self.match_next_char_in(&Delimiters::default().whitespace)
    }

    pub fn is_next_punctuation(&self) -> bool {
        self.match_next_char_in(&Delimiters::default().punctuation)
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

    pub fn next_first_char(&self) -> Option<char> { self.next.chars().nth(0) }

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
    /// Panics if number of chars for a single lexeme exceeds `i32::MAX`.
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

    pub fn split_segments(self) -> Vec<String> {
        self.text().split(' ').map(str::to_string).collect()
    }

    pub fn first_segment(self) -> Option<String> {
        self.split_segments().first().map(String::to_owned)
    }

    pub fn collect(segments_slice: &[String]) -> Vec<Lexeme> {
        let mut lexemes = Vec::with_capacity(segments_slice.len());
        let mut segments = segments_slice.to_vec();

        let Some(last) = segments.pop() else {
            return vec![];
        };
        let last_lexeme = Lexeme {
            text: last.clone(),
            next: String::default(),
            third: String::default(),
            first: segments.is_empty(),
            last: true,
        };

        let Some(penultimate) = segments.pop() else {
            return vec![last_lexeme];
        };
        let penultimate_lexeme = Lexeme {
            text: penultimate.clone(),
            next: last.clone(),
            third: String::default(),
            first: false,
            last: false,
        };

        let mut third = last;
        let mut next = penultimate;

        let mut iterator = segments.iter().rev().peekable();
        while let Some(current) = iterator.next() {
            let lexeme = Lexeme {
                text: current.to_owned(),
                next: next.clone(),
                third: third.clone(),
                first: iterator.peek().is_none(),
                last: false,
            };

            lexemes.push(lexeme);

            third.clone_from(&next);
            next.clone_from(current);
        }

        lexemes.reverse();
        lexemes.push(penultimate_lexeme);
        lexemes.push(last_lexeme);
        lexemes
    }
}

impl fmt::Display for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::dev::log::wrap;

        let properties = if self.last && self.first {
            "[S] "
        } else if self.last {
            "[L] "
        } else if self.first {
            "[F] "
        } else {
            ""
        };

        let next_display = if self.last {
            " <EOI>"
        } else if self.third.is_empty() {
            &format!(" -> {} <EOI>", wrap(&self.next))
        } else {
            &format!(" -> {} -> {}", wrap(&self.next), wrap(&self.third))
        };
        write!(f, "Lx {}{}{}", properties, wrap(&self.text), next_display)
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
    fn first_segment() {
        let payload = "nhNc fGev QnGW E4hj ExyZ";
        let lexeme = Lexeme::new(payload, "", "");
        assert_eq!(lexeme.first_segment(), Some(String::from("nhNc")));
    }

    #[test]
    fn first_lexeme() {
        let input = ["h015r", "cvYde", "aw1Ui", "ASwew"].map(str::to_string);
        let lexemes = Lexeme::collect(&input);
        let first = lexemes.first().unwrap();
        assert!(first.clone().first());
        assert_eq!(first.text(), "h015r".to_string());
    }

    #[test]
    fn count_char() {
        let payload = "6Ur3UjnndhENjFNSYWF7bhej2NZKLwdY";
        let lexeme = Lexeme::new(payload, "", "");
        assert_eq!(lexeme.count_char('j'), 3);
    }

    #[test]
    fn mutate_text() {
        let mut lexeme = Lexeme::new("b71Je", "I6y3i", "LC8na");
        lexeme.mutate_text("qkjjK2");
        assert_eq!(lexeme.text(), "qkjjK2");
    }

    #[test]
    fn third_as_char() {
        let lexeme_a = Lexeme::new("1", "2", "3");
        assert_eq!(lexeme_a.third_as_char().unwrap(), '3');
        let lexeme_c = Lexeme::new("a", "b", "");
        assert!(lexeme_c.third_as_char().is_none());
    }

    #[test]
    fn match_third_char() {
        let lexeme = Lexeme::new("1", "2", "3");
        assert!(lexeme.match_third_char('3'));
    }

    #[test]
    fn match_next_either_char() {
        let lexeme = Lexeme::new("1", "2", "3");
        assert!(lexeme.match_next_either_char('x', '2'));
        assert!(lexeme.match_next_either_char('2', 'x'));
    }

    #[test]
    fn match_triple() {
        let lexeme = Lexeme::new("1", "2", "3");
        assert!(lexeme.match_char_triple('1', '2', '3'));
    }

    #[test]
    fn is_punctuation() {
        let delimiters = Delimiters::default();
        let mut lexemes: Vec<Lexeme> = vec![];
        for p in delimiters.punctuation {
            lexemes.push(Lexeme::new(&p.to_string(), "", ""));
        }
        for lexeme in lexemes {
            assert!(lexeme.is_punctuation());
        }
    }

    #[test]
    fn is_next_punctuation() {
        let delimiters = Delimiters::default();
        let mut lexemes: Vec<Lexeme> = vec![];
        for p in delimiters.punctuation {
            lexemes.push(Lexeme::new("", &p.to_string(), ""));
        }
        for lexeme in lexemes {
            assert!(lexeme.is_next_punctuation());
        }
    }

    #[test]
    fn match_last_char() {
        let lexeme = Lexeme::new("qYBWuNX", "", "");
        assert!(lexeme.match_last_char('X'));
    }

    #[test]
    fn match_next_last_char() {
        let lexeme = Lexeme::new("", "teDAqVx", "");
        assert!(lexeme.match_next_first_char('t'));
    }

    #[test]
    fn display() {
        let input = ["pcdA", "o32X", "kz2i", "79Lz"].map(str::to_string);
        let lexemes = Lexeme::collect(&input);

        let first = lexemes.first().unwrap();
        let second = &lexemes[1];
        let third = &lexemes[2];
        let last = lexemes.last().unwrap();

        assert_eq!(
            format!("{first}"),
            String::from("Lx [F] pcdA -> o32X -> kz2i"),
            "first"
        );
        assert_eq!(
            format!("{second}"),
            String::from("Lx o32X -> kz2i -> 79Lz"),
            "second"
        );
        assert_eq!(
            format!("{third}"),
            String::from("Lx kz2i -> 79Lz <EOI>"),
            "third"
        );
        assert_eq!(
            format!("{last}"),
            String::from("Lx [L] 79Lz <EOI>"),
            "last"
        );

        let input_single = ["9fOC"].map(str::to_string);

        let lexemes_single = Lexeme::collect(&input_single);
        let single = lexemes_single.first().unwrap();
        println!("{single:#?}");
        assert!(input_single.to_vec().len() == 1);
        assert_eq!(format!("{single}"), "Lx [S] 9fOC <EOI>");
    }
}
