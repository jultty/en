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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let m = Mime::guess("/home/jane/top/inner/kitty.png");
        assert_eq!(String::from(m), "image/png");
    }

    #[test]
    fn all() {
        let pairs = [
            ("file.txt", "text/plain"),
            ("file.csv", "text/csv"),
            ("file.css", "text/css"),
            ("file.ttf", "font/ttf"),
            ("file.otf", "font/otf"),
            ("file.woff", "font/woff"),
            ("file.woff2", "font/woff2"),
            ("file.svg", "image/svg+xml"),
            ("file.ico", "image/x-icon"),
            ("file.jpeg", "image/jpeg"),
            ("file.png", "image/png"),
            ("file.apng", "image/apng"),
            ("caddy.gif", "image/gif"),
            ("file.webp", "image/webp"),
            ("file.avif", "image/avif"),
            ("file.toml", "application/toml"),
            ("file.xml", "application/xml"),
            ("file.json", "application/json"),
            ("file.js", "text/javascript"),
            ("file.pdf", "application/pdf"),
            ("book.epub", "application/epub+zip"),
            ("weird.xzx", "application/octet-stream"),
        ];

        for (file, mime) in pairs {
            assert_eq!(String::from(Mime::guess(file)), mime);
        }
    }

    #[test]
    fn unknown() {
        let u = Mime::guess("x");
        assert!(matches!(u, Mime::Unknown));
    }
}
