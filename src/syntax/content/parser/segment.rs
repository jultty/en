pub fn segment(text: &str) -> Vec<String> {
    delimiter::atomize(text)
}

mod delimiter {

    fn make_delimiters() -> Vec<char> {
        vec!['\n', ' ', '`', '|']
    }

    pub fn atomize(text: &str) -> Vec<String> {
        let delimiters = make_delimiters();
        text.chars().fold(
            Vec::new(),
            |mut accumulator: Vec<String>, character| {
                if delimiters.contains(&character) {
                    accumulator.push(character.to_string());
                } else if let Some(last) = accumulator.last_mut() {
                    if delimiters
                        .iter()
                        .map(char::to_string)
                        .filter(|d| d == last)
                        .count()
                        > 0
                    {
                        accumulator.push(character.to_string());
                    } else {
                        last.push(character);
                    }
                } else {
                    accumulator.push(character.to_string());
                }
                accumulator
            },
        )
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
