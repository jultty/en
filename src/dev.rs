#[allow(clippy::print_stderr)]
pub fn elog(function: &str, message: &str) {
    eprintln!("{:?} [{function}] {message}", crate::ONSET.elapsed());
}

// Paths in this slice suppress logging if found in the stack trace
pub const SKIP_PATHS: &[&str] = &["en::types::Config::parse_text"];

#[macro_export]
macro_rules! log {
        ($fmt:expr $(, $($arg:tt)+ )? ) => {{
        let mut display_path = String::default();
        let mut path = std::any::type_name_of_val(&|| {})
            .to_string().replace("::{{closure}}", "");

        let trace = format!("{:?}", std::backtrace::Backtrace::capture());

        let level: u8 = std::env::var("DEBUG")
            .unwrap_or("0".to_string()).trim().parse().unwrap_or(0);

        if path.matches("::").count() > 3 {

            if let Some(s) = path.split(" as ").next()
                .map(|parent| parent.replace(['<', '>'], ""))
                .and_then(|parent| { path.split(" as ").nth(1)
                    .and_then(|s| s.split("::").last())
                    .map(|caller| format!("{parent}::{caller}"))
                }) { path = s; }

            let path_vec: Vec<&str> = path.split("::").collect();

            if let (
                Some(last),
                Some(second_to_last),
                Some(third_to_last),
            ) = (
                path_vec.get(path_vec.len().saturating_sub(1)),
                path_vec.get(path_vec.len().saturating_sub(2)),
                path_vec.get(path_vec.len().saturating_sub(3)),
            ) {
                display_path = if level > 3 {
                    format!("{} -> {}", trace, path.clone())
                } else if level > 2 {
                    path.clone()
                } else if level > 0 {
                    format!("{third_to_last}::{second_to_last}::{last}")
                } else {
                    format!("{second_to_last}::{last}")
                };
            }
        } else {
            display_path = path.clone()
        };

        let filter = std::env::var("DEBUG_FILTER").unwrap_or("any".to_string());

        if $crate::dev::SKIP_PATHS.iter().all(|&s| !trace.contains(s)) &&
        (filter == "any" || filter.is_empty() || path.contains(&filter)) {
            $crate::dev::elog(&display_path, &format!($fmt $(, $($arg)+ )?));
        };

    }};
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

    fn escape(s: &str) -> String {
        s.escape_debug().collect()
    }

    symbolize(&quote(&escape(s)))
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
        #[allow(unsafe_code)]
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

        struct Logger {}

        impl Loggable for Logger {
            fn test(&self) {
                log!("This is inside a trait implementation");
            }
        }

        let logger = Logger {};
        logger.test();
    }
}
