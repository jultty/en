use std::{backtrace::Backtrace, env, time::Instant};

pub use level::*;

mod level;

/// Strings in this slice suppress logging if found in the stack trace.
pub const EXCLUSIONS: &[&str] = &["en::graph::Graph::parse_config"];

#[derive(Debug)]
pub struct Data {
    pub env_level: Level,
    pub exclude: String,
    pub filter: String,
    pub message_level: Level,
    pub path: String,
    pub should_log: bool,
    pub trace: std::backtrace::Backtrace,
}

impl Data {
    pub fn new(
        message_level_opt: Option<Level>,
        captured_path: &str,
        trace: Backtrace,
    ) -> Data {
        let trace_string = format!("{trace:?}");
        let filter = env::var("DEBUG_FILTER").unwrap_or_default();
        let exclude = env::var("DEBUG_EXCLUDE").unwrap_or_default();
        let env_level = env_level();
        let message_level = message_level_opt.unwrap_or(MESSAGE_DEFAULT);
        let path = make_display_path(captured_path, &env_level);

        let is_silent = env_level <= Level::SILENT;
        let level_within_env_level = message_level <= env_level;
        let excluded_in_code =
            !EXCLUSIONS.iter().all(|&s| !trace_string.contains(s));
        let excluded_by_env =
            !exclude.is_empty() && !trace_string.contains(&exclude);
        let matches_filter =
            filter.is_empty() || captured_path.contains(&filter);

        let should_log = !is_silent
            && level_within_env_level
            && !excluded_in_code
            && !excluded_by_env
            && matches_filter;

        #[expect(clippy::print_stderr)]
        if env_level == Level::META {
            eprintln!(
                "Log decision for message from {path}: {should_log} given\n\
                is_silent: {is_silent} (expected false)\n\
                level_within_env_level: {level_within_env_level}\n\
                excluded_in_code: {excluded_in_code} (expected false)\n\
                excluded_by_env: {excluded_by_env} (expected false)\n\
                matches_filter: {matches_filter}\n\
            "
            );
        }

        Data {
            env_level,
            exclude,
            filter,
            message_level,
            path,
            should_log,
            trace,
        }
    }
}

pub fn env_level() -> Level {
    if let Ok(level) = env::var("DEBUG") {
        Level::from(level.as_str())
    } else {
        ENV_DEFAULT
    }
}

#[expect(clippy::print_stderr)]
pub fn print_state() {
    let env_level = env_level();
    let version = env!("CARGO_PKG_VERSION");
    if env_level == ENV_DEFAULT {
        eprintln!("en {version}");
    } else {
        eprintln!("en {version} [logging level {env_level}]");
    }
}

#[expect(clippy::print_stderr)]
pub fn timed(past: &Instant, message: &str) -> Instant {
    let now = Instant::now();
    let env_level = env_level();
    let duration = now.duration_since(*past);
    let display_duration = if duration.as_millis() > 1000 {
        format!("{}s {}ms", duration.as_secs(), duration.subsec_millis())
    } else if duration.as_millis() <= 1 {
        if env_level < Level::VERBOSE {
            return now;
        }
        format!("{}ns", duration.as_nanos())
    } else {
        format!("{}ms", duration.as_millis())
    };
    if !message.is_empty() && Level::DEBUG <= env_level {
        eprintln!("[tlog] +{display_duration} {message}");
    }
    now
}

#[macro_export]
macro_rules! tlog {
    ($instant:expr, $fmt:expr $(, $($arg:tt)+ )?) => {{
        $crate::dev::log::timed($instant, &format!($fmt $(, $($arg)+ )?))
    }};
}

pub fn now() -> Instant { Instant::now() }

#[expect(clippy::print_stderr, clippy::use_debug)]
pub fn elog(function: &str, message: &str) {
    eprintln!("{:?} [{function}] {message}", crate::ONSET.elapsed());
}

#[macro_export]
macro_rules! log {
    ($level:path, $fmt:expr $(, $($arg:tt)+ )?) => {{

         let data = $crate::dev::log::Data::new(
            Some($level),
            std::any::type_name_of_val(&|| {}),
            std::backtrace::Backtrace::capture(),
        );

        if data.should_log {
            $crate::dev::log::elog(&data.path, &format!($fmt $(, $($arg)+ )?));
        }
    }};
    ($fmt:expr $(, $($arg:tt)+ )?) => {{

        let data = $crate::dev::log::Data::new(
            None,
            std::any::type_name_of_val(&|| {}),
            std::backtrace::Backtrace::capture(),
        );

        if data.should_log {
            $crate::dev::log::elog(&data.path, &format!($fmt $(, $($arg)+ )?));
        };

    }};
}

pub fn make_display_path(type_path: &str, env_level: &Level) -> String {
    let mut path = type_path.to_string().replace("::{{closure}}", "");

    if let Some(s) = path
        .split(" as ")
        .next()
        .map(|parent| parent.replace(['<', '>'], ""))
        .and_then(|parent| {
            path.split(" as ")
                .nth(1)
                .and_then(|s| s.split("::").last())
                .map(|caller| format!("{parent}::{caller}"))
        })
    {
        path = s;
    }

    let path_vec: Vec<&str> = path.split("::").collect();

    if let (Some(last), Some(second_to_last), Some(third_to_last)) = (
        path_vec.get(path_vec.len().saturating_sub(1)),
        path_vec.get(path_vec.len().saturating_sub(2)),
        path_vec.get(path_vec.len().saturating_sub(3)),
    ) {
        if *env_level > Level::VERBOSE {
            path.clone()
        } else if *env_level > Level::DEBUG {
            format!("{third_to_last}::{second_to_last}::{last}")
        } else if *env_level >= ENV_DEFAULT {
            format!("{second_to_last}::{last}")
        } else {
            String::from(*last)
        }
    } else {
        path.clone()
    }
}

pub fn wrap(s: &str) -> String {
    fn symbolize(s: &str) -> String {
        if s == r"\n" {
            String::from('↳')
        } else {
            String::from(s)
        }
    }

    fn quote(s: &str) -> String {
        if s.contains(' ') {
            format!("'{s}'")
        } else {
            String::from(s)
        }
    }

    fn escape(s: &str) -> String { s.escape_debug().collect() }

    symbolize(&quote(&escape(s)))
}

#[macro_export]
macro_rules! write_log {
    ($buffer:expr, $format_string:expr $(, $format_args:expr)* $(,)?) => {{
        use std::fmt::Write as _;
        let result = write!($buffer, $format_string $(, $format_args)*);
        if let Err(error) = result {
            log!(ERROR, "Unexpected error writing into {}: ${error}", $buffer);
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_newline() {
        assert_eq!(wrap("\n"), String::from("↳"));
    }

    #[test]
    fn wrap_space() {
        assert_eq!(wrap(" "), String::from("' '"));
    }

    #[test]
    fn wrap_spaces() {
        assert_eq!(wrap("  "), String::from("'  '"));
    }

    #[test]
    fn wrap_containing_space() {
        assert_eq!(wrap("< "), String::from("'< '"));
    }

    fn run_in_debug_level(level: &str) {
        #[expect(unsafe_code)]
        unsafe {
            std::env::set_var("DEBUG", level);
            log!("Debug is set to {level}");
        }
    }

    #[test]
    fn debug_var_set() {
        for level in 0..9 {
            run_in_debug_level(&level.to_string());
        }
        run_in_debug_level("");
        run_in_debug_level("駄目！");
    }

    #[test]
    fn trait_stripping() {
        pub trait Loggable {
            fn test(&self);
        }

        struct Logger;

        impl Loggable for Logger {
            fn test(&self) {
                log!("This is inside a trait implementation");
            }
        }

        let logger = Logger {};
        logger.test();
    }
}
