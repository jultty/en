use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::prelude::*;

static FIRST_PARSE: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arguments {
    pub hostname: String,
    pub port: u16,
    pub graph_path: PathBuf,
}

impl Arguments {
    pub fn make_address(&self) -> String {
        format!("{}:{}", self.hostname, self.port)
    }

    #[must_use]
    pub fn parse(&self) -> Arguments {
        let args: Vec<String> = std::env::args().collect();
        parse(self, &args)
    }
}

impl Default for Arguments {
    fn default() -> Arguments {
        Arguments {
            hostname: String::from("0.0.0.0"),
            port: 0,
            graph_path: PathBuf::from("./static/graph.toml"),
        }
    }
}

fn parse(defaults: &Arguments, args: &[String]) -> Arguments {
    let mut out_args = defaults.clone();

    let filtered_args = if let Some((head, tail)) = args.split_first() {
        if head.starts_with('-') { args } else { tail }
    } else {
        args
    };

    for arg in filtered_args.chunks(2) {
        if let Some(argument) = arg.first()
            && let Some(parameter) = arg.get(1)
        {
            if argument.eq("-h") || argument.eq("--hostname") {
                out_args.hostname = String::from(parameter);
            } else if argument.eq("-p") || argument.eq("--port") {
                out_args.port = parameter.parse().unwrap_or(out_args.port);
            } else if argument.eq("-g") || argument.eq("--graph") {
                out_args.graph_path = PathBuf::from(parameter);
            } else {
                if FIRST_PARSE.load(Ordering::SeqCst) {
                    log!(WARN, "Dropped unrecognized argument {argument}");
                    FIRST_PARSE.store(false, Ordering::SeqCst);
                }
            }
        } else {
            panic!("Argument {arg:?} has no corresponding value")
        }
    }
    out_args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address() {
        let args = Arguments {
            hostname: String::from("localhost"),
            port: 3007,
            graph_path: PathBuf::default(),
        };

        assert_eq!(args.make_address(), "localhost:3007");
    }

    #[test]
    fn hostname() {
        let defaults = Arguments::default();

        let payload = String::from("olUCu7vWcUAsumv2xpj2Z55EDheWLTEu");
        let args =
            parse(&defaults, &[String::from("-h"), String::from(&payload)]);
        assert_eq!(args.hostname, payload);
    }

    #[test]
    fn port() {
        let defaults = Arguments::default();

        let payload = 3901;
        let args = parse(&defaults, &[String::from("-p"), payload.to_string()]);
        assert_eq!(args.port, payload);
    }

    #[test]
    fn graph_path() {
        let defaults = Arguments::default();

        let payload = PathBuf::from("/tmp/");
        let args = parse(
            &defaults,
            &[String::from("-g"), payload.to_str().unwrap().to_string()],
        );
        assert_eq!(args.graph_path, payload);
    }

    #[test]
    fn empty() {
        let defaults = Arguments::default();

        let args = parse(&defaults, &[]);
        assert_eq!(defaults, args);
    }
}
