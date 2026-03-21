use std::{env, fs, io, path::PathBuf};

use crate::prelude::*;

#[derive(Debug)]
pub struct Directories {
    pub original: PathBuf,
    pub templates: PathBuf,
    pub assets: PathBuf,
    pub test: PathBuf,
}

impl Directories {
    /// Sets up self-cleaning original, temporary and 'templates' directories.
    ///
    /// # Errors
    /// May return Error when:
    /// - Current directory does not exist or lacking permissions
    /// - Several I/O possibilities from directory creation failures
    /// - Several I/O possibilities from working directory changing failures
    pub fn setup(dir_name: &str) -> Result<Directories, Error> {
        let original = env::current_dir()?;
        let test = original.join(format!("target/mocks/{dir_name}"));
        let templates = test.join("templates");
        let assets = test.join("static").join("public").join("assets");

        drop(fs::remove_dir_all(&test));

        if let Err(error) = fs::create_dir_all(&test) {
            return Err(Error::with_io(
                "Failed test's directory creation",
                error,
            ))
        }

        if let Err(error) = fs::create_dir_all(&templates) {
            return Err(Error::with_io(
                "Failed 'templates' directory creation",
                error,
            ))
        }

        if let Err(error) = fs::create_dir_all(&assets) {
            return Err(Error::with_io(
                "Failed 'assets' directory creation",
                error,
            ))
        }

        if let Err(error) = env::set_current_dir(&test) {
            return Err(Error::with_io("Failed current directory change", error))
        }

        Ok(Directories {
            original,
            templates,
            assets,
            test,
        })
    }
}

impl Drop for Directories {
    fn drop(&mut self) {
        if let Err(error) = std::env::set_current_dir(&self.original) {
            log!(ERROR, "Couldn't reset to original directory: {error}");
        }
        if let Err(error) = std::fs::remove_dir_all(&self.test) {
            log!(WARN, "Couldn't cleanup test directory: {error}");
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub inner_io: Option<io::Error>,
    pub inner_tera: Option<tera::Error>,
}

impl Error {
    fn with_io(message: &str, inner_error: io::Error) -> Error {
        Error {
            message: String::from(message),
            inner_io: Some(inner_error),
            inner_tera: None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut message = self.message.clone();

        if let Some(inner_io) = &self.inner_io {
            message = format!("{message}\n{inner_io}");
        }

        if let Some(inner_tera) = &self.inner_tera {
            message = format!("{message}\n{inner_tera}");
        }

        write!(f, "{message}")
    }
}

impl From<String> for Error {
    fn from(string: String) -> Error {
        Error {
            message: string,
            inner_io: None,
            inner_tera: None,
        }
    }
}

impl From<&str> for Error {
    fn from(str: &str) -> Error { Error::from(String::from(str)) }
}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Error {
        let mut error = Error::from(inner.to_string());
        error.inner_io = Some(inner);
        error
    }
}

impl From<tera::Error> for Error {
    fn from(inner: tera::Error) -> Error {
        let mut error = Error::from(inner.to_string());
        error.inner_tera = Some(inner);
        error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_test_directory_name() {
        let dirs = Directories::setup("\0");
        assert!(dirs.is_err());
    }

    #[test]
    fn display_contains_str_from_from() {
        let payload = "rHneusPkYNGW0Ia0";
        let error = Error::from(payload);
        assert!(format!("{error}").contains(payload));
    }

    #[test]
    fn display_contains_str_from_io_error() {
        let payload = "SsVi0d3Ywc8kVhwp";
        let io_payload = "LoPbZP7cJEHzAjGW";
        let io_error = std::io::Error::other(io_payload);
        let error = Error::with_io(payload, io_error);
        assert!(format!("{error}").contains(payload));
        assert!(format!("{error}").contains(io_payload));
    }

    #[test]
    fn display_contains_str_from_tera_error() {
        let payload = "pA6B0LhiiDMNCl1J";
        let tera_payload = "5ob8H594dCAQ8pfk";
        let error = Error {
            message: payload.to_string(),
            inner_tera: Some(tera::Error::msg(tera_payload)),
            inner_io: None,
        };
        assert!(format!("{error}").contains(payload));
        assert!(format!("{error}").contains(tera_payload));
    }
    #[test]
    fn from_io_error() {
        let payload = "YgmTKBm3VtHt5h3x9";
        let io_error = std::io::Error::other(payload);
        let error = Error::from(io_error);

        assert!(error.message.contains(payload));
    }

    #[test]
    fn from_tera_error() {
        let payload = "XEB3dcvYuz0M1lYt";
        let tera_error = tera::Error::msg(payload);
        let error = Error::from(tera_error);

        assert!(error.message.contains(payload));
    }
}

#[cfg(test)]
mod serial_tests {
    use super::*;

    #[test]
    fn failed_working_directory_reset() {
        let dirs = Directories::setup("\0");

        let error = dirs.unwrap_err();
        println!("{error}");
        assert!(error.message.contains("Failed test's directory creation"));
        assert!(
            format!("{error}")
                .contains("file name contained an unexpected NUL byte")
        );
    }
}
