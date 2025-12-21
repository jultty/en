pub fn elog(function: &str, message: &str) {
    eprintln!("{:?} [{function}] {message}", crate::ONSET.elapsed());
}

#[macro_export]
macro_rules! log {
        ($fmt:expr $(, $($arg:tt)+ )? ) => {{
        let mut display_path = String::new();
        let mut path = std::any::type_name_of_val(&|| {})
            .to_string().replace("::{{closure}}", "");

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

        if filter == "any" || filter.is_empty() || path.contains(&filter) {
            $crate::dev::elog(&display_path, &format!($fmt $(, $($arg)+ )?));
        };

    }};
}
