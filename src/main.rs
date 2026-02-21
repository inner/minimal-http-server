mod http;
mod http_request;
mod http_response;
mod router;
mod thread_pool;

use self::http::headers::{OCTET_STREAM, TEXT_PLAIN};
use self::http_request::{HttpRequest, Method};
use self::http_response::HttpResponse;
use self::router::Router;
use self::thread_pool::ThreadPool;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;

fn handle_root(_: &HttpRequest, _: &HashMap<String, String>) -> HttpResponse {
    HttpResponse::ok()
}

fn handle_echo(req: &HttpRequest, _: &HashMap<String, String>) -> HttpResponse {
    let echo = req.path.strip_prefix("/echo/").unwrap_or("");
    HttpResponse::ok()
        .with_content_type(TEXT_PLAIN)
        .with_body(echo.as_bytes().into())
}

fn handle_user_agent_header_read(req: &HttpRequest, _: &HashMap<String, String>) -> HttpResponse {
    let user_agent = req.headers.get("user-agent").unwrap();
    HttpResponse::ok()
        .with_content_type(TEXT_PLAIN)
        .with_body(user_agent.as_bytes().into())
}

fn handle_read_body(req: &HttpRequest, args: &HashMap<String, String>) -> HttpResponse {
    let file_name = req.path.strip_prefix("/files/").unwrap_or("");
    let d = args.get("directory").unwrap();
    let path = Path::new(d).join(file_name);
    let _f = File::create(path).unwrap().write(&req.body);
    HttpResponse::created()
}

fn handle_return_file(req: &HttpRequest, args: &HashMap<String, String>) -> HttpResponse {
    let file_name = req.path.strip_prefix("/files/").unwrap_or("");
    if let Some(d) = args.get("directory") {
        if let Ok(mut f) = File::open(d.to_string() + file_name) {
            let mut contents: Vec<u8> = Vec::new();
            let _ = f.read_to_end(&mut contents).unwrap();
            HttpResponse::ok()
                .with_content_type(OCTET_STREAM)
                .with_body(contents)
        } else {
            HttpResponse::not_found()
        }
    } else {
        HttpResponse::not_found()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;
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

    let router = Router::new()
        .add(Method::Get, "/", handle_root)
        .add(Method::Get, "/echo", handle_echo)
        .add(Method::Get, "/user-agent", handle_user_agent_header_read)
        .add(Method::Get, "/files", handle_return_file)
        .add(Method::Post, "/files", handle_read_body)
        .build_arc();

    let pool = ThreadPool::build(10)?;
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let map = Arc::clone(&map);
                let router = Arc::clone(&router);
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
    args: Arc<HashMap<String, String>>,
    router: Arc<Router>,
    mut stream: TcpStream,
) -> Result<(), Box<dyn Error>> {
    let request = HttpRequest::new(&stream)?;
    let response = router.handle(&request, &args);
    stream.write_all(&response.as_bytes())?;

    Ok(())
}
