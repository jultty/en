use crate::graph::Graph;

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
    Custom(String),
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
        match mime {
            Mime::Txt => "text/plain".into(),
            Mime::Csv => "text/csv".into(),
            Mime::Css => "text/css".into(),
            Mime::Ttf => "font/ttf".into(),
            Mime::Otf => "font/otf".into(),
            Mime::Woff => "font/woff".into(),
            Mime::Woff2 => "font/woff2".into(),
            Mime::Svg => "image/svg+xml".into(),
            Mime::Ico => "image/x-icon".into(),
            Mime::Jpeg => "image/jpeg".into(),
            Mime::Png => "image/png".into(),
            Mime::Apng => "image/apng".into(),
            Mime::Gif => "image/gif".into(),
            Mime::Webp => "image/webp".into(),
            Mime::Avif => "image/avif".into(),
            Mime::Toml => "application/toml".into(),
            Mime::Xml => "application/xml".into(),
            Mime::Json => "application/json".into(),
            Mime::Js => "text/javascript".into(),
            Mime::Pdf => "application/pdf".into(),
            Mime::Epub => "application/epub+zip".into(),
            Mime::Unknown => "application/octet-stream".into(),
            Mime::Custom(value) => value,
        }
    }
}

pub enum Kind {
    Text,
    Font,
    Image,
    Blob,
}

impl Mime {
    /// Returns a mimetypegiven a filename extension and a graph. The graph
    /// is used to read custom mimetypes from the configuration.
    ///
    /// Only considers the last dot-delimited fragment of `path`.
    pub fn from_extension(path: &str, graph: &Graph) -> Mime {
        if let Some(pair) = path.rsplit_once('.') {
            #[expect(clippy::wildcard_enum_match_arm)]
            match Mime::from(pair.1) {
                Mime::Unknown => match graph.meta.config.mime.get(pair.1) {
                    Some(custom) => Mime::Custom(custom.clone()),
                    None => Mime::Unknown,
                },
                other => other,
            }
        } else {
            Mime::Unknown
        }
    }

    /// Returns one of four kind of mimetypes among text, font, image and blob.
    ///
    /// This is mainly used when serving assets through the `fixed` module in
    /// order to determine what `Asset` field to use when assemblimg a response
    /// body.
    pub const fn kind(&self) -> Kind {
        use Mime::*;
        match self {
            Txt | Csv | Css | Toml | Xml | Json | Js | Svg => Kind::Text,
            Ttf | Otf | Woff | Woff2 => Kind::Font,
            Ico | Jpeg | Png | Apng | Gif | Webp | Avif => Kind::Image,
            Pdf | Epub | Custom(_) | Unknown => Kind::Blob,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let m = Mime::from_extension(
            "/home/jane/top/inner/kitty.png",
            &Graph::default(),
        );
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
            assert_eq!(
                String::from(Mime::from_extension(file, &Graph::default())),
                mime
            );
        }
    }

    #[test]
    fn unknown() {
        let u = Mime::from_extension("x", &Graph::default());
        assert!(matches!(u, Mime::Unknown));
    }

    #[test]
    #[expect(clippy::shadow_unrelated, unused)]
    fn custom() {
        let payload = String::from("mime/custom");
        let graph = Graph::from_serial(
            &format!(
                "[meta.config]\n\
                mime = {{ custom = \"{payload}\" }}"
            ),
            &crate::graph::Format::TOML,
        )
        .unwrap();
        assert!(matches!(
            Mime::from_extension("file.custom", &graph),
            Mime::Custom(payload),
        ));
    }
}
