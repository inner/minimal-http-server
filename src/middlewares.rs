use crate::http::compression::Encoding;
use crate::http::headers::{CONNECTION, CONTENT_ENCODING, CONTENT_LENGTH};
use crate::request::HttpRequest;
use crate::response::HttpResponse;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

pub struct Middlewares;

impl Middlewares {
    pub fn run(req: &HttpRequest, res: &mut HttpResponse) {
        Self::apply_encoding(req, res);
        Self::apply_content_length(res);
        Self::apply_keep_alive_headers(req, res);
    }

    fn apply_keep_alive_headers(req: &HttpRequest, res: &mut HttpResponse) {
        let close: &'static str = "close";
        if !req.keep_alive {
            res.headers.insert(CONNECTION, close.to_string());
        }
    }

    fn apply_encoding(req: &HttpRequest, res: &mut HttpResponse) {
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
                                    res.headers.insert(CONTENT_ENCODING, "gzip".to_string());
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

    fn apply_content_length(res: &mut HttpResponse) {
        res.headers
            .insert(CONTENT_LENGTH, res.body.len().to_string());
    }
}
