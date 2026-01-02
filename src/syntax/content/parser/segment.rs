pub fn segment(text: &str) -> Vec<String> {
    delimiter::atomize(text)
}

pub mod delimiter {

    pub struct Delimiters {
        pub atomic: Vec<char>,
        pub boundary: Vec<char>,
        pub flanking: Vec<char>,
        pub punctuation: Vec<char>,
        pub whitespace: Vec<char>,
    }

    impl Default for Delimiters {
        fn default() -> Self {
            let atomic = vec!['`', '|'];
            let flanking = vec!['_', '*'];
            let punctuation = vec![',', '.', ';', ':', '?', '!'];
            let whitespace = vec!['\n', ' '];

            let boundary =
                [atomic.clone(), punctuation.clone(), whitespace.clone()]
                    .concat();

            Delimiters {
                atomic,
                boundary,
                flanking,
                punctuation,
                whitespace,
            }
        }
    }

    impl Delimiters {
        pub fn is_boundary(&self, c: char) -> bool {
            [
                self.atomic.clone(),
                self.punctuation.clone(),
                self.whitespace.clone(),
            ]
            .concat()
            .contains(&c)
        }

        fn is_str_delimiter(&self, s: &str) -> bool {
            if s.chars().count() > 1 {
                return false;
            }
            if let Some(c) = s.chars().nth(0) {
                self.boundary.contains(&c) || self.flanking.contains(&c)
            } else {
                false
            }
        }
    }

    pub fn atomize(text: &str) -> Vec<String> {
        let delimiters = Delimiters::default();
        let mut atomized: Vec<String> = vec![];

        let mut iterator = text.chars().peekable();
        while let Some(c) = iterator.next() {
            // if the current char is an atomic delimiter
            if delimiters.atomic.contains(&c) {
                atomized.push(c.to_string());

            // if the current char is a flanking delimiter
            } else if delimiters.flanking.contains(&c) {
                // if next char is a boundary
                if iterator
                    .peek()
                    .is_some_and(|next| delimiters.is_boundary(*next))
                {
                    atomized.push(c.to_string());

                // if the previous char was whitespace
                } else if let Some(last_string) = atomized.last()
                    && let Some(last_char) = last_string.chars().last()
                    && last_char.is_whitespace()
                {
                    atomized.push(c.to_string());
                }

            // if there is a last atomized element
            } else if let Some(last) = atomized.last_mut() {
                // if the last atomized element is a delimiter
                if delimiters.is_delimiter(last) {
                    atomized.push(c.to_string());
                } else {
                    last.push(c);
                }

            // if there is no last atomized element
            } else {
                atomized.push(c.to_string());
            }
        }
        atomized
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn atomize_words() {
            let words = "    justification for  the actions   of those  who hold authority   inevitably dwindles  "; // 2
            let actual = atomize(words);
            let expected = vec![
                " ",
                " ",
                " ",
                " ",
                "justification",
                " ",
                "for",
                " ",
                " ",
                "the",
                " ",
                "actions",
                " ",
                " ",
                " ",
                "of",
                " ",
                "those",
                " ",
                " ",
                "who",
                " ",
                "hold",
                " ",
                "authority",
                " ",
                " ",
                " ",
                "inevitably",
                " ",
                "dwindles",
                " ",
                " ",
            ];
            assert_eq!(actual, expected);
        }

        #[test]
        fn atomize_ticks_no_spaces() {
            let s = "a`c`adc`dadcdbd`cdb`dcdb`dc`dad`bdc";
            let actual = atomize(s);
            let expected = vec![
                "a", "`", "c", "`", "adc", "`", "dadcdbd", "`", "cdb", "`",
                "dcdb", "`", "dc", "`", "dad", "`", "bdc",
            ]
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();

            assert_eq!(actual, expected);
        }

        #[test]
        fn atomize_ticks_with_spaces() {
            let s = "a`c`adc`da dcdb d` cdb` dcdb `dc ` d ad ` bdc";

            let actual = atomize(s);
            let expected = vec![
                "a", "`", "c", "`", "adc", "`", "da", " ", "dcdb", " ", "d",
                "`", " ", "cdb", "`", " ", "dcdb", " ", "`", "dc", " ", "`",
                " ", "d", " ", "ad", " ", "`", " ", "bdc",
            ]
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();
            assert_eq!(actual, expected);
        }

        #[test]
        fn atomize_pipes() {
            let s = "every other |time| as it was perceived";
            let actual = atomize(s);
            let expected = vec![
                "every",
                " ",
                "other",
                " ",
                "|",
                "time",
                "|",
                " ",
                "as",
                " ",
                "it",
                " ",
                "was",
                " ",
                "perceived",
            ];
            assert_eq!(actual, expected);
        }

        #[test]
        fn atomize_pipes_and_ticks() {
            let s = "every other |time| as `it could or |perhaps somehow|then or now| it was` perceived";
            let actual = atomize(s);
            let expected = vec![
                "every",
                " ",
                "other",
                " ",
                "|",
                "time",
                "|",
                " ",
                "as",
                " ",
                "`",
                "it",
                " ",
                "could",
                " ",
                "or",
                " ",
                "|",
                "perhaps",
                " ",
                "somehow",
                "|",
                "then",
                " ",
                "or",
                " ",
                "now",
                "|",
                " ",
                "it",
                " ",
                "was",
                "`",
                " ",
                "perceived",
            ];
            assert_eq!(actual, expected);
        }

        #[test]
        fn atomize_newlines() {
            let s = "a`c`adc`da \ndcdb d` cdb` dc\ndb `dc ` d ad ` bdc";

            let actual = atomize(s);
            let expected = vec![
                "a", "`", "c", "`", "adc", "`", "da", " ", "\n", "dcdb", " ",
                "d", "`", " ", "cdb", "`", " ", "dc", "\n", "db", " ", "`",
                "dc", " ", "`", " ", "d", " ", "ad", " ", "`", " ", "bdc",
            ]
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();
            assert_eq!(actual, expected);
        }
    }
}
