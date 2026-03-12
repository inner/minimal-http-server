mod files;
mod http;
mod request;
mod response;
mod router;
mod threadpool;

use self::files::FileManager;
use self::http::headers::{CONNECTION, OCTET_STREAM, TEXT_PLAIN};
use self::request::{HttpRequest, Method};
use self::response::HttpResponse;
use self::router::Router;
use self::threadpool::ThreadPool;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::{BufReader, ErrorKind, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn handle_root(_: &HttpRequest, _: &HashMap<String, String>) -> HttpResponse {
    HttpResponse::ok()
}

fn handle_echo(req: &HttpRequest, _: &HashMap<String, String>) -> HttpResponse {
    let Some(echo) = req.path.strip_prefix("/echo/") else {
        return HttpResponse::not_found();
    };

    let res = HttpResponse::ok()
        .with_content_type(TEXT_PLAIN)
        .with_body(echo.as_bytes().into());

    let Some(content_encoding) = req.headers.get("accept-encoding") else {
        return res;
    };

    res.with_encoding(String::from(content_encoding))
        .with_gzip_body()
}

fn handle_user_agent_header_read(req: &HttpRequest, _: &HashMap<String, String>) -> HttpResponse {
    let Some(user_agent) = req.headers.get("user-agent") else {
        return HttpResponse::not_found();
    };

    HttpResponse::ok()
        .with_content_type(TEXT_PLAIN)
        .with_body(user_agent.as_bytes().into())
}

fn handle_read_body(req: &HttpRequest, args: &HashMap<String, String>) -> HttpResponse {
    let Some(file_name) = req.path.strip_prefix("/files/") else {
        return HttpResponse::not_found();
    };

    let Some(path) = args.get("directory") else {
        return HttpResponse::not_found();
    };

    match FileManager::create(Path::new(path), file_name, &req.body) {
        Ok(_) => HttpResponse::created(),
        Err(_) => HttpResponse::not_found(),
    }
}

fn handle_return_file(req: &HttpRequest, args: &HashMap<String, String>) -> HttpResponse {
    let Some(file_name) = req.path.strip_prefix("/files/") else {
        return HttpResponse::not_found();
    };

    let Some(d) = args.get("directory") else {
        return HttpResponse::not_found();
    };

    let Ok(contents) = FileManager::read(Path::new(d), file_name) else {
        return HttpResponse::not_found();
    };

    HttpResponse::ok()
        .with_content_type(OCTET_STREAM)
        .with_body(contents)
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;
    let args: Vec<String> = env::args().skip(1).collect();
    let map: &HashMap<String, String> = Box::leak(Box::new(
        args.chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 && chunk[0].starts_with("--") {
                    Some((chunk[0][2..].to_string(), chunk[1].clone()))
                } else {
                    None
                }
            })
            .collect(),
    ));

    let router = Box::leak(Box::new(
        Router::new()
            .add(Method::Get, "/", handle_root)
            .add(Method::Get, "/echo", handle_echo)
            .add(Method::Get, "/user-agent", handle_user_agent_header_read)
            .add(Method::Get, "/files", handle_return_file)
            .add(Method::Post, "/files", handle_read_body),
    ));

    let pool = ThreadPool::build(10)?;
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                pool.execute(|| {
                    if let Err(e) = handle_connection(map, router, s) {
                        eprintln!("connection error: {e}");
                    }
                })?;
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_connection(
    args: &HashMap<String, String>,
    router: &Router,
    stream: TcpStream,
) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(&stream);
    loop {
        let request = match HttpRequest::new(&mut reader) {
            Ok(r) => r,
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        };

        let mut response = router.handle(&request, &args);
        let keep_alive = !request.close_connection;

        if !keep_alive {
            response.headers.insert(CONNECTION, "close".to_string());
        }

        (&stream).write_all(&response.as_bytes())?;

        if !keep_alive {
            break;
        }
    }

    Ok(())
}
