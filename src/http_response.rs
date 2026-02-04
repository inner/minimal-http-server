use std::collections::HashMap;

pub struct HttpResponse {
    pub http_line: String,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl HttpResponse {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();
        response.extend_from_slice(format!("{}\r\n", &self.http_line).as_bytes());

        for (k, v) in &self.headers {
            response.extend_from_slice(format!("{k}: {v}\r\n").as_bytes());
        }

        response.extend_from_slice("\r\n".as_bytes());

        if !self.body.is_empty() {
            response.extend_from_slice(self.body.as_bytes());
        }

        response
    }
}
