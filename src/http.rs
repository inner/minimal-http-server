pub mod status {
    pub const OK: &str = "HTTP/1.1 200 OK";
    pub const NOT_FOUND: &str = "HTTP/1.1 404 Not Found";
    pub const NOT_ALLOWED: &str = "HTTP/1.1 405 Method Not Allowed";
    pub const CREATED: &str = "HTTP/1.1 201 Created";
}

pub mod headers {
    pub const CONTENT_LENGTH: &str = "Content-Length";
    pub const CONNECTION: &str = "Connection";
    pub const CONTENT_TYPE: &str = "Content-Type";
    pub const CONTENT_ENCODING: &str = "Content-Encoding";
    pub const TEXT_PLAIN: &str = "text/plain";
    pub const ALLOW: &str = "Allow";
    pub const OCTET_STREAM: &str = "application/octet-stream";
}

pub mod compression {
    use std::str::FromStr;

    #[derive(Debug, PartialEq)]
    pub enum Encoding {
        Gzip,
    }

    #[derive(Debug)]
    pub struct ParseEncodingError;

    impl FromStr for Encoding {
        type Err = ParseEncodingError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "gzip" => Ok(Encoding::Gzip),
                _ => Err(ParseEncodingError),
            }
        }
    }
}
