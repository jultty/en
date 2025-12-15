use std::fmt::Write as _;
use crate::dev::log;

pub fn parse(text: &str) -> String {
    let mut out_text: Vec<String> = Vec::new();

    for line in text.lines() {
        if line.is_empty() || line.replace(" ", "").is_empty() {
            continue;
        }

        let mut out_line: String = line.to_owned();
        let words: Vec<String> = line.split(" ").map(str::to_string).collect();
        let first_word: &String =
            words.first().unwrap_or_else(|| unreachable!());

        if is_header(first_word) {
            out_line = parse_header(&out_line, first_word);
        }
        // if not special, default to treating line as a paragraph
        else {
            out_line.insert_str(0, "<p>");
            out_line.push_str("</p>");
        }

        out_text.push(out_line);
    }

    out_text.join("\n")
}

fn is_header(lexeme: &str) -> bool {
    !lexeme.trim().is_empty()
        && lexeme.replace("#", "").is_empty()
        && lexeme.len() <= 6
}

fn parse_header(line: &str, first_word: &str) -> String {
    log(&parse_header, &format!("Parsing: {line:?}"));

    let header_level = first_word.len();
    log(&parse, &format!("Header level is {header_level}"));
    let header_text = line.to_owned().replace(first_word, "");
    let mut w = String::with_capacity(header_text.len().strict_add(9));
    let alloc = w.capacity();
    match write!(w, "<h{header_level}>{header_text}</h{header_level}>") {
        Ok(()) => (),
        Err(e) => panic!("{e:?}"),
    }
    if alloc != w.capacity() {
        log(
            &parse_header,
            &format!("w reallocated to {} despite prediction", w.capacity()),
        );
    }
    w
}
