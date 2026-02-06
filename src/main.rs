mod http_request;
mod http_response;

use crate::http_request::HttpRequest;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

use self::http_response::HttpResponse;

const STATUS_LINE_200: &str = "HTTP/1.1 200 OK";
const STATUS_LINE_404: &str = "HTTP/1.1 404 Not Found";

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
            http_status_line: String::from(STATUS_LINE_200),
            body: String::new(),
            headers: HashMap::new(),
        };
        response.as_bytes()
    } else if let Some(echo) = request.path.strip_prefix("/echo/") {
        let mut headers = HashMap::new();
        headers.insert(String::from("Content-Length"), echo.len().to_string());
        headers.insert(String::from("Content-Type"), String::from("text/plain"));

        let response = HttpResponse {
            http_status_line: String::from(STATUS_LINE_200),
            headers,
            body: String::from(echo),
        };
        response.as_bytes()
    } else if let Some(_) = request.path.strip_prefix("/user-agent") {
        if let Some(user_agent) = request.headers.get("User-Agent") {
            let mut headers = HashMap::new();
            headers.insert("Content-Length".to_string(), user_agent.len().to_string());

            let response = HttpResponse {
                http_status_line: String::from(STATUS_LINE_200),
                headers,
                body: String::from(user_agent),
            };
            response.as_bytes()
        } else {
            let response = HttpResponse {
                http_status_line: String::from(STATUS_LINE_404),
                headers: HashMap::new(),
                body: String::new(),
            };
            response.as_bytes()
        }
    } else {
        let response = HttpResponse {
            http_status_line: String::from(STATUS_LINE_404),
            body: String::new(),
            headers: HashMap::new(),
        };
        response.as_bytes()
    };

    stream.write_all(&status_line)?;

    Ok(())
}
