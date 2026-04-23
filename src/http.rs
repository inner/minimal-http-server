#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    PayloadTooLarge = 413,
    RequestHeadersTooLarge = 431,
    InternalServerError = 500,
    HttpVersionNotSupported = 505,
}

impl StatusCode {
    pub fn reason(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Created => "Created",
            Self::BadRequest => "Bad Request",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::PayloadTooLarge => "Payload Too Large",
            Self::RequestHeadersTooLarge => "Request Header Fields Too Large",
            Self::InternalServerError => "Internal Server Error",
            Self::HttpVersionNotSupported => "HTTP Version Not Supported",
        }
    }
}

// pub mod status {
//     pub const OK: &str = "HTTP/1.1 200 OK";
//     pub const NOT_ALLOWED: &str = "HTTP/1.1 405 Method Not Allowed";
//     pub const CREATED: &str = "HTTP/1.1 201 Created";
//     pub const NOT_FOUND: &str = "HTTP/1.1 404 Not Found";
// }

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
