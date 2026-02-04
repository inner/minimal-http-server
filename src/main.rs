mod http_request;
mod http_response;

use crate::http_request::HttpRequest;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

use self::http_response::HttpResponse;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(move || {
                    handle_connection(s).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let request = HttpRequest::new(&stream)?;

    let status_line = if request.path == "/" {
        let response = HttpResponse {
            http_line: String::from("HTTP/1.1 200 OK"),
            body: String::new(),
            headers: HashMap::new(),
        };
        response.as_bytes()
    } else if let Some(echo) = request.path.strip_prefix("/echo/") {
        let mut headers = HashMap::new();

        headers.insert(
            String::from("Content-Length"),
            String::from(echo.len().to_string()),
        );

        headers.insert(String::from("Content-Type"), String::from("text/plain"));

        let response = HttpResponse {
            http_line: String::from("HTTP/1.1 200 OK"),
            headers,
            body: String::from(echo),
        };
        response.as_bytes()
    } else {
        let response = HttpResponse {
            http_line: String::from("HTTP/1.1 404 Not Found"),
            body: String::new(),
            headers: HashMap::new(),
        };
        response.as_bytes()
    };

    stream.write_all(&status_line)?;

    Ok(())
}
