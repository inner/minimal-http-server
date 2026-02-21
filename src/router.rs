use std::collections::HashMap;
use std::sync::Arc;

use crate::http_request::{HttpRequest, Method};
use crate::http_response::HttpResponse;

pub type Handler = fn(&HttpRequest, &HashMap<String, String>) -> HttpResponse;

pub struct Router {
    routes: HashMap<(Method, &'static str), Handler>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add(mut self, method: Method, path: &'static str, handler: Handler) -> Self {
        self.routes.insert((method, path), handler);
        self
    }

    pub fn build_arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn handle(&self, req: &HttpRequest, args: &HashMap<String, String>) -> HttpResponse {
        let prefix = if req.path == "/" {
            "/"
        } else {
            req.path.split('/').nth(1).unwrap_or("")
        };

        let key = if prefix == "/" {
            "/".to_string()
        } else {
            format!("/{prefix}")
        };

        if let Some(handler) = self.routes.get(&(req.method, &key)) {
            handler(req, args)
        } else {
            HttpResponse::not_found()
        }
    }
}
