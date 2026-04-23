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

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HeaderName {
    ContentLength,
    ContentType,
    Connection,
    ContentEncoding,
    AcceptEncoding,
    UserAgent,
    Allow,
    Custom(String),
}

impl HeaderName {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ContentLength => "Content-Length",
            Self::ContentType => "Content-Type",
            Self::Connection => "Connection",
            Self::ContentEncoding => "Content-Encoding",
            Self::AcceptEncoding => "Accept-Encoding",
            Self::UserAgent => "User-Agent",
            Self::Allow => "Allow",
            Self::Custom(value) => value.as_str(),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HeaderValue {
    TextPlain,
    OctetStream,
    Custom(String),
}

impl HeaderValue {
    pub fn as_str(&self) -> &str {
        match self {
            Self::TextPlain => "text/plain",
            Self::OctetStream => "application/octet-stream",
            Self::Custom(value) => value.as_str(),
        }
    }
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
