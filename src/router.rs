use std::collections::HashMap;

use crate::Args;
use crate::middlewares::Middlewares;
use crate::request::{HttpRequest, Method};
use crate::response::HttpResponse;

pub type Handler = fn(&HttpRequest, &Args) -> HttpResponse;

#[allow(dead_code)]
#[derive(Default)]
struct MethodMap {
    handlers: HashMap<Method, Handler>,
}

#[allow(unused)]
pub struct Router {
    routes: HashMap<(Method, &'static str), Handler>,
    inner: matchit::Router<MethodMap>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            inner: matchit::Router::<MethodMap>::new(),
        }
    }

    pub fn add(mut self, method: Method, path: &'static str, handler: Handler) -> Self {
        self.routes.insert((method, path), handler);
        self
    }

    pub fn route(mut self, method: Method, path: &'static str, handler: Handler) {
        if let Ok(map) = self.inner.at_mut(path) {
            map.value.handlers.insert(method, handler);
        } else {
            let mut map = MethodMap::default();
            map.handlers.insert(method, handler);
            self.inner.insert(path, map).expect("valid route pattern");
        }
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
