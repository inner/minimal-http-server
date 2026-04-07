use crate::request::HttpRequest;
use crate::response::HttpResponse;

pub struct Middlewares;

impl Middlewares {
    pub fn run<'a>(req: &'a HttpRequest, res: &'a mut HttpResponse) {
        if let Some(content_encoding) = req.headers.get("accept-encoding") {
            let _ = res.with_encoding(String::from(content_encoding));
        }
    }
}
