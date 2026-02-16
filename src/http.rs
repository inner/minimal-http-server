pub mod status {
    pub const OK: &'static str = "HTTP/1.1 200 OK";
    pub const NOT_FOUND: &'static str = "HTTP/1.1 404 Not Found";
    pub const CREATED: &'static str = "HTTP/1.1 201 Created";
}

pub mod headers {
    pub const CONTENT_LENGTH: &'static str = "Content-Length";
    pub const CONTENT_TYPE: &'static str = "Content-Type";
    pub const TEXT_PLAIN: &'static str = "text/plain";
    pub const OCTET_STREAM: &'static str = "application/octet-stream";
}
