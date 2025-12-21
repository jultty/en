use crate::prelude::*;

pub fn cluster(text: &str) -> Vec<String> {
    let words: Vec<String> = text
        .replace("\n", " \n ")
        .split(' ')
        .map(str::to_string)
        .collect();

    let mut clusters: Vec<String> = vec![];
    let mut raw_context = false;

    let mut iterator = words.into_iter().peekable();
    while let Some(word) = iterator.next() {
        log!("Iterating: {word:?}");

        if word == "`" {
            raw_context = !raw_context;
            log!("Raw context is now {raw_context}");
        } else if raw_context {
            log!("Skip: In raw context");
            clusters.push(word);
            continue;
        }

        let Some(delimiter) = delimiter::match_delimiter(&word) else {
            log!("Skip: {word:?} does not start with a delimiter");
            clusters.push(word);
            continue;
        };

        if let Some(next) = iterator.peek()
            && next == "\n"
            && delimiter.greedy
        {
            log!("Skip: Next {next:?} is a break, delimiter is greedy");
            clusters.push(word);
            continue;
        }

        if word.starts_with(&delimiter.string)
            && word.ends_with(&delimiter.string)
        {
            log!("Skip: {word:?} is atomically-delimited");
            clusters.push(word);
            continue;
        }

        if (!delimiter.greedy
            && !delimiter.triple
            && word.matches(delimiter.char).count() == 2)
            || (delimiter.triple && word.matches(delimiter.char).count() == 3)
        {
            log!("Skip: {word:?} is almost atomic, but must be split");
            match word.rsplit_once(delimiter.char) {
                Some((head, tail)) => {
                    log!("Pushing head {head:?}, tail {tail:?} into clusters");
                    clusters.push(format!("{head}{}", delimiter.char));
                    clusters.push(tail.to_string());
                    continue;
                },
                None => unreachable!(),
            }
        }

        log!("Found cluster from {delimiter:?} in {word:?}");
        let mut parts: Vec<String> = vec![word.clone()];
        log!("Seeking from a base of {parts:?}");

        while let Some(next) = iterator.peek() {
            if next.contains(&delimiter.char.to_string()) {
                log!("Found end of cluster: {next:?}");
                if delimiter.greedy
                    && delimiter.triple
                    && next.matches(delimiter.char).count() > 1
                {
                    match next.rsplit_once(delimiter.char) {
                        Some((head, tail)) => {
                            log!(
                                "Pushing head {head:?} of greedy triple EOC \
                                into parts and tail {tail:?} into clusters"
                            );
                            parts.push(format!("{head}{}", delimiter.char));
                            clusters.push(parts.join(" "));
                            clusters.push(tail.to_string());
                            log!("Breaking past clusters {clusters:?}");
                            iterator.next();
                            break;
                        },
                        None => unreachable!(),
                    }
                } else if delimiter.greedy {
                    log!("Pushing end of cluster into parts");
                    parts.push(
                        iterator.next().unwrap_or_else(|| unreachable!()),
                    );
                    log!("Pushing parts {parts:?} into clusters {clusters:?}");
                    clusters.push(parts.join(" "));
                    log!("Breaking past clusters {clusters:?}");
                    break;
                } else {
                    match next.rsplit_once(delimiter.char) {
                        Some((head, tail)) => {
                            log!(
                                "Pushing head {head:?} of humble end of \
                                cluster into parts"
                            );
                            parts.push(format!("{head}{}", delimiter.char));
                            log!("Pushing parts into clusters");
                            clusters.push(parts.join(" "));
                            log!("Pushing tail {tail:?} into clusters");
                            clusters.push(tail.to_string());
                            log!("Breaking past clusters");
                            iterator.next();
                            break;
                        },
                        // is this one really unreachable?
                        None => unreachable!(),
                    }
                }
            } else {
                log!("No delimiter: Pushing {:?} into parts", iterator.peek());
                parts.push(iterator.next().unwrap_or_default());
                log!("Seeking a boundary for parts {parts:?}");
            }
        }
    }

    log!("Returning clusters");
    clusters
}

mod delimiter {

    #[derive(Debug, Clone)]
    pub struct Delimiter {
        pub char: char,
        pub string: String,
        pub greedy: bool,
        pub triple: bool,
    }

    fn make_delimiters() -> Vec<Delimiter> {
        vec![
            Delimiter {
                char: '|',
                string: "|".to_string(),
                greedy: true,
                triple: true,
            },
            Delimiter {
                char: '`',
                string: "`".to_string(),
                greedy: false,
                triple: false,
            },
        ]
    }

    pub fn match_delimiter(word: &str) -> Option<Delimiter> {
        let first_char = word.chars().next()?;
        make_delimiters()
            .iter()
            .find(|d| d.char == first_char)
            .cloned()
    }
}
