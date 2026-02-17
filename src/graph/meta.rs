use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Meta {
    pub config: Config,
    #[serde(default)]
    pub version: Version,
    #[serde(default)]
    pub messages: Vec<String>,
    #[serde(default)]
    pub malformed: bool,
}

impl Default for Meta {
    fn default() -> Meta {
        Meta {
            config: Config::default(),
            version: Version::from_compilation().unwrap_or_default(),
            messages: vec![],
            malformed: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Config {
    #[serde(default)]
    _private: bool,
    #[serde(default = "mktrue")]
    pub about: bool,
    #[serde(default)]
    pub about_text: String,
    #[serde(default = "mkfalse")]
    pub ascii_dom_ids: bool,
    #[serde(default)]
    pub content_language: String,
    #[serde(default = "mkfalse")]
    error_poem: bool,
    #[serde(default = "mktrue")]
    pub footer: bool,
    #[serde(default = "mktrue")]
    pub footer_credits: bool,
    #[serde(default = "mktrue")]
    pub footer_date: bool,
    #[serde(default)]
    pub footer_text: String,
    #[serde(default = "mk8")]
    pub index_node_count: u16,
    #[serde(default = "mktrue")]
    pub index_node_list: bool,
    #[serde(default = "mktrue")]
    pub index_root_node: bool,
    #[serde(default = "mktrue")]
    pub index_search: bool,
    #[serde(default)]
    node_selector: bool,
    #[serde(default)]
    navbar_search: bool,
    #[serde(default = "mktrue")]
    pub raw: bool,
    #[serde(default = "mktrue")]
    pub raw_json: bool,
    #[serde(default = "mktrue")]
    pub raw_toml: bool,
    #[serde(default)]
    pub site_description: String,
    #[serde(default)]
    pub site_title: String,
    #[serde(default = "mktrue")]
    pub tree: bool,
    #[serde(default = "mkfalse")]
    pub tree_node_summary: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            _private: true,
            about: true,
            about_text: String::default(),
            ascii_dom_ids: false,
            content_language: String::default(),
            error_poem: false,
            footer: true,
            footer_credits: true,
            footer_date: true,
            footer_text: String::default(),
            index_node_count: 8,
            index_node_list: true,
            index_root_node: true,
            index_search: true,
            node_selector: true,
            navbar_search: true,
            raw: true,
            raw_json: true,
            raw_toml: true,
            site_description: String::default(),
            site_title: String::default(),
            tree: true,
            tree_node_summary: false,
        }
    }
}

// See: https://github.com/serde-rs/serde/issues/368
const fn mktrue() -> bool { true }
const fn mkfalse() -> bool { false }
const fn mk8() -> u16 { 8 }

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

impl Default for Version {
    fn default() -> Version {
        match Version::from_compilation() {
            Ok(v) => v,
            Err(e) => {
                log!(ERROR, "{e}");
                Version {
                    major: 0,
                    minor: 0,
                    patch: 0,
                }
            },
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Version {
    /// Parses the compile-time version into a Version struct.
    ///
    /// # Errors
    /// This function is a thin wrapper around `Meta::from_text` and will
    /// return its errors without change.
    pub fn from_compilation() -> Result<Version, VersionError> {
        Version::from_text(env!("CARGO_PKG_VERSION"))
    }

    /// Parses a string into a Version struct.
    ///
    /// It is expected for the version string to contain exactly three
    /// dot-separated numeric values with an optional leading `v` character.
    ///
    /// # Errors
    /// Will error if the version string is malformed.
    pub fn from_text(version: &str) -> Result<Version, VersionError> {
        use VersionErrorCause::*;

        let triple: Vec<String> =
            version.split('.').map(str::to_string).collect();

        let has_two_dots = version.matches('.').count() == 2;
        let has_three_elements = triple.len() == 3;
        let has_whitespace = version.contains(' ') || version.contains('\n');
        let has_contiguous_dots = version.contains("..");
        let ends_with_dot = version.ends_with('.');
        let starts_with_dot = version.starts_with('.');

        let major: u8 = if let Some(s) = triple.first()
            && !s.is_empty()
        {
            match s.trim_start_matches('v').trim().parse() {
                Ok(parsed) => parsed,
                Err(e) => {
                    return Err(VersionError {
                        cause: FailedMajorParse,
                        message: Some(e.to_string()),
                    });
                },
            }
        } else {
            return Err(VersionError {
                cause: MissingMajor,
                message: None,
            });
        };

        let minor: u8 = if triple.len() >= 2
            && let Some(s) = triple.get(1)
        {
            match s.trim().parse() {
                Ok(parsed) => parsed,
                Err(e) => {
                    return Err(VersionError {
                        cause: FailedMinorParse,
                        message: Some(e.to_string()),
                    });
                },
            }
        } else {
            return Err(VersionError {
                cause: MissingMinor,
                message: None,
            });
        };

        let patch: u8 = if triple.len() >= 3
            && let Some(s) = triple.get(2)
        {
            match s.trim().parse() {
                Ok(parsed) => parsed,
                Err(e) => {
                    return Err(VersionError {
                        cause: FailedPatchParse,
                        message: Some(e.to_string()),
                    });
                },
            }
        } else {
            return Err(VersionError {
                cause: MissingPatch,
                message: None,
            });
        };

        let conditions = has_two_dots
            && has_three_elements
            && !has_whitespace
            && !has_contiguous_dots
            && !ends_with_dot
            && !starts_with_dot;

        if conditions {
            Ok(Version {
                major,
                minor,
                patch,
            })
        } else {
            Err(VersionError {
                cause: FailedValidation,
                message: Some(format!(
                    "Evaluated rules (all must be true):\n\
                    Has exactly two dots: {has_two_dots},\n\
                    Splits to three elements: {has_three_elements}\n\
                    Has no whitespace: {}\n\
                    Has no contiguous dots: {}\n\
                    Does not end with a dot: {}\n\
                    Does not start with a dot: {}",
                    !has_whitespace,
                    !has_contiguous_dots,
                    !ends_with_dot,
                    !starts_with_dot,
                )),
            })
        }
    }
}

#[derive(Debug)]
pub struct VersionError {
    cause: VersionErrorCause,
    message: Option<String>,
}

#[derive(Debug)]
enum VersionErrorCause {
    MissingMajor,
    FailedMajorParse,
    MissingMinor,
    FailedMinorParse,
    MissingPatch,
    FailedPatchParse,
    FailedValidation,
}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(message) = &self.message {
            write!(f, "{}: {}", self.cause, message)
        } else {
            write!(f, "{}", self.cause)
        }
    }
}

impl std::fmt::Display for VersionErrorCause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use VersionErrorCause as V;

        write!(
            f,
            "{}",
            match self {
                V::MissingMajor => "Major version couldn't be split",
                V::FailedMajorParse => "Failed parse of major version",
                V::MissingMinor => "Minor version couldn't be split",
                V::FailedMinorParse => "Failed parse of minor version",
                V::MissingPatch => "Patch version couldn't be split",
                V::FailedPatchParse => "Failed parse of patch version",
                V::FailedValidation => "Validation failed",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn empty_footer_text() {
        let mut graph = Graph::load();

        graph.meta.config = Config {
            footer_text: String::default(),
            ..graph.meta.config
        };

        graph.parse_config();

        println!("{:?}", graph.meta.config.footer_text);
        assert!(graph.meta.config.footer_text.is_empty());
    }

    #[test]
    fn config_footer_text() {
        let payload = "0kqBrdS8NPrU4xVxh2xW0hUzAw926JCQ";
        let mut graph = Graph::load();

        graph.meta.config = Config {
            footer_text: format!("`{payload}`"),
            ..graph.meta.config
        };

        graph.parse_config();

        assert!(
            graph
                .meta
                .config
                .footer_text
                .matches(format!("<code>{payload}</code>").as_str())
                .count()
                == 1
        );
    }

    #[test]
    fn config_about_text() {
        let payload = "ZqPFl84JlzSS0QUo61RwTUPONIE78Lmw";
        let mut graph = Graph::load();

        graph.meta.config = Config {
            about_text: format!("`{payload}`"),
            ..graph.meta.config
        };

        graph.parse_config();

        assert!(
            graph
                .meta
                .config
                .about_text
                .matches(format!("<code>{payload}</code>").as_str())
                .count()
                == 1
        );
    }

    #[test]
    fn modulated_graph_version_matches_compilation_version() {
        let version = Graph::load().meta.version;

        assert_eq!(format!("{version}"), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn from_compilation_matches_compilation_version() {
        let version = Version::from_compilation().unwrap();

        assert_eq!(format!("{version}"), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn version_from_str() {
        let payload = "3.9.74";
        let version_result = Version::from_text(payload);

        println!("{version_result:#?}");
        assert_eq!(format!("{}", version_result.unwrap()), payload);
    }

    #[test]
    fn missing_major() {
        let error = Version::from_text("").unwrap_err();
        println!("{error:#?}");
        assert!(matches!(error.cause, VersionErrorCause::MissingMajor));
    }

    #[test]
    fn missing_minor() {
        let error = Version::from_text("3").unwrap_err();
        println!("{error:#?}");
        assert!(matches!(error.cause, VersionErrorCause::MissingMinor));
    }

    #[test]
    fn missing_patch() {
        let error = Version::from_text("3.6").unwrap_err();
        assert!(matches!(error.cause, VersionErrorCause::MissingPatch));
    }

    #[test]
    fn malformed_patch() {
        let error = Version::from_text("3.6.x").unwrap_err();
        assert!(matches!(error.cause, VersionErrorCause::FailedPatchParse));

        let error_empty = Version::from_text("3.6.").unwrap_err();
        assert!(matches!(
            error_empty.cause,
            VersionErrorCause::FailedPatchParse
        ));
    }

    #[test]
    fn malformed_minor() {
        let error = Version::from_text("3.x.0").unwrap_err();
        assert!(matches!(error.cause, VersionErrorCause::FailedMinorParse));

        let error_bad_patch = Version::from_text("3.x.z").unwrap_err();
        assert!(matches!(
            error_bad_patch.cause,
            VersionErrorCause::FailedMinorParse
        ));

        let error_empty_patch = Version::from_text("3.x.").unwrap_err();
        assert!(matches!(
            error_empty_patch.cause,
            VersionErrorCause::FailedMinorParse
        ));

        let error_patchless = Version::from_text("3.x").unwrap_err();
        assert!(matches!(
            error_patchless.cause,
            VersionErrorCause::FailedMinorParse
        ));

        let error_empty = Version::from_text("3.").unwrap_err();
        assert!(matches!(
            error_empty.cause,
            VersionErrorCause::FailedMinorParse
        ));
    }

    #[test]
    fn malformed_major() {
        let error = Version::from_text("x.6.0").unwrap_err();
        assert!(matches!(error.cause, VersionErrorCause::FailedMajorParse));

        let error_bad_patch = Version::from_text("x.y.z").unwrap_err();
        assert!(matches!(
            error_bad_patch.cause,
            VersionErrorCause::FailedMajorParse
        ));

        let error_empty_patch = Version::from_text("x.6.").unwrap_err();
        assert!(matches!(
            error_empty_patch.cause,
            VersionErrorCause::FailedMajorParse
        ));

        let error_patchless = Version::from_text("x.6").unwrap_err();
        assert!(matches!(
            error_patchless.cause,
            VersionErrorCause::FailedMajorParse
        ));

        let error_bad_minor = Version::from_text("x.y").unwrap_err();
        assert!(matches!(
            error_bad_minor.cause,
            VersionErrorCause::FailedMajorParse
        ));

        let error_empty_minor = Version::from_text("x.").unwrap_err();
        assert!(matches!(
            error_empty_minor.cause,
            VersionErrorCause::FailedMajorParse
        ));

        let error_minorless = Version::from_text("x").unwrap_err();
        assert!(matches!(
            error_minorless.cause,
            VersionErrorCause::FailedMajorParse
        ));

        let error_empty = Version::from_text("").unwrap_err();
        assert!(matches!(error_empty.cause, VersionErrorCause::MissingMajor));
    }

    #[test]
    fn version_validation() {
        assert!(["3.1.4.", "3.1.4.1"].iter().all(|s| matches!(
            Version::from_text(s).unwrap_err().cause,
            VersionErrorCause::FailedValidation
        )));
    }

    #[test]
    fn leading_v() {
        let version = Version::from_text("v3.1.4").unwrap();
        assert_eq!(format!("{version}"), "3.1.4");
    }

    #[test]
    fn display_version_error_cause() {
        fn assert(version: &str, message: &str) {
            let error = Version::from_text(version).unwrap_err();
            assert_eq!(format!("{error}"), message);
        }

        assert("3.6", "Patch version couldn't be split");
        assert(
            "3.6.",
            "Failed parse of patch version: \
                cannot parse integer from empty string",
        );
        assert(
            "3.6.x",
            "Failed parse of patch version: \
                invalid digit found in string",
        );

        assert("3", "Minor version couldn't be split");
        assert(
            "3.",
            "Failed parse of minor version: \
                cannot parse integer from empty string",
        );
        assert(
            "3.x",
            "Failed parse of minor version: \
                invalid digit found in string",
        );

        assert("", "Major version couldn't be split");
        assert(
            "x",
            "Failed parse of major version: \
                invalid digit found in string",
        );

        let validation_error = Version::from_text("3.1.4.1..").unwrap_err();
        println!("{validation_error}");

        assert!(matches!(
            validation_error.cause,
            VersionErrorCause::FailedValidation
        ));
        assert!(
            validation_error
                .message
                .clone()
                .unwrap()
                .contains("Has exactly two dots: false")
        );
        assert!(
            validation_error
                .message
                .clone()
                .unwrap()
                .contains("Splits to three elements: false")
        );
    }
}
