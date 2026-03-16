use std::{sync, time};

pub mod prelude {
    pub use crate::{
        dev::log::{Level::*, now},
        log, tlog, write_log,
    };
}

pub mod dev;
pub mod graph;
pub mod router;
pub mod syntax;

pub static ONSET: sync::LazyLock<time::Instant> =
    sync::LazyLock::new(time::Instant::now);
