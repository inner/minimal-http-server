use std::collections::HashMap;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read};
use std::net::TcpStream;

const MAX_HEADER_SIZE: usize = 8 * 1024;
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

#[derive(Debug)]
#[allow(dead_code)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(stream: &TcpStream) -> std::io::Result<HttpRequest> {
        let mut reader = BufReader::new(stream);

        let mut http_line = String::new();
        reader.read_line(&mut http_line)?;

        let mut parts = http_line.split_whitespace();

        let method = parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing method"))?;

        let path = parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing path"))?;

        let version = parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing version"))?;

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

            if let Some(colon_pos) = header_line.find(":") {
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

        Ok(Self {
            method: method.to_string(),
            path: path.to_string(),
            version: version.to_string(),
            headers,
            body,
        })
    }
}
