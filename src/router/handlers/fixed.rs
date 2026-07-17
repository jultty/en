use std::{collections::HashMap, io::ErrorKind};

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, Response, header},
};
use serde::Serialize;

use crate::{
    dev::log,
    graph::{Format, Graph, SerialErrorCause},
    prelude::*,
    router::{
        GlobalState,
        handlers::{
            self,
            asset::{Asset, AssetError, AssetErrorKind},
            error, mime,
        },
    },
    syntax::command::Arguments,
};

/// Assembles an HTTP response given Asset.
fn assemble(asset: Asset, graph: &Graph) -> Response<Body> {
    let set_content_type = |response: &mut Response<_>, content_type: &str| {
        if let Ok(header_value) =
            HeaderValue::from_str(&String::from(content_type))
        {
            response
                .headers_mut()
                .append(header::CONTENT_TYPE, header_value);
        } else {
            // This should be unreachable considering the possible mimetypes
            // and their string representations are internal to en
            log!(
                WARN,
                "Failed to create content type header value from {content_type}"
            );
        }
    };

    match asset.mime.kind() {
        mime::Kind::Text => {
            if let Some(text) = asset.text {
                let mut response = Response::new(Body::from(text));
                set_content_type(
                    &mut response,
                    &String::from(asset.mime.clone()),
                );
                response
            } else {
                // This should be unreachable, considering the constructors
                // will convert to text even if a blob is passed
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
                // This should be unreachable, considering the constructors
                // will convert to blob even if a text is passed
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

/// Given a relative path, returns the file contents or a default fallback.
///
/// The `path` argument is relative to the `static/public` directory,
/// or a directory passed with the `--public` CLI option.
///
/// Defaults are found in the `fixed::DEFAULTS` map.
///
/// Returns a `FallbackError` if neither is found or an I/O error occurred.
fn fallback(path: &str, graph: &Graph) -> Result<Asset, AssetError> {
    let cli_args = Arguments::default().parse();
    let target = cli_args.public.join(path);
    let defaults: HashMap<&str, &str> = TEXTS.iter().copied().collect();
    let fonts: HashMap<&str, &'static Font> = FONTS.iter().copied().collect();
    let mime = mime::Mime::guess(path);

    log!("Seeking {target:?}");

    match std::fs::read(&target) {
        // A matching file exists on disk and is accessible
        Ok(content) => Ok(Asset::new(&content, mime)?),
        Err(io_error) => {
            // A matching file does not exist on disk
            if io_error.kind() == ErrorKind::NotFound {
                if let Some(content) = defaults.get(path) {
                    Ok(Asset::from_str(content, mime))
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
            // A matching file exists on disk and is not accessible
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
                    permission to access it or another I/O error occurred.",
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
        "../../../static/public/assets/fonts/_canon/SIL_OFL_1_1.body.LICENSE"
    ),
};

static CCND: License = License {
    name: "Creative Commons Attribution-NoDerivatives 4.0 International",
    kind: &LicenseKind::CC_BY_ND_4_0_INTERNATIONAL,
    url: "/assets/licenses/CC_BY_ND_4_0_INTERNATIONAL.txt",
    text: include_str!(
        "../../../static/public/assets/fonts/_canon/\
        CC_BY_ND_4_0_INTERNATIONAL.body.LICENSE"
    ),
};

#[cfg(test)]
mod tests {
    use axum::http::status::StatusCode;

    use super::*;
    use crate::router::handlers::mime::Mime;

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
        assert!(response.status() == StatusCode::NOT_FOUND);
    }

    #[test]
    fn error_from_utf8error() {
        let bytes = vec![0, 159];
        let utf8error = String::from_utf8(bytes.clone()).unwrap_err();
        let error = AssetError::from(utf8error);
        assert!(error.utf8_error.is_some());
        assert_eq!(error.utf8_error.unwrap().into_bytes(), bytes);
    }

    #[test]
    fn error_from_string() {
        let payload = "r5MDnkEojW9HZDAG";
        let asset_error = AssetError::from(payload.to_string());
        println!("{asset_error}");
        assert!(asset_error.path.contains(payload));
    }

    #[test]
    fn new_text_asset() {
        let asset = Asset::new(&[1, 0, 1], mime::Mime::Txt).unwrap();

        assert!(asset.blob.is_none());
        assert!(asset.text.is_some());
        assert_eq!(asset.text.unwrap(), "\u{1}\0\u{1}");
    }

    #[test]
    fn new_blob_asset() {
        let asset = Asset::new(&[1, 0, 1], mime::Mime::Png).unwrap();

        assert!(asset.blob.is_some());
        assert!(asset.text.is_none());
        assert_eq!(asset.blob.unwrap(), &[1, 0, 1]);
    }

    #[test]
    fn asset_from_str() {
        let payload = "\u{1}\0\u{6}";
        let asset = Asset::from_str(payload, mime::Mime::Ico);
        assert_eq!(asset.blob.unwrap(), &[1, 0, 6]);
    }

    #[test]
    fn new_asset_utf8_error() {
        let bad_bytes = [0xff, 0xc0, 0xf5, 0xc1, 0x80];

        let error = Asset::new(&bad_bytes, mime::Mime::Txt).unwrap_err();

        assert!(matches!(&error.kind, AssetErrorKind::UTF8));
        assert!(format!("{error}").contains("UTF8 decoding error"));
    }

    #[test]
    fn not_found_asset_error() {
        let error = fallback("not_found.png", &Graph::default()).unwrap_err();

        assert!(matches!(&error.kind, AssetErrorKind::NotFound));
        println!("{error:?}");
        assert!(format!("{error}").contains("No file was found for path"));
    }

    #[test]
    fn assemble_from_blob() {
        let asset = Asset::new(&[1, 0, 1], Mime::Pdf).unwrap();
        let response = assemble(asset, &Graph::default());
        let content_type =
            response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "application/pdf");
    }
}

#[cfg(test)]
#[cfg(unix)]
#[expect(clippy::panic_in_result_fn, clippy::unwrap_in_result)]
mod serial_tests {
    use std::{fs, os::unix::fs::PermissionsExt as _, path::PathBuf};

    use axum::http::StatusCode;

    use super::*;
    use crate::{
        dev::test::{Directories, Error, request},
        router::handlers::mime::Mime,
    };

    #[test]
    fn io_asset_error() -> Result<(), Error> {
        let dirs = Directories::setup("io_asset_error")?;

        let file = dirs.assets.join("unreadable.png");

        fs::write(&file, [1, 0, 1])?;
        let mut permissions = fs::metadata(&file)?.permissions();
        permissions.set_mode(0o200);
        fs::set_permissions(&file, permissions)?;

        let new_permissions = fs::metadata(&file)?.permissions();
        assert_eq!(new_permissions.mode() & 0o777, 0o200);

        let error =
            fallback("assets/unreadable.png", &Graph::default()).unwrap_err();

        assert!(matches!(&error.kind, AssetErrorKind::IO));
        assert!(
            format!("{error}")
                .contains("was found, but it could not be loaded")
        );

        Ok(())
    }

    #[test]
    fn target_file_exists() -> Result<(), Error> {
        let dirs = Directories::setup("target_file_exists")?;

        let file = dirs.public.join("asset.woff2");

        fs::write(&file, [1, 0, 1])?;
        let asset = fallback("asset.woff2", &Graph::default()).unwrap();
        assert!(asset.text.is_none());
        assert!(asset.blob.is_some());
        assert!(matches!(asset.mime, Mime::Woff2));

        Ok(())
    }

    #[test]
    fn default_font_found_if_serving_enabled() -> Result<(), Error> {
        let dirs = Directories::setup("font_found_if_serving_enabled")?;

        let relative_font_path = PathBuf::from(FONTS[0].0);
        let font_path = dirs.assets.join(&relative_font_path);
        let font_dir = font_path.parent().expect("failed getting font dir");

        println!("{font_dir:?}");
        fs::create_dir_all(font_dir)?;
        fs::write(&font_path, [1, 0, 1])?;
        let graph = Graph::from_serial(
            "[meta.config]\nserve_fonts = true",
            &Format::TOML,
        )
        .expect("failed instantiating graph");
        println!("{font_path:?}");
        let asset = fallback(relative_font_path.to_str().unwrap(), &graph)
            .expect("fallback failed");

        assert!(asset.text.is_none());
        assert!(asset.blob.is_some());
        assert!(matches!(asset.mime, Mime::Woff2));

        Ok(())
    }

    #[test]
    fn custom_font_found_if_serving_enabled() -> Result<(), Error> {
        let dirs = Directories::setup("font_found_if_serving_enabled")?;

        let font_dir = dirs.assets.join("fonts");
        let font_path = font_dir.join("custom.ttf");

        eprintln!("Creating directory {font_dir:?}");
        fs::create_dir_all(font_dir)?;
        eprintln!("Writing to {font_path:?}");
        fs::write(&font_path, [1, 0, 1])?;
        let graph = Graph::from_serial(
            "[meta.config]\nserve_fonts = true",
            &Format::TOML,
        )
        .expect("failed instantiating graph");
        let asset = fallback("assets/fonts/custom.ttf", &graph)
            .expect("fallback failed");

        assert!(asset.text.is_none());
        assert!(asset.blob.is_some());
        assert!(matches!(asset.mime, Mime::Ttf));

        Ok(())
    }

    #[test]
    fn font_not_found_if_serving_disabled() -> Result<(), Error> {
        let dirs = Directories::setup("target_file_exists")?;

        let relative_font_path =
            PathBuf::from(FONTS[0].0.replace("assets/", ""));
        let font_path = dirs.assets.join(&relative_font_path);
        let font_dir = font_path.parent().unwrap();

        let graph = Graph::from_serial(
            "[meta.config]\nserve_fonts = false",
            &Format::TOML,
        )
        .unwrap();
        let error = fallback(font_path.to_str().unwrap(), &graph).unwrap_err();
        assert!(matches!(error.kind, AssetErrorKind::NotFound));

        Ok(())
    }

    #[tokio::test]
    async fn custom_file_is_served() -> Result<(), Error> {
        let dirs = Directories::setup("custom_file_is_served")?;

        let file1 = dirs.public.join("file1.txt");
        let subdir = dirs.public.join("subdir");
        let file2 = subdir.join("file2.txt");

        fs::create_dir_all(subdir)?;
        fs::write(file1, "eff90_1")?;
        fs::write(file2, "eff90_2")?;

        let asset1 = fallback("file1.txt", &Graph::default()).unwrap();
        let asset2 = fallback("subdir/file2.txt", &Graph::default()).unwrap();

        assert!(matches!(asset1.mime, Mime::Txt));
        assert!(matches!(asset2.mime, Mime::Txt));

        let response1 = request("/static/file1.txt", None).await?;
        let response2 = request("/static/subdir/file2.txt", None).await?;

        assert_eq!(response1.status(), StatusCode::OK);
        assert_eq!(response2.status(), StatusCode::OK);

        Ok(())
    }
}
