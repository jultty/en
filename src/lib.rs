use std::{sync, time};

pub mod prelude {
    pub use crate::log;
}

pub mod types;
pub mod router;
pub mod syntax;
pub mod dev;

pub static ONSET: sync::LazyLock<time::Instant> =
    sync::LazyLock::new(time::Instant::now);
