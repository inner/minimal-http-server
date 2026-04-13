use std::collections::HashMap;

use crate::Args;
use crate::request::{HttpRequest, Method};
use crate::response::HttpResponse;

pub type Handler = fn(&HttpRequest, &Args, &matchit::Params) -> HttpResponse;

pub enum Match<'r, 'p> {
    Found(&'r Handler, matchit::Params<'r, 'p>),
    MethodNotAllowed,
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

    pub fn route(mut self, method: Method, path: &'static str, handler: Handler) -> Self {
        if let Ok(map) = self.inner.at_mut(path) {
            map.value.handlers.insert(method, handler);
        } else {
            let mut map = MethodMap::default();
            map.handlers.insert(method, handler);
            self.inner.insert(path, map).expect("valid route pattern");
        }

        self
    }

    pub fn find<'r, 'p>(&'r self, path: &'p str, method: &Method) -> Match<'r, 'p> {
        match self.inner.at(path) {
            Err(_) => Match::NotFound,
            Ok(map) => match map.value.handlers.get(method) {
                Some(handler) => Match::Found(handler, map.params),
                None => Match::MethodNotAllowed,
            },
        }
    }
}
