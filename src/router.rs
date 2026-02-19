use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

use crate::http;
use crate::http_request::{HttpRequest, Method};
use crate::http_response::HttpResponse;

#[allow(dead_code)]
pub struct Router;

#[allow(dead_code)]
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
            let mut headers = HashMap::new();
            headers.insert(http::headers::CONTENT_LENGTH, echo.len().to_string());
            headers.insert(
                http::headers::CONTENT_TYPE,
                http::headers::TEXT_PLAIN.to_string(),
            );

            HttpResponse {
                status_line: http::status::OK,
                headers,
                body: echo.as_bytes().into(),
            }
        } else if req.path.starts_with("/user-agent") {
            let user_agent = req.headers.get("user-agent").unwrap();
            let mut headers = HashMap::new();
            headers.insert(http::headers::CONTENT_LENGTH, user_agent.len().to_string());
            headers.insert(
                http::headers::CONTENT_TYPE,
                http::headers::TEXT_PLAIN.to_string(),
            );

            HttpResponse {
                status_line: http::status::OK,
                headers,
                body: user_agent.as_bytes().into(),
            }
        } else if let Some(file_name) = req.path.strip_prefix("/files/") {
            if let Some(d) = args.get("directory") {
                if let Ok(mut f) = File::open(d.to_string() + file_name) {
                    let mut contents: Vec<u8> = Vec::new();
                    let content_len = f.read_to_end(&mut contents).unwrap();

                    let mut headers = HashMap::new();
                    headers.insert(http::headers::CONTENT_LENGTH, content_len.to_string());
                    headers.insert(
                        http::headers::CONTENT_TYPE,
                        http::headers::OCTET_STREAM.to_string(),
                    );

                    HttpResponse {
                        status_line: http::status::OK,
                        headers,
                        body: contents,
                    }
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
