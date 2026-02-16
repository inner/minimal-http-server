use crate::http;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpResponse<'a> {
    pub status_line: &'static str,
    pub body: &'a str,
    pub headers: HashMap<&'static str, Cow<'static, str>>,
}

impl<'a> HttpResponse<'a> {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();
        response.extend_from_slice(self.status_line.as_bytes());
        response.extend_from_slice("\r\n".as_bytes());

        for (k, v) in &self.headers {
            response.extend_from_slice(k.as_bytes());
            response.extend_from_slice(": ".as_bytes());
            response.extend_from_slice(v.as_bytes());
            response.extend_from_slice("\r\n".as_bytes());
        }

        response.extend_from_slice("\r\n".as_bytes());

        if !self.body.is_empty() {
            response.extend_from_slice(self.body.as_bytes());
        }

        response
    }

    pub fn not_found() -> Self {
        HttpResponse {
            status_line: http::status::NOT_FOUND,
            body: "",
            headers: HashMap::new(),
        }
    }

    pub fn created() -> Self {
        HttpResponse {
            status_line: http::status::CREATED,
            body: "",
            headers: HashMap::new(),
        }
    }
}
