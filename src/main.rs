mod files;
mod http;
mod middlewares;
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

use clap::Parser;
use std::error::Error;
use std::io::{BufReader, ErrorKind, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;

fn handle_root(_: &HttpRequest, _: &Args) -> HttpResponse {
    HttpResponse::ok()
}

fn handle_echo(req: &HttpRequest, _: &Args) -> HttpResponse {
    let Some(echo) = req.path.strip_prefix("/echo/") else {
        return HttpResponse::not_found();
    };

    let res = HttpResponse::ok()
        .with_content_type(TEXT_PLAIN)
        .with_body(echo.as_bytes().into());

    res
}

fn handle_user_agent_header_read(req: &HttpRequest, _: &Args) -> HttpResponse {
    let Some(user_agent) = req.headers.get("user-agent") else {
        return HttpResponse::not_found();
    };

    HttpResponse::ok()
        .with_content_type(TEXT_PLAIN)
        .with_body(user_agent.as_bytes().into())
}

fn handle_read_body(req: &HttpRequest, args: &Args) -> HttpResponse {
    let Some(file_name) = req.path.strip_prefix("/files/") else {
        return HttpResponse::not_found();
    };

    let Some(d) = args.directory.as_deref() else {
        return HttpResponse::not_found();
    };

    match FileManager::create(Path::new(d), file_name, &req.body) {
        Ok(_) => HttpResponse::created(),
        Err(_) => HttpResponse::not_found(),
    }
}

fn handle_return_file(req: &HttpRequest, args: &Args) -> HttpResponse {
    let Some(file_name) = req.path.strip_prefix("/files/") else {
        return HttpResponse::not_found();
    };

    let Some(d) = args.directory.as_deref() else {
        return HttpResponse::not_found();
    };

    let Ok(contents) = FileManager::read(Path::new(d), file_name) else {
        return HttpResponse::not_found();
    };

    HttpResponse::ok()
        .with_content_type(OCTET_STREAM)
        .with_body(contents)
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    directory: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arc::new(Args::parse());
    let listener = TcpListener::bind("127.0.0.1:4221")?;

    let router = Arc::new(
        Router::new()
            .add(Method::Get, "/", handle_root)
            .add(Method::Get, "/echo", handle_echo)
            .add(Method::Get, "/user-agent", handle_user_agent_header_read)
            .add(Method::Get, "/files", handle_return_file)
            .add(Method::Post, "/files", handle_read_body),
    );

    let pool = ThreadPool::build(10)?;
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let args = Arc::clone(&args);
                let router = Arc::clone(&router);
                pool.execute(move || {
                    if let Err(e) = handle_connection(&args, &router, s) {
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
    args: &Args,
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

        if !request.keep_alive {
            response.headers.insert(CONNECTION, "close".to_string());
        }

        (&stream).write_all(&response.as_bytes())?;

        if !request.keep_alive {
            break;
        }
    }

    Ok(())
}
