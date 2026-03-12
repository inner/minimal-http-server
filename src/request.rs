use std::collections::HashMap;
use std::io::{BufRead, Error, ErrorKind, Result};

const MAX_HEADER_SIZE: usize = 8 * 1024;
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Unknown,
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

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub keep_alive: bool,
}

impl HttpRequest {
    pub fn new<R: BufRead>(reader: &mut R) -> Result<HttpRequest> {
        let mut http_line = String::new();
        reader.read_line(&mut http_line)?;
        if http_line.is_empty() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "connection closed"));
        }

        let mut parts = http_line.split_whitespace();

        let method: Method = parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing method"))?
            .into();

        let path = parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing path"))?;

        let version: Version = parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing version"))?
            .into();

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut total_header_size = 0;
        let mut header_line = String::new();

        loop {
            header_line.clear();
            reader.read_line(&mut header_line)?;

            total_header_size += header_line.len();
            if total_header_size > MAX_HEADER_SIZE {
                return Err(Error::new(ErrorKind::InvalidInput, "headers too large"));
            }

            if header_line == "\r\n" || header_line == "\n" {
                break;
            }

            if let Some(colon_pos) = header_line.find(':') {
                let name = header_line[..colon_pos].trim().to_lowercase();
                let value = header_line[colon_pos + 1..].trim_end().trim().to_string();
                headers.insert(name, value);
            }
        }

        let content_length = headers
            .get("content-length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        if content_length > MAX_BODY_SIZE {
            return Err(Error::new(ErrorKind::InvalidInput, "body too large"));
        }

        let mut body = Vec::new();
        if content_length > 0 {
            body = vec![0u8; content_length];
            reader.read_exact(&mut body)?;
        }

        let keep_alive =
            !headers.get("connection").is_some_and(|v| v == "close") && version == Version::Http11;

        println!("version: {:?}", version);
        println!("alive: {}", keep_alive);

        Ok(Self {
            method,
            path: path.to_string(),
            headers,
            body,
            keep_alive,
        })
    }
}
