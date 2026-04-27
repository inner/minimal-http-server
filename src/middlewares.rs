use crate::http::HeaderName;
use crate::http::compression::Encoding;
use crate::request::HttpRequest;
use crate::response::HttpResponse;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

pub type Middleware = fn(&HttpRequest, &mut HttpResponse);

pub struct MiddlewarePipeline {
    middlewares: Vec<Middleware>,
}

impl MiddlewarePipeline {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add(mut self, middleware: Middleware) -> Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn run(&self, req: &HttpRequest, res: &mut HttpResponse) {
        for middleware in &self.middlewares {
            middleware(req, res);
        }
    }
}

pub fn apply_keep_alive_headers(req: &HttpRequest, res: &mut HttpResponse) {
    let close: &'static str = "close";
    if !req.keep_alive {
        res.headers
            .insert(HeaderName::Connection, close.to_string());
    }
}

pub fn apply_encoding(req: &HttpRequest, res: &mut HttpResponse) {
    if let Some(content_encoding) = req.headers.get("accept-encoding") {
        let matched = content_encoding
            .split(',')
            .find_map(|s| s.trim().parse::<Encoding>().ok());

        if let Some(encoding) = matched {
            match encoding {
                Encoding::Gzip => {
                    if res.body.is_empty() {
                        return;
                    }

                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                    match encoder.write_all(&res.body) {
                        Ok(_) => match encoder.finish() {
                            Ok(encoded) => {
                                res.body = encoded;
                                res.headers
                                    .insert(HeaderName::ContentEncoding, "gzip".to_string());
                            }
                            Err(e) => eprintln!("error encoding finish(): {}", e),
                        },
                        Err(e) => eprintln!("error encoding write_all: {}", e),
                    }
                }
            };
        }
    }
}

pub fn apply_content_length(_: &HttpRequest, res: &mut HttpResponse) {
    res.headers
        .insert(HeaderName::ContentLength, res.body.len().to_string());
}
