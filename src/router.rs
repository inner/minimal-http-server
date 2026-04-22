use std::collections::HashMap;

use crate::Args;
use crate::request::{HttpRequest, Method};
use crate::response::HttpResponse;

pub type Handler = fn(&HttpRequest, &Args, &matchit::Params) -> HttpResponse;

pub enum Match<'r, 'p> {
    Found(&'r Handler, matchit::Params<'r, 'p>),
    MethodNotAllowed(Vec<Method>),
    NotFound,
}

#[derive(Default)]
struct MethodMap {
    handlers: HashMap<Method, Handler>,
}

pub struct Router {
    inner: matchit::Router<MethodMap>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            inner: matchit::Router::<MethodMap>::new(),
        }
    }

    pub fn add(&mut self, method: Method, path: &str, handler: Handler) {
        if let Ok(map) = self.inner.at_mut(path) {
            map.value.handlers.insert(method, handler);
        } else {
            let mut map = MethodMap::default();
            map.handlers.insert(method, handler);
            let _ = self.inner.insert(path, map);
        }
    }

    pub fn route(mut self, method: Method, path: &str, handler: Handler) -> Self {
        if let Ok(map) = self.inner.at_mut(path) {
            map.value.handlers.insert(method, handler);
        } else {
            let mut map = MethodMap::default();
            map.handlers.insert(method, handler);
            let _ = self.inner.insert(path, map);
        }

        self
    }

    pub fn find<'r, 'p>(&'r self, path: &'p str, method: &Method) -> Match<'r, 'p> {
        match self.inner.at(path) {
            Err(_) => Match::NotFound,
            Ok(map) => match map.value.handlers.get(method) {
                Some(handler) => Match::Found(handler, map.params),
                None => Match::MethodNotAllowed(map.value.handlers.keys().copied().collect()),
            },
        }
    }
}
