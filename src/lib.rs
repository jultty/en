use std::{sync, time};

pub mod prelude {
    pub use crate::log;
    pub use crate::tlog;
    pub use crate::log::Level::*;
    pub use crate::log::now;
}

pub mod graph;
pub mod router;
pub mod syntax;
pub mod log;

pub static ONSET: sync::LazyLock<time::Instant> =
    sync::LazyLock::new(time::Instant::now);
