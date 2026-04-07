use crate::request::HttpRequest;
use crate::response::HttpResponse;

pub struct Middlewares;

impl Middlewares {
    pub fn run<'a>(req: &'a HttpRequest, res: &'a mut HttpResponse) {
        let Some(content_encoding) = req.headers.get("accept-encoding") else {
            println!("test1");
            return;
        };

        let Ok(_) = res.with_encoding(String::from(content_encoding)) else {
            println!("test2");
            return;
        };
    }
}
