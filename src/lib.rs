use std::{sync, time};

pub mod prelude {
    pub use crate::{
        log,
        log::{Level::*, now},
        tlog, write_log,
    };
}

pub mod graph;
pub mod log;
pub mod router;
pub mod syntax;

pub static ONSET: sync::LazyLock<time::Instant> =
    sync::LazyLock::new(time::Instant::now);
