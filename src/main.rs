mod app;
mod files;
mod http;
mod middlewares;
mod request;
mod response;
mod router;
mod threadpool;

use self::files::FileManager;
use self::http::{HeaderName, HeaderValue};
use self::middlewares::Middlewares;
use self::request::{HttpRequest, Method};
use self::response::HttpResponse;
use self::router::{Match, Router};
use self::threadpool::ThreadPool;

use crate::files::FileError;
use clap::Parser;
use std::error::Error;
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;

pub type AppResult<T> = Result<T, AppError>;

pub enum AppError {
    BadRequest(&'static str),
    Forbidden(&'static str),
    NotFound,
    MethodNotAllowed { allow: Vec<Method> },
    Internal(&'static str),
}

impl AppError {
    fn into_response(self) -> HttpResponse {
        match self {
            Self::BadRequest(_) => HttpResponse::bad_request(),
            Self::Forbidden(_) => HttpResponse::forbidden(),
            Self::NotFound => HttpResponse::not_found(),
            Self::MethodNotAllowed { allow } => HttpResponse::not_allowed(&allow),
            Self::Internal(_) => HttpResponse::internal_server_error(),
        }
    }
}

impl From<FileError> for AppError {
    fn from(err: FileError) -> Self {
        match err {
            FileError::InvalidPath => AppError::Forbidden("invalid path"),
            FileError::NotFound => AppError::NotFound,
            FileError::PermissionDenied => AppError::Forbidden("permission denied"),
            FileError::Io(_) => AppError::Internal("file I/O failed"),
        }
    }
}

fn handle_root(_: &HttpRequest, _: &Args, _: &matchit::Params) -> AppResult<HttpResponse> {
    Ok(HttpResponse::ok())
}

fn handle_echo(_req: &HttpRequest, _: &Args, params: &matchit::Params) -> AppResult<HttpResponse> {
    let echo = params
        .get("echo")
        .ok_or(AppError::BadRequest("Missing param: echo"))?;

    let res = HttpResponse::ok()
        .with_content_type(HeaderValue::TextPlain)
        .with_body(echo.as_bytes().into());

    Ok(res)
}

fn handle_user_agent_header_read(
    req: &HttpRequest,
    _: &Args,
    _: &matchit::Params,
) -> AppResult<HttpResponse> {
    let user_agent = req
        .headers
        .get("user-agent")
        .ok_or(AppError::BadRequest("Missing user-agent"))?;

    Ok(HttpResponse::ok()
        .with_content_type(HeaderValue::TextPlain)
        .with_body(user_agent.as_bytes().into()))
}

fn handle_read_body(
    req: &HttpRequest,
    args: &Args,
    params: &matchit::Params,
) -> AppResult<HttpResponse> {
    let dir = args
        .directory
        .as_deref()
        .ok_or(AppError::BadRequest("Missing directory"))?;

    let file_name = params
        .get("file")
        .ok_or(AppError::BadRequest("Missing file"))?;

    FileManager::create(Path::new(dir), file_name, &req.body)?;
    Ok(HttpResponse::created())
}

fn handle_return_file(
    _: &HttpRequest,
    args: &Args,
    params: &matchit::Params,
) -> AppResult<HttpResponse> {
    let file_name = params
        .get("file")
        .ok_or(AppError::BadRequest("Missing file"))?;

    let dir = args
        .directory
        .as_deref()
        .ok_or(AppError::BadRequest("Missing directory"))?;

    let contents = FileManager::read(Path::new(dir), file_name)?;

    Ok(HttpResponse::ok()
        .with_content_type(HeaderValue::OctetStream)
        .with_body(contents))
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    directory: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // working on changes
    // App::new()
    //     .with_route(Method::Get, "/", handle_root)
    //     .with_route(Method::Get, "/echo", handle_echo)
    //     .run();

    let args = Arc::new(Args::parse());
    let listener = TcpListener::bind("0.0.0.0:4221")?;

    let router = Arc::new(
        Router::new()
            .route(Method::Get, "/", handle_root)
            .route(Method::Get, "/echo/{echo}", handle_echo)
            .route(Method::Get, "/user-agent", handle_user_agent_header_read)
            .route(Method::Get, "/files/{file}", handle_return_file)
            .route(Method::Post, "/files/{file}", handle_read_body),
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
        let request = match HttpRequest::parse(&mut reader) {
            Ok(req) => req,
            Err(err) => {
                let Some(mut response) = err.into_response() else {
                    break;
                };

                response
                    .headers
                    .insert(HeaderName::Connection, "close".to_string());
                response
                    .headers
                    .insert(HeaderName::ContentLength, response.body.len().to_string());

                (&stream).write_all(&response.as_bytes())?;
                break;
            }
        };

        let handler_result = match router.find(&request.path, &request.method) {
            Match::Found(handler, params) => handler(&request, args, &params),
            Match::NotFound => Err(AppError::NotFound),
            Match::MethodNotAllowed(allowed) => Err(AppError::MethodNotAllowed { allow: allowed }),
        };

        let mut response = handler_result.unwrap_or_else(AppError::into_response);

        Middlewares::run(&request, &mut response);

        (&stream).write_all(&response.as_bytes())?;

        if !request.keep_alive {
            break;
        }
    }

    Ok(())
}
