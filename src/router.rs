use crate::http::status::OK;
use crate::http_response::HttpResponse;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct Router;

#[allow(dead_code)]
impl Router {
    pub fn handle<'a>() -> HttpResponse<'a> {
        HttpResponse {
            status_line: OK,
            headers: HashMap::new(),
            body: "",
        }
    }
}
