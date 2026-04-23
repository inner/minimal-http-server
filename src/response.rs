use crate::http::StatusCode;
use crate::http::headers::{ALLOW, CONTENT_TYPE};
use crate::request::Method;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpResponse {
    pub status: StatusCode,
    pub body: Vec<u8>,
    pub headers: HashMap<&'static str, String>,
}

impl HttpResponse {
    pub fn ok() -> Self {
        Self {
            status: StatusCode::Ok,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn with_content_type(mut self, ct: &'static str) -> Self {
        self.headers.insert(CONTENT_TYPE, ct.to_string());
        self
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();
        response.extend_from_slice(
            format!("HTTP/1.1 {} {}", self.status as u16, self.status.reason()).as_bytes(),
        );
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
            status: StatusCode::NotFound,
            body: Vec::new(),
            headers: HashMap::new(),
        }
    }

    pub fn not_allowed(allowed: &[Method]) -> Self {
        let value = allowed
            .iter()
            .filter(|m| **m != Method::Unknown)
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let mut headers = HashMap::new();
        headers.insert(ALLOW, value);

        Self {
            status: StatusCode::MethodNotAllowed,
            body: Vec::new(),
            headers,
        }
    }

    pub fn created() -> Self {
        Self {
            status: StatusCode::Created,
            body: Vec::new(),
            headers: HashMap::new(),
        }
    }
}
