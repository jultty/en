use std::{sync, time};

pub mod formats;
pub mod types;
pub mod handlers;
pub mod syntax;
pub mod dev;

pub static ONSET: sync::LazyLock<time::Instant> =
    sync::LazyLock::new(time::Instant::now);
