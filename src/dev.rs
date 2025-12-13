pub fn log<F>(function: &F, message: &str) {
    eprintln!(
        "{:?} [{}] {message}",
        crate::ONSET.elapsed(),
        std::any::type_name_of_val(function).replace("en::", ""),
    );
}

