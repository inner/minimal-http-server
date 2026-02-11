mod http;
mod http_request;
mod http_response;

use self::http_request::HttpRequest;
use self::http_response::HttpResponse;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::{env, thread};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let args: Vec<String> = env::args().skip(1).collect();
    let map: Arc<HashMap<String, String>> = Arc::new(
        args.chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 && chunk[0].starts_with("--") {
                    Some((chunk[0][2..].to_string(), chunk[1].clone()))
                } else {
                    None
                }
            })
            .collect(),
    );

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let map = Arc::clone(&map);
                thread::spawn(move || {
                    handle_connection(map, s).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(
    args: Arc<HashMap<String, String>>,
    mut stream: TcpStream,
) -> std::io::Result<()> {
    let request = HttpRequest::new(&stream)?;

    let status_line = if request.method == "GET" && request.path == "/" {
        let response = HttpResponse {
            http_status_line: http::status::OK,
            headers: HashMap::new(),
            body: "",
        };
        response.as_bytes()
    } else if request.method == "GET"
        && let Some(echo) = request.path.strip_prefix("/echo/")
    {
        let mut headers = HashMap::new();
        headers.insert(
            http::headers::CONTENT_LENGTH,
            Cow::Owned(echo.len().to_string()),
        );
        headers.insert(
            http::headers::CONTENT_TYPE,
            Cow::Borrowed(http::headers::TEXT_PLAIN),
        );

        let response = HttpResponse {
            http_status_line: http::status::OK,
            headers,
            body: echo,
        };
        response.as_bytes()
    } else if request.method == "GET" && request.path.starts_with("/user-agent") {
        if let Some(user_agent) = request.headers.get("user-agent") {
            let mut headers = HashMap::new();
            headers.insert(
                http::headers::CONTENT_LENGTH,
                Cow::Owned(user_agent.len().to_string()),
            );
            headers.insert(
                http::headers::CONTENT_TYPE,
                Cow::Borrowed(http::headers::TEXT_PLAIN),
            );

            let response = HttpResponse {
                http_status_line: http::status::OK,
                headers,
                body: user_agent,
            };
            response.as_bytes()
        } else {
            let response = HttpResponse {
                http_status_line: http::status::NOT_FOUND,
                headers: HashMap::new(),
                body: "",
            };
            response.as_bytes()
        }
    } else if request.method == "GET"
        && let Some(file_name) = request.path.strip_prefix("/files/")
    {
        if let Some(d) = args.get("directory") {
            if let Ok(mut f) = File::open(d.to_string() + file_name) {
                let mut contents = String::new();
                let bytes = f.read_to_string(&mut contents)?;

                let mut headers = HashMap::new();
                headers.insert(http::headers::CONTENT_LENGTH, Cow::Owned(bytes.to_string()));
                headers.insert(
                    http::headers::CONTENT_TYPE,
                    Cow::Borrowed(http::headers::OCTET_STREAM),
                );

                let response = HttpResponse {
                    http_status_line: http::status::OK,
                    headers,
                    body: &contents,
                };
                response.as_bytes()
            } else {
                let response = HttpResponse {
                    http_status_line: http::status::NOT_FOUND,
                    headers: HashMap::new(),
                    body: "",
                };
                response.as_bytes()
            }
        } else {
            let response = HttpResponse {
                http_status_line: http::status::NOT_FOUND,
                headers: HashMap::new(),
                body: "",
            };
            response.as_bytes()
        }
    } else if request.method == "POST"
        && let Some(_f) = request.path.strip_suffix("/files/")
    {
        let response = HttpResponse {
            http_status_line: http::status::NOT_FOUND,
            headers: HashMap::new(),
            body: "",
        };
        response.as_bytes()
    } else {
        let response = HttpResponse {
            http_status_line: http::status::NOT_FOUND,
            headers: HashMap::new(),
            body: "",
        };
        response.as_bytes()
    };

    stream.write_all(&status_line)?;

    Ok(())
}
