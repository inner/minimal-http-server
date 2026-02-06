use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
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

        let parts: Vec<&str> = http_line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid request line",
            ));
        }

        let (method, path, version) = (parts[0], parts[1], parts[2]);
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut total_header_size = 0;

        loop {
            let mut header_line = String::new();
            reader.read_line(&mut header_line)?;

            total_header_size += header_line.len();
            if total_header_size > MAX_HEADER_SIZE {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "headers too large",
                ));
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "body too large",
            ));
        }

        let mut body = Vec::new();
        if content_length > 0 {
            body = vec![0u8; content_length];
            reader.read_exact(&mut body)?;
        }

        Ok(Self {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}
