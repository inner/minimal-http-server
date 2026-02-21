use crate::http;
use crate::http::headers::{CONTENT_LENGTH, CONTENT_TYPE};
use crate::http::status::OK;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpResponse {
    pub status_line: &'static str,
    pub body: Vec<u8>,
    pub headers: HashMap<&'static str, String>,
}

impl HttpResponse {
    pub fn ok() -> Self {
        Self {
            status_line: OK,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.headers.insert(CONTENT_LENGTH, body.len().to_string());
        self.body = body;
        self
    }

    pub fn with_content_type(mut self, ct: &'static str) -> Self {
        self.headers.insert(CONTENT_TYPE, ct.to_string());
        self
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();
        response.extend_from_slice(self.status_line.as_bytes());
        response.extend_from_slice(b"\r\n");

        for (k, v) in &self.headers {
            response.extend_from_slice(k.as_bytes());
            response.extend_from_slice(b": ");
            response.extend_from_slice(v.as_bytes());
            response.extend_from_slice(b"\r\n");
        }

        response.extend_from_slice(b"\r\n");

        if !self.body.is_empty() {
            response.extend(&self.body);
        }

        response
    }

    pub fn not_found() -> Self {
        Self {
            status_line: http::status::NOT_FOUND,
            body: Vec::new(),
            headers: HashMap::new(),
        }
    }

    pub fn created() -> Self {
        Self {
            status_line: http::status::CREATED,
            body: Vec::new(),
            headers: HashMap::new(),
        }
    }
}
