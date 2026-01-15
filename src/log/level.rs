#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
#[repr(u16)]
pub enum Level {
    SILENT = 0,
    FATAL = 1,
    ERROR = 2,
    WARN = 3,
    INFO = 4,
    DEBUG = 5,
    VERBOSE = 6,
    TRACE = 7,
    META = 37,
}

pub const ENV_DEFAULT: Level = Level::WARN;
pub const MESSAGE_DEFAULT: Level = Level::DEBUG;

impl From<Level> for u16 {
    fn from(level: Level) -> u16 {
        match level {
            Level::SILENT => 0,
            Level::FATAL => 1,
            Level::ERROR => 2,
            Level::WARN => 3,
            Level::INFO => 4,
            Level::DEBUG => 5,
            Level::VERBOSE => 6,
            Level::TRACE => 7,
            Level::META => 37,
        }
    }
}

impl From<u16> for Level {
    fn from(numeric: u16) -> Level {
        if numeric == 0 {
            Level::SILENT
        } else if numeric == 1 {
            Level::FATAL
        } else if numeric == 2 {
            Level::ERROR
        } else if numeric == 3 {
            Level::WARN
        } else if numeric == 4 {
            Level::INFO
        } else if numeric == 5 {
            Level::DEBUG
        } else if numeric == 6 {
            Level::VERBOSE
        } else if numeric == 7 {
            Level::TRACE
        } else if numeric == 37 {
            Level::META
        } else {
            super::ENV_DEFAULT
        }
    }
}

impl From<&str> for Level {
    fn from(s: &str) -> Level {
        if s == "0" || s == "SILENT" || s == "silent" {
            Level::SILENT
        } else if s == "1" || s == "FATAL" || s == "fatal" {
            Level::FATAL
        } else if s == "2" || s == "ERROR" || s == "error" {
            Level::ERROR
        } else if s == "3"
            || s == "WARN"
            || s == "warn"
            || s == "WARNING"
            || s == "warning"
        {
            Level::WARN
        } else if s == "4" || s == "INFO" || s == "info" {
            Level::INFO
        } else if s == "5" || s == "DEBUG" || s == "debug" {
            Level::DEBUG
        } else if s == "6" || s == "VERBOSE" || s == "verbose" {
            Level::VERBOSE
        } else if s == "7" || s == "TRACE" || s == "trace" {
            Level::TRACE
        } else if s == "37" || s == "META" || s == "meta" {
            Level::META
        } else {
            super::ENV_DEFAULT
        }
    }
}
impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Level::SILENT => "SILENT",
            Level::FATAL => "FATAL",
            Level::ERROR => "ERROR",
            Level::WARN => "WARNING",
            Level::INFO => "INFO",
            Level::DEBUG => "DEBUG",
            Level::VERBOSE => "VERBOSE",
            Level::TRACE => "TRACE",
            Level::META => "META",
        };
        write!(f, "{s}")
    }
}
