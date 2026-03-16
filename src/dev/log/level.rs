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
        if s == "0" || s.to_uppercase() == "SILENT" {
            Level::SILENT
        } else if s == "1" || s.to_uppercase() == "FATAL" {
            Level::FATAL
        } else if s == "2" || s.to_uppercase() == "ERROR" {
            Level::ERROR
        } else if s == "3"
            || s.to_uppercase() == "WARN"
            || s.to_uppercase() == "WARNING"
        {
            Level::WARN
        } else if s == "4" || s.to_uppercase() == "INFO" {
            Level::INFO
        } else if s == "5" || s.to_uppercase() == "DEBUG" {
            Level::DEBUG
        } else if s == "6" || s.to_uppercase() == "VERBOSE" {
            Level::VERBOSE
        } else if s == "7" || s.to_uppercase() == "TRACE" {
            Level::TRACE
        } else if s == "37" || s.to_uppercase() == "META" {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u16_from_level() {
        assert_eq!(u16::from(Level::SILENT), 0);
        assert_eq!(u16::from(Level::FATAL), 1);
        assert_eq!(u16::from(Level::ERROR), 2);
        assert_eq!(u16::from(Level::WARN), 3);
        assert_eq!(u16::from(Level::INFO), 4);
        assert_eq!(u16::from(Level::DEBUG), 5);
        assert_eq!(u16::from(Level::VERBOSE), 6);
        assert_eq!(u16::from(Level::TRACE), 7);
        assert_eq!(u16::from(Level::META), 37);
    }

    #[test]
    fn level_from_u16() {
        assert_eq!(Level::from(0), Level::SILENT);
        assert_eq!(Level::from(1), Level::FATAL);
        assert_eq!(Level::from(2), Level::ERROR);
        assert_eq!(Level::from(3), Level::WARN);
        assert_eq!(Level::from(4), Level::INFO);
        assert_eq!(Level::from(5), Level::DEBUG);
        assert_eq!(Level::from(6), Level::VERBOSE);
        assert_eq!(Level::from(7), Level::TRACE);
        assert_eq!(Level::from(37), Level::META);
        assert_eq!(Level::from(99), Level::WARN);
    }

    #[test]
    fn level_from_str() {
        assert_eq!(Level::from("0"), Level::SILENT);
        assert_eq!(Level::from("SILENT"), Level::SILENT);
        assert_eq!(Level::from("silent"), Level::SILENT);
        assert_eq!(Level::from("SiLEnT"), Level::SILENT);

        assert_eq!(Level::from("1"), Level::FATAL);
        assert_eq!(Level::from("FATAL"), Level::FATAL);
        assert_eq!(Level::from("fatal"), Level::FATAL);
        assert_eq!(Level::from("FaTaL"), Level::FATAL);

        assert_eq!(Level::from("3"), Level::WARN);
        assert_eq!(Level::from("WARN"), Level::WARN);
        assert_eq!(Level::from("warn"), Level::WARN);
        assert_eq!(Level::from("WaRn"), Level::WARN);
        assert_eq!(Level::from("WARNING"), Level::WARN);
        assert_eq!(Level::from("warning"), Level::WARN);
        assert_eq!(Level::from("WaRninG"), Level::WARN);

        assert_eq!(Level::from("4"), Level::INFO);
        assert_eq!(Level::from("INFO"), Level::INFO);
        assert_eq!(Level::from("info"), Level::INFO);
        assert_eq!(Level::from("iNFo"), Level::INFO);

        assert_eq!(Level::from("5"), Level::DEBUG);
        assert_eq!(Level::from("DEBUG"), Level::DEBUG);
        assert_eq!(Level::from("debug"), Level::DEBUG);
        assert_eq!(Level::from("deBuG"), Level::DEBUG);

        assert_eq!(Level::from("6"), Level::VERBOSE);
        assert_eq!(Level::from("VERBOSE"), Level::VERBOSE);
        assert_eq!(Level::from("verbose"), Level::VERBOSE);
        assert_eq!(Level::from("VerBosE"), Level::VERBOSE);

        assert_eq!(Level::from("7"), Level::TRACE);
        assert_eq!(Level::from("TRACE"), Level::TRACE);
        assert_eq!(Level::from("trace"), Level::TRACE);
        assert_eq!(Level::from("trAcE"), Level::TRACE);

        assert_eq!(Level::from("37"), Level::META);
        assert_eq!(Level::from("META"), Level::META);
        assert_eq!(Level::from("meta"), Level::META);
        assert_eq!(Level::from("mETa"), Level::META);
    }

    #[test]
    fn display_level() {
        assert_eq!(format!("{}", Level::SILENT), "SILENT");
        assert_eq!(format!("{}", Level::FATAL), "FATAL");
        assert_eq!(format!("{}", Level::ERROR), "ERROR");
        assert_eq!(format!("{}", Level::WARN), "WARNING");
        assert_eq!(format!("{}", Level::INFO), "INFO");
        assert_eq!(format!("{}", Level::DEBUG), "DEBUG");
        assert_eq!(format!("{}", Level::VERBOSE), "VERBOSE");
        assert_eq!(format!("{}", Level::TRACE), "TRACE");
        assert_eq!(format!("{}", Level::META), "META");
    }
}
