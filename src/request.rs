use std::collections::HashMap;
use std::io::{BufRead, Read};
use std::str::SplitWhitespace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Unknown,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Unknown => "",
        }
    }
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        match value {
            "GET" => Method::Get,
            "POST" => Method::Post,
            _ => Method::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Version {
    Http10,
    Http11,
    Unknown,
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        match s {
            "HTTP/1.0" => Version::Http10,
            "HTTP/1.1" => Version::Http11,
            _ => Version::Unknown,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub version: Version,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub keep_alive: bool,
}

const MAX_HTTP_LINE_SIZE: usize = 8 * 1024;
const MAX_HEADER_SIZE: usize = 8 * 1024;
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

#[allow(dead_code)]
pub enum RequestParseError {
    ConnectionClosed,
    InvalidRequestLine,
    HeadersTooLarge,
    BodyTooLarge,
    UnsupportedMethod,
    UnsupportedVersion,
    MalformedRequestLine,
    Io(std::io::Error),
}

impl From<std::io::Error> for RequestParseError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub type RequestResult<T> = Result<T, RequestParseError>;

fn parse_method(parts: &mut SplitWhitespace) -> RequestResult<Method> {
    let method_str = parts.next().ok_or(RequestParseError::InvalidRequestLine)?;
    let method = Method::from(method_str);

    if method == Method::Unknown {
        return Err(RequestParseError::UnsupportedMethod);
    }

    Ok(method)
}

fn parse_path<'a>(parts: &mut SplitWhitespace<'a>) -> RequestResult<&'a str> {
    let path = parts.next().ok_or(RequestParseError::InvalidRequestLine)?;
    Ok(path)
}

fn parse_version(parts: &mut SplitWhitespace) -> RequestResult<Version> {
    let version_str = parts.next().ok_or(RequestParseError::InvalidRequestLine)?;
    let version = Version::from(version_str);

    if version == Version::Unknown {
        return Err(RequestParseError::UnsupportedVersion);
    }

    Ok(version)
}

impl HttpRequest {
    pub fn parse<R: BufRead>(reader: &mut R) -> RequestResult<HttpRequest> {
        let mut http_line = String::new();
        reader
            .take(MAX_HTTP_LINE_SIZE as u64 + 1)
            .read_line(&mut http_line)?;

        if http_line.is_empty() {
            return Err(RequestParseError::ConnectionClosed);
        }

        if http_line.len() > MAX_HTTP_LINE_SIZE {
            return Err(RequestParseError::InvalidRequestLine);
        }

        let mut parts = http_line.split_whitespace();

        if parts.by_ref().count() != 3 {
            return Err(RequestParseError::MalformedRequestLine);
        }

        let method = parse_method(&mut parts)?;
        let path = parse_path(&mut parts)?;
        let version = parse_version(&mut parts)?;

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut total_header_size = 0;
        let mut header_line = String::new();

        loop {
            header_line.clear();
            reader
                .take(MAX_HEADER_SIZE as u64 + 1)
                .read_line(&mut header_line)?;

            total_header_size += header_line.len();
            if total_header_size > MAX_HEADER_SIZE {
                return Err(RequestParseError::HeadersTooLarge);
            }

            if header_line == "\r\n" || header_line == "\n" {
                break;
            }

            if let Some(colon_pos) = header_line.find(':') {
                let name = header_line[..colon_pos].trim().to_lowercase();
                let value = header_line[colon_pos + 1..].trim().to_string();
                headers.insert(name, value);
            }
        }

        let content_length = headers
            .get("content-length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        if content_length > MAX_BODY_SIZE {
            return Err(RequestParseError::BodyTooLarge);
        }

        let mut body = Vec::new();
        if content_length > 0 {
            body = vec![0u8; content_length];
            reader.read_exact(&mut body)?;
        }

        let keep_alive =
            !headers.get("connection").is_some_and(|v| v == "close") && version == Version::Http11;

        Ok(Self {
            method,
            version,
            path: path.to_string(),
            headers,
            body,
            keep_alive,
        })
    }
}
