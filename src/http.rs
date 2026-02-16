pub mod status {
    pub const OK: &str = "HTTP/1.1 200 OK";
    pub const NOT_FOUND: &str = "HTTP/1.1 404 Not Found";
    pub const CREATED: &str = "HTTP/1.1 201 Created";
}

pub mod headers {
    pub const CONTENT_LENGTH: &str = "Content-Length";
    pub const CONTENT_TYPE: &str = "Content-Type";
    pub const TEXT_PLAIN: &str = "text/plain";
    pub const OCTET_STREAM: &str = "application/octet-stream";
}
