pub fn segment(text: &str) -> Vec<String> {
    delimiter::atomize(text)
}

pub mod delimiter {

    pub struct Delimiters {
        pub atomic: Vec<char>,
        pub flanking: Vec<char>,
        pub punctuation: Vec<char>,
        pub whitespace: Vec<char>,
        pub double: Vec<char>,
    }

    impl Default for Delimiters {
        fn default() -> Self {
            Delimiters {
                atomic: vec!['`', '|', '\\'],
                double: vec!['_', '~'],
                flanking: vec!['_', '*', '~', '(', ')', '[', ']', '\'', '"'],
                punctuation: vec![',', '.', ';', ':', '?', '!'],
                whitespace: vec!['\n', ' '],
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

        pub fn is_delimiter(&self, c: char) -> bool {
            self.is_boundary(c) || self.flanking.contains(&c)
        }

        fn is_str_delimiter(&self, s: &str) -> bool {
            if s.chars().count() > 1 {
                return false;
            }
            if let Some(c) = s.chars().nth(0) {
                self.is_delimiter(c)
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
            // if current char is a boundary
            if delimiters.is_boundary(c) {
                atomized.push(c.to_string());
                continue;

            // if current char is a double delimiter and the next its double
            } else if delimiters.double.contains(&c)
                && iterator.peek().is_some_and(|n| *n == c)
            {
                atomized.push(c.to_string());
                iterator.next();
                atomized.push(c.to_string());
                continue;

            // if current char is a flanking delimiter
            } else if delimiters.flanking.contains(&c) {
                // if next char is a boundary
                if iterator
                    .peek()
                    .is_none_or(|next| delimiters.is_boundary(*next))
                {
                    atomized.push(c.to_string());
                    continue;

                // if previous char was whitespace
                } else if let Some(last_string) = atomized.last()
                    && let Some(last_char) = last_string.chars().last()
                    && delimiters.whitespace.contains(&last_char)
                {
                    atomized.push(c.to_string());
                    continue;
                }
            }

            // if there is a last atomized element
            if let Some(last) = atomized.last_mut() {
                // if last atomized element is a boundary
                if delimiters.is_str_delimiter(last) {
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
        fn atomize_nonflanking_underscore() {
            assert_eq!(atomize("false_dichotomy"), vec!["false_dichotomy"]);
        }

        #[test]
        fn atomize_left_flanking_underscore() {
            assert_eq!(
                atomize("_false_dichotomy"),
                vec!["_", "false_dichotomy"]
            );
        }

        #[test]
        fn atomize_right_flanking_underscore() {
            assert_eq!(
                atomize("false_dichotomy_"),
                vec!["false_dichotomy", "_"]
            );
        }

        #[test]
        fn atomize_dual_flanking_underscore() {
            assert_eq!(
                atomize("_false_dichotomy_"),
                vec!["_", "false_dichotomy", "_"]
            );
        }

        #[test]
        fn atomize_flankign_sentence() {
            assert_eq!(
                atomize(
                    "about_colors: the colors _amber_, _orange_ and _yellow mustard_ to `jane_bishop@mail.com`."
                ),
                vec![
                    "about_colors",
                    ":",
                    " ",
                    "the",
                    " ",
                    "colors",
                    " ",
                    "_",
                    "amber",
                    "_",
                    ",",
                    " ",
                    "_",
                    "orange",
                    "_",
                    " ",
                    "and",
                    " ",
                    "_",
                    "yellow",
                    " ",
                    "mustard",
                    "_",
                    " ",
                    "to",
                    " ",
                    "`",
                    "jane_bishop@mail",
                    ".",
                    "com",
                    "`",
                    "."
                ],
            );
        }

        #[test]
        fn atomize_words() {
            let actual = atomize(
                "    justification for  the actions   of those  who hold authority   inevitably dwindles  ",
            );
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
            let actual = atomize("every other |time| as it was perceived");
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
            let actual = atomize(
                "every other |time| as `it could or |perhaps somehow|then or now| it was` perceived",
            );
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
            let actual =
                atomize("a`c`adc`da \ndcdb d` cdb` dc\ndb `dc ` d ad ` bdc");
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
