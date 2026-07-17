use std::string::FromUtf8Error;

use crate::router::handlers::mime;

#[derive(Debug)]
pub struct Asset {
    pub blob: Option<Vec<u8>>,
    pub text: Option<String>,
    pub mime: mime::Mime,
    _private: (),
}

impl Asset {
    pub fn new(blob: &[u8], mime: mime::Mime) -> Result<Asset, AssetError> {
        match mime.kind() {
            mime::Kind::Text => Ok(Asset {
                text: Some(String::from_utf8(blob.to_vec())?),
                blob: None,
                mime,
                _private: (),
            }),
            mime::Kind::Font | mime::Kind::Image | mime::Kind::Blob => {
                Ok(Asset {
                    text: None,
                    blob: Some(blob.to_vec()),
                    mime,
                    _private: (),
                })
            },
        }
    }

    pub fn from_str(str: &str, mime: mime::Mime) -> Asset {
        match mime.kind() {
            mime::Kind::Text => Asset {
                text: Some(String::from(str)),
                blob: None,
                mime,
                _private: (),
            },
            mime::Kind::Font | mime::Kind::Image | mime::Kind::Blob => Asset {
                text: None,
                blob: Some(String::from(str).into_bytes()),
                mime,
                _private: (),
            },
        }
    }
}

#[derive(Debug)]
pub struct AssetError {
    pub path: String,
    pub kind: AssetErrorKind,
    pub io_error: Option<std::io::Error>,
    pub utf8_error: Option<FromUtf8Error>,
}

#[derive(Debug)]
#[expect(clippy::upper_case_acronyms)]
pub enum AssetErrorKind {
    NotFound,
    IO,
    UTF8,
    Unknown,
}

impl AssetError {
    pub fn new(
        path: &str,
        kind: AssetErrorKind,
        io_error: Option<std::io::Error>,
        utf8_error: Option<FromUtf8Error>,
    ) -> AssetError {
        AssetError {
            path: String::from(path),
            kind,
            io_error,
            utf8_error,
        }
    }
}

impl From<FromUtf8Error> for AssetError {
    fn from(error: FromUtf8Error) -> AssetError {
        AssetError::new("", AssetErrorKind::UTF8, None, Some(error))
    }
}

impl From<String> for AssetError {
    fn from(string: String) -> AssetError {
        AssetError::new(&string, AssetErrorKind::Unknown, None, None)
    }
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut message = match self.kind {
            AssetErrorKind::IO => {
                format!(
                    "File {} was found, but it could not be loaded",
                    self.path
                )
            },
            AssetErrorKind::NotFound => {
                format!("No file was found for path \"{}\"", self.path)
            },
            AssetErrorKind::UTF8 => String::from(
                "UTF8 decoding error: is the file properly encoded?",
            ),
            AssetErrorKind::Unknown => {
                String::from("An unknown error happened.")
            },
        };

        if let Some(error) = &self.io_error {
            message = format!(
                "{message}\n\
                    The following I/O error has happened: \n{error:?}"
            );
        }

        if let Some(error) = &self.utf8_error {
            message = format!(
                "{message}\n\
                    The following encoding error has happened: \n{error:?}"
            );
        }

        write!(f, "{message}")
    }
}
