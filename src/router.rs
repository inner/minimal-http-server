use std::collections::HashMap;

use crate::Args;
use crate::middlewares::Middlewares;
use crate::request::{HttpRequest, Method};
use crate::response::HttpResponse;

pub type Handler = fn(&HttpRequest, &Args) -> HttpResponse;

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

    pub fn handle(&self, req: &HttpRequest, args: &Args) -> HttpResponse {
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

        let mut response: HttpResponse;

        if let Some(handler) = self.routes.get(&(req.method, &key)) {
            response = handler(req, args);
        } else {
            response = HttpResponse::not_found();
        }

        Middlewares::run(req, &mut response);
        response
    }
}
