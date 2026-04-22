use crate::request::Method;
use crate::router::{Handler, Router};

#[allow(unused)]
pub struct App {
    router: Router,
}

#[allow(unused)]
impl App {
    pub fn new() -> Self {
        App {
            router: Router::new(),
        }
    }

    pub fn with_route(mut self, method: Method, path: &str, handler: Handler) -> Self {
        self.router.add(method, path, handler);
        self
    }

    pub fn run(self) {
        //
    }
}
