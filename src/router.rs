use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

use crate::http::headers::{OCTET_STREAM, TEXT_PLAIN};
use crate::http_request::{HttpRequest, Method};
use crate::http_response::HttpResponse;

pub struct Router;

impl Router {
    pub fn handle(req: &HttpRequest, args: &HashMap<String, String>) -> HttpResponse {
        match req.method {
            Method::Get => Self::get(req, args),
            Method::Post => Self::post(req, args),
            Method::Unknown => Self::unknown(),
        }
    }

    fn get(req: &HttpRequest, args: &HashMap<String, String>) -> HttpResponse {
        if req.path == "/" {
            HttpResponse::ok()
        } else if let Some(echo) = req.path.strip_prefix("/echo/") {
            HttpResponse::ok()
                .with_content_type(TEXT_PLAIN)
                .with_body(echo.as_bytes().into())
        } else if req.path.starts_with("/user-agent") {
            let user_agent = req.headers.get("user-agent").unwrap();
            HttpResponse::ok()
                .with_content_type(TEXT_PLAIN)
                .with_body(user_agent.as_bytes().into())
        } else if let Some(file_name) = req.path.strip_prefix("/files/") {
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
        } else {
            HttpResponse::not_found()
        }
    }

    fn post(req: &HttpRequest, args: &HashMap<String, String>) -> HttpResponse {
        if let Some(file_name) = req.path.strip_prefix("/files/") {
            let d = args.get("directory").unwrap();
            let _f = File::create(d.to_string() + file_name)
                .unwrap()
                .write(&req.body);
            HttpResponse::created()
        } else {
            HttpResponse::not_found()
        }
    }

    fn unknown() -> HttpResponse {
        HttpResponse::not_found()
    }
}
