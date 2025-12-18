use std::path::PathBuf;

use crate::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct Arguments {
    pub hostname: String,
    pub port: u16,
    pub graph_path: PathBuf,
}

impl Arguments {
    pub fn make_address(&self) -> String {
        format!("{}:{}", self.hostname, self.port)
    }

    pub fn new() -> Arguments {
        Arguments {
            hostname: String::from("0.0.0.0"),
            port: 0,
            graph_path: PathBuf::from("./static/graph.toml"),
        }
    }

    #[must_use]
    pub fn parse(&self) -> Arguments {
        let args: Vec<String> = std::env::args().collect();
        parse(self, &args)
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
                log!("Dropped unrecognized argument {argument}");
            }
        } else {
            panic!("Argument {arg:?} has no corresponding value")
        }
    }
    out_args
}
