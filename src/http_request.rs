use std::net::TcpStream;

pub struct HttpRequest {
    http_line: String,
    headers: Vec<String>,
    body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(stream: &TcpStream) -> HttpRequest {
        HttpRequest {
            http_line: String::from("test"),
            headers: vec![],
            body: vec![],
        }
    }
}
