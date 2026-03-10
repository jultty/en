use std::{collections::HashMap, io::ErrorKind, string::FromUtf8Error};

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, Response, header},
};
use serde::Serialize;

use crate::{
    graph::{Format, Graph, SerialErrorCause},
    prelude::*,
    router::{
        GlobalState,
        handlers::{self, error, mime},
    },
};

/// Assembles an HTTP response given Asset.
fn assemble(asset: Asset, graph: &Graph) -> Response<Body> {
    let kind = match asset.mime.kind() {
        Ok(kind) => kind,
        Err(error) => {
            return error::make(
                Some(500),
                Some(&format!("Could not determine a mimetype kind: {error}")),
                graph,
            )
        },
    };

    let set_content_type = |response: &mut Response<_>, content_type: &str| {
        if let Ok(header_value) =
            HeaderValue::from_str(&String::from(content_type))
        {
            response
                .headers_mut()
                .append(header::CONTENT_TYPE, header_value);
        } else {
            log!(
                WARN,
                "Failed to create content type header value from {content_type}"
            );
        }
    };

    match kind {
        mime::Kind::Text => {
            if let Some(text) = asset.text {
                let mut response = Response::new(Body::from(text));
                set_content_type(
                    &mut response,
                    &String::from(asset.mime.clone()),
                );
                response
            } else {
                let mut response = error::make(
                    Some(500),
                    Some(
                        "Asset mimetype indicates text content, \
                            but none was found",
                    ),
                    graph,
                );
                set_content_type(&mut response, "text/html");
                response
            }
        },
        mime::Kind::Font | mime::Kind::Blob | mime::Kind::Image => {
            if let Some(blob) = asset.blob {
                let mut response = Response::new(Body::from(blob));
                set_content_type(
                    &mut response,
                    &String::from(asset.mime.clone()),
                );
                response
            } else {
                let mut response = error::make(
                    Some(500),
                    Some(
                        "Asset mimetype indicates binary content, \
                            but none was found",
                    ),
                    graph,
                );
                set_content_type(&mut response, "text/html");
                response
            }
        },
    }
}

#[expect(clippy::upper_case_acronyms)]
#[derive(Debug)]
enum AssetErrorKind {
    NotFound,
    IO,
    UTF8,
    Unknown,
}

#[derive(Debug)]
struct AssetError {
    path: String,
    kind: AssetErrorKind,
    io_error: Option<std::io::Error>,
    utf8_error: Option<FromUtf8Error>,
}

impl AssetError {
    fn new(
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
                    "A default fallback for {} was found, \
                    but it could not be loaded",
                    self.path
                )
            },
            AssetErrorKind::NotFound => {
                String::from("The file was not found in the searched path")
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

struct Asset {
    blob: Option<Vec<u8>>,
    text: Option<String>,
    mime: mime::Mime,
}

impl Asset {
    fn new(blob: &[u8], mime: mime::Mime) -> Result<Asset, AssetError> {
        match mime.kind()? {
            mime::Kind::Text => Ok(Asset {
                text: Some(String::from_utf8(blob.to_vec())?),
                blob: None,
                mime,
            }),
            mime::Kind::Font | mime::Kind::Image | mime::Kind::Blob => {
                Ok(Asset {
                    text: None,
                    blob: Some(blob.to_vec()),
                    mime,
                })
            },
        }
    }

    fn from_str(str: &str, mime: mime::Mime) -> Result<Asset, AssetError> {
        match mime.kind()? {
            mime::Kind::Text => Ok(Asset {
                text: Some(String::from(str)),
                blob: None,
                mime,
            }),
            mime::Kind::Font | mime::Kind::Image | mime::Kind::Blob => {
                Ok(Asset {
                    text: None,
                    blob: Some(String::from(str).into_bytes()),
                    mime,
                })
            },
        }
    }
}

/// Given a relative path, returns the file contents or a default fallback.
///
/// The `path` argument is relative to the `static/public` directory.
///
/// Defaults are found in the `fixed::DEFAULTS` map.
///
/// Returns a `FallbackError` if neither is found or an I/O error ocurred.
fn fallback(path: &str, graph: &Graph) -> Result<Asset, AssetError> {
    let target = format!("static/public/assets/{path}");
    let defaults: HashMap<&str, &str> = TEXTS.iter().copied().collect();
    let fonts: HashMap<&str, &'static Font> = FONTS.iter().copied().collect();
    let mime = mime::Mime::guess(path);

    match std::fs::read(&target) {
        // A matching file exists on disk
        Ok(content) => Ok(Asset {
            blob: Some(content),
            text: None,
            mime,
        }),
        Err(io_error) => {
            if io_error.kind() == ErrorKind::NotFound {
                if let Some(content) = defaults.get(path) {
                    Asset::from_str(content, mime)
                } else {
                    let not_found_error = Err(AssetError::new(
                        path,
                        AssetErrorKind::NotFound,
                        Some(io_error),
                        None,
                    ));

                    if !graph.meta.config.serve_fonts {
                        return not_found_error
                    }

                    match fonts.get(path) {
                        // A matching font exists
                        Some(content) => Asset::new(content.blob, mime),
                        None => not_found_error,
                    }
                }
            } else {
                Err(AssetError::new(
                    path,
                    AssetErrorKind::IO,
                    Some(io_error),
                    None,
                ))
            }
        },
    }
}

/// Handles requests for static files.
///
/// This handler receives and extracts requests from `/static/{path}`.
pub async fn file(
    Path(path): Path<String>,
    State(state): State<GlobalState>,
) -> Response<Body> {
    let instant = now();

    match fallback(&path, &state.graph) {
        Ok(asset) => {
            let response = assemble(asset, &state.graph);
            tlog!(
                &instant,
                "Assembled {} response for {path}",
                response.status()
            );
            response
        },
        Err(asset_error) => {
            let mut error_message =
                if matches!(asset_error.kind, AssetErrorKind::NotFound) {
                    String::from("The requested file was not found.")
                } else {
                    String::from(
                        "The requested file exists, but the server lacks \
                    permission to access it or another I/O error ocurred.",
                    )
                };
            if log::env_level() >= DEBUG {
                error_message = format!(
                    "<p>{error_message}</p>\
                    <p>Targeted path: <code>{path}</code></p>\
                    <p>Error:</p> <pre>{asset_error}</pre>"
                );
            }
            log!(ERROR, "{error_message}");
            error::make(Some(404), Some(&error_message), &state.graph)
        },
    }
}

pub async fn serial(
    Path(format): Path<String>,
    State(state): State<GlobalState>,
) -> Response<Body> {
    let config = &state.graph.meta.config;

    let make_error = |code: u16, message: &str| -> Response<Body> {
        handlers::error::make(
            Some(code),
            Some(
                format!(
                    "<p>{message}</p>\n\
            <p>Check the <a href=/data>data</a> \n\
            page for the available formats.</p>"
                )
                .as_str(),
            ),
            &state.graph,
        )
    };

    let forbidden_response =
        make_error(403, "This graph format is not available.");
    let unsupported_response =
        make_error(400, "This graph format is not supported.");
    let parse_failure = make_error(505, "The graph has failed to parse.");

    let body =
        match Graph::to_serial(&state.graph, &Format::from(format.as_str())) {
            Ok(serial) => serial,
            Err(error) => match error.cause {
                SerialErrorCause::MalformedInput => return parse_failure,
                SerialErrorCause::UnsupportedFormat => {
                    return unsupported_response;
                },
            },
        };

    match Format::from(format.as_str()) {
        Format::TOML => {
            if config.raw && config.raw_toml {
                handlers::raw::make_response(
                    &body,
                    200,
                    &[(header::CONTENT_TYPE, "text/plain")],
                )
            } else {
                forbidden_response
            }
        },
        Format::JSON => {
            if config.raw && config.raw_json {
                handlers::raw::make_response(
                    &body,
                    200,
                    &[(header::CONTENT_TYPE, "application/json")],
                )
            } else {
                forbidden_response
            }
        },
        Format::Unsupported => unsupported_response,
    }
}

static TEXTS: &[(&str, &str)] = &[
    (
        "assets/style.css",
        include_str!("../../../static/public/assets/style.css"),
    ),
    (
        "assets/fonts/fonts.css",
        include_str!("../../../static/public/assets/fonts/fonts.css"),
    ),
    (
        "assets/favicon.svg",
        include_str!("../../../static/public/assets/favicon.svg"),
    ),
    ("assets/licenses/SIL_OFL_1_1.txt", OFL.text),
    ("assets/licenses/CC_BY_ND_4_0_INTERNATIONAL.txt", CCND.text),
];

pub static FONTS: &[(&str, &Font)] = &[
    (
        "assets/fonts/cormorant/cormorant-infant-latin-300-normal.woff2",
        &Font {
            name: "Cormorant Infant",
            attribution: &Attribution {
                project_name: "Cormorant",
                author: "Christian Thalmann",
                project_url: "https://github.com/CatharsisFonts/Cormorant",
                author_url: "https://github.com/CatharsisFonts",
                license_header: include_str!(
                    "../../../static/public/assets/fonts/\
                cormorant/header.LICENSE"
                ),
            },
            blob: include_bytes!(
                "../../../static/public/assets/fonts/\
                cormorant/cormorant-infant-latin-300-normal.woff2"
            ),
            license: &OFL,
        },
    ),
    (
        "assets/fonts/cormorant/cormorant-infant-latin-300-italic.woff2",
        &Font {
            name: "Cormorant Infant Italic",
            attribution: &Attribution {
                project_name: "Cormorant",
                author: "Christian Thalmann",
                project_url: "https://github.com/CatharsisFonts/Cormorant",
                author_url: "https://github.com/CatharsisFonts",
                license_header: include_str!(
                    "../../../static/public/assets/fonts/\
                    cormorant/header.LICENSE"
                ),
            },
            blob: include_bytes!(
                "../../../static/public/assets/fonts/\
                cormorant/cormorant-infant-latin-300-italic.woff2"
            ),
            license: &OFL,
        },
    ),
    (
        "assets/fonts/maven/maven-pro-latin-400-normal.woff2",
        &Font {
            name: "Maven Pro",
            attribution: &Attribution {
                project_name: "Maven",
                author: "Joe Prince and Project Authors",
                author_url: "https://github.com/m4rc1e/mavenproFont/blob/\
                    main/AUTHORS.txt",
                project_url: "https://github.com/m4rc1e/mavenproFont",
                license_header: include_str!(
                    "../../../static/public/assets/fonts/\
                maven/header.LICENSE"
                ),
            },
            blob: include_bytes!(
                "../../../static/public/assets/fonts/\
                maven/maven-pro-latin-400-normal.woff2"
            ),
            license: &OFL,
        },
    ),
    (
        "assets/fonts/mononoki/mononoki-latin-400-normal.woff2",
        &Font {
            name: "Mononoki",
            attribution: &Attribution {
                project_name: "Mononoki",
                author: "Matthias Tellen",
                author_url: "https://github.com/madmalik",
                project_url: "https://madmalik.github.io/mononoki/",
                license_header: include_str!(
                    "../../../static/public/assets/fonts/\
                    mononoki/header.LICENSE"
                ),
            },
            blob: include_bytes!(
                "../../../static/public/assets/fonts/\
                mononoki/mononoki-latin-400-normal.woff2"
            ),
            license: &OFL,
        },
    ),
    (
        "assets/fonts/rawengulk/RawengulkLight.woff2",
        &Font {
            name: "Rawengulk Light",
            attribution: &Attribution {
                project_name: "Rawengulk",
                author: "gluk Fonts",
                author_url: "https://www.glukfonts.pl",
                project_url: "https://www.glukfonts.pl/font.php?font=Rawengulk",
                license_header: include_str!(
                    "../../../static/public/assets/fonts/\
                    rawengulk/header.LICENSE"
                ),
            },
            license: &OFL,
            blob: include_bytes!(
                "../../../static/public/assets/fonts/\
                rawengulk/RawengulkLight.woff2"
            ),
        },
    ),
    (
        "assets/fonts/reforma/Reforma1969-Blanca.woff2",
        &Font {
            name: "Reforma 1969 Blanca",
            attribution: &REFORMA_ATTRIBUTION,
            license: &CCND,
            blob: include_bytes!(
                "../../../static/public/assets/fonts/\
                reforma/Reforma1969-Blanca.woff2"
            ),
        },
    ),
    (
        "assets/fonts/reforma/Reforma1969-BlancaItalica.woff2",
        &Font {
            name: "Reforma 1969 Blanca Italica",
            attribution: &REFORMA_ATTRIBUTION,
            license: &CCND,
            blob: include_bytes!(
                "../../../static/public/assets/fonts/\
                reforma/Reforma1969-BlancaItalica.woff2"
            ),
        },
    ),
    (
        "assets/fonts/reforma/Reforma1969-Gris.woff2",
        &Font {
            name: "Reforma 1969 Blanca Gris",
            attribution: &REFORMA_ATTRIBUTION,
            license: &CCND,
            blob: include_bytes!(
                "../../../static/public/assets/fonts/\
                reforma/Reforma1969-Gris.woff2"
            ),
        },
    ),
    (
        "assets/fonts/reforma/Reforma1969-GrisItalica.woff2",
        &Font {
            name: "Reforma 1969 Blanca Gris Italica",
            attribution: &REFORMA_ATTRIBUTION,
            license: &CCND,
            blob: include_bytes!(
                "../../../static/public/assets/fonts/\
                reforma/Reforma1969-GrisItalica.woff2"
            ),
        },
    ),
];

static REFORMA_ATTRIBUTION: Attribution = Attribution {
    project_name: "Reforma",
    project_url: "https://pampatype.com/reforma",
    author: "PampaType",
    author_url: "https://pampatype.com",
    license_header: include_str!(
        "../../../static/public/assets/fonts/\
        reforma/header.LICENSE"
    ),
};

#[derive(Serialize)]
pub struct Font<'f> {
    name: &'f str,
    attribution: &'f Attribution<'f>,
    license: &'f License<'f>,
    blob: &'f [u8],
}

#[derive(Serialize)]
pub struct Attribution<'a> {
    project_name: &'a str,
    project_url: &'a str,
    author: &'a str,
    author_url: &'a str,
    license_header: &'a str,
}

#[derive(Serialize)]
pub struct License<'l> {
    name: &'l str,
    kind: &'l LicenseKind,
    text: &'l str,
    url: &'l str,
}

#[derive(Serialize)]
#[expect(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum LicenseKind {
    SIL_OFL_1_1,
    CC_BY_ND_4_0_INTERNATIONAL,
}

impl std::fmt::Display for LicenseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            LicenseKind::SIL_OFL_1_1 => "SIL Open Font License 1.1",
            LicenseKind::CC_BY_ND_4_0_INTERNATIONAL => {
                "Creative Commons Attribution-NoDerivatives 4.0 International"
            },
        };
        write!(f, "{s}")
    }
}

static OFL: License = License {
    name: "SIL Open Font License 1.1",
    kind: &LicenseKind::SIL_OFL_1_1,
    url: "assets/licenses/SIL_OFL_1_1.txt",
    text: include_str!(
        "../../../static/public/assets/fonts/_canon/SIL_OFL_1_1.LICENSE"
    ),
};

static CCND: License = License {
    name: "Creative Commons Attribution-NoDerivatives 4.0 International",
    kind: &LicenseKind::CC_BY_ND_4_0_INTERNATIONAL,
    url: "/assets/licenses/CC_BY_ND_4_0_INTERNATIONAL.txt",
    text: include_str!(
        "../../../static/public/assets/fonts/_canon/\
        CC_BY_ND_4_0_INTERNATIONAL.LICENSE"
    ),
};

#[cfg(test)]
mod tests {
    use super::*;

    async fn wrap_serial(format: &str) -> Response<Body> {
        let state = GlobalState {
            graph: Graph::load(),
        };
        serial(Path(format.to_string()), State(state)).await
    }

    #[tokio::test]
    async fn serial_toml() {
        let response = wrap_serial("toml").await;
        assert!(response.status() == 200);
    }

    #[tokio::test]
    async fn serial_json() {
        let response = wrap_serial("json").await;
        assert!(response.status() == 200);
    }

    #[tokio::test]
    async fn serial_toml_content_type() {
        let response = wrap_serial("TOML").await;
        assert!(
            response.headers().get(header::CONTENT_TYPE).unwrap()
                == "text/plain"
        );
    }

    #[tokio::test]
    async fn serial_json_content_type() {
        let response = wrap_serial("json").await;
        assert!(
            response.headers().get(header::CONTENT_TYPE).unwrap()
                == "application/json"
        );
    }

    #[tokio::test]
    async fn not_found() {
        let state = GlobalState {
            graph: Graph::default(),
        };
        let response = file(Path("/k/j/m".to_string()), State(state)).await;
        assert!(response.status() == axum::http::status::StatusCode::NOT_FOUND);
    }
}
