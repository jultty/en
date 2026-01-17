#[derive(Debug, Clone)]
pub enum Mime {
    Txt,
    Csv,
    Css,
    Ttf,
    Otf,
    Woff,
    Woff2,
    Svg,
    Ico,
    Jpeg,
    Png,
    Apng,
    Gif,
    Webp,
    Avif,
    Toml,
    Xml,
    Json,
    Js,
    Pdf,
    Epub,
    Unknown,
}

impl From<&str> for Mime {
    fn from(extension: &str) -> Mime {
        match extension {
            "txt" => Mime::Txt,
            "csv" => Mime::Csv,
            "css" => Mime::Css,
            "ttf" => Mime::Ttf,
            "otf" => Mime::Otf,
            "woff" => Mime::Woff,
            "woff2" => Mime::Woff2,
            "svg" => Mime::Svg,
            "ico" => Mime::Ico,
            "jpeg" => Mime::Jpeg,
            "png" => Mime::Png,
            "apng" => Mime::Apng,
            "gif" => Mime::Gif,
            "webp" => Mime::Webp,
            "avif" => Mime::Avif,
            "toml" => Mime::Toml,
            "xml" => Mime::Xml,
            "json" => Mime::Json,
            "js" => Mime::Js,
            "pdf" => Mime::Pdf,
            "epub" => Mime::Epub,
            _ => Mime::Unknown,
        }
    }
}

impl From<Mime> for String {
    fn from(mime: Mime) -> String {
        let s = match mime {
            Mime::Txt => "text/plain",
            Mime::Csv => "text/csv",
            Mime::Css => "text/css",
            Mime::Ttf => "font/ttf",
            Mime::Otf => "font/otf",
            Mime::Woff => "font/woff",
            Mime::Woff2 => "font/woff2",
            Mime::Svg => "image/svg+xml",
            Mime::Ico => "image/x-icon",
            Mime::Jpeg => "image/jpeg",
            Mime::Png => "image/png",
            Mime::Apng => "image/apng",
            Mime::Gif => "image/gif",
            Mime::Webp => "image/webp",
            Mime::Avif => "image/avif",
            Mime::Toml => "application/toml",
            Mime::Xml => "application/xml",
            Mime::Json => "application/json",
            Mime::Js => "text/javascript",
            Mime::Pdf => "application/pdf",
            Mime::Epub => "application/epub+zip",
            Mime::Unknown => "application/octet-stream",
        };
        String::from(s)
    }
}

impl Mime {
    pub fn guess(path: &str) -> Mime {
        if let Some(pair) = path.rsplit_once('.') {
            Mime::from(pair.1)
        } else {
            Mime::Unknown
        }
    }
}
