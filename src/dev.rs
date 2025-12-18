pub fn elog(function: &str, message: &str) {
    eprintln!("{:?} [{function}] {message}", crate::ONSET.elapsed());
}

#[macro_export]
macro_rules! log {
        ($fmt:expr $(, $($arg:tt)+ )? ) => {{
        let mut scope = std::any::type_name_of_val(&|| {})
            .to_string().replace("::{{closure}}", "");

        if scope.matches("::").count() > 3 {
                let parts: Vec<&str> = scope.split("::").collect();
                if let (Some(module) , Some(caller)) =
                (parts.get(parts.len().saturating_sub(1)),
                    parts.get(parts.len().saturating_sub(2))) {
                    scope = format!("{module}::{caller}");
                }
        }

        $crate::dev::elog(&scope, &format!($fmt $(, $($arg)+ )?));
    }};
}
