# Suggested Architecture: Error Handling, Status Codes & Middleware

## Current Pain Points

1. **No error semantics in handlers** — `handle_read_body` returns `not_found()` on an I/O write failure, which is semantically wrong (should be 500). Handlers have no way to express "this was my fault" vs "that resource doesn't exist."

2. **`status_line` is a raw `&'static str`** — `"HTTP/1.1 404 Not Found"` is a magic string you can't introspect or match on. Status codes should be a typed enum.

3. **Middleware is a monolith** — `Middlewares::run` calls two hardcoded functions. You can't add/remove middlewares at registration time or compose them.

4. **Handlers are coupled to `Args`** — Handler signature is `fn(&HttpRequest, &Args) -> HttpResponse`. Any new app-level config (not just directory) means changing every handler's signature.

---

## Recommended Architecture

### 1. Typed Status Codes

```rust
// http/status.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
}

impl StatusCode {
    pub fn reason(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Created => "Created",
            Self::BadRequest => "Bad Request",
            Self::NotFound => "Not Found",
            Self::InternalServerError => "Internal Server Error",
        }
    }
}
```

`HttpResponse::as_bytes()` then writes `format!("HTTP/1.1 {} {}\r\n", status as u16, status.reason())` — no more magic strings.

---

### 2. `AppError` That Converts to a Response

```rust
// error.rs
pub enum AppError {
    NotFound,
    BadRequest(&'static str),
    Internal(Box<dyn std::error::Error>),
}

impl AppError {
    pub fn into_response(self) -> HttpResponse {
        match self {
            Self::NotFound => HttpResponse::new(StatusCode::NotFound),
            Self::BadRequest(msg) => HttpResponse::new(StatusCode::BadRequest)
                .with_body(msg.as_bytes().to_vec()),
            Self::Internal(e) => {
                eprintln!("internal: {e}");
                HttpResponse::new(StatusCode::InternalServerError)
            }
        }
    }
}
```

Now `handle_read_body` becomes:

```rust
fn handle_read_body(req: &HttpRequest, state: &AppState) -> Result<HttpResponse, AppError> {
    let file_name = req.path.strip_prefix("/files/").ok_or(AppError::NotFound)?;
    let dir = state.directory.as_deref().ok_or(AppError::NotFound)?;

    FileManager::create(Path::new(dir), file_name, &req.body)
        .map_err(|e| AppError::Internal(e.into()))?;  // ← correct 500, not 404

    Ok(HttpResponse::new(StatusCode::Created))
}
```

---

### 3. Handler Type + Router Error Conversion

```rust
// router.rs
pub type Handler = fn(&HttpRequest, &AppState) -> Result<HttpResponse, AppError>;

// in Router::handle():
let mut response = match handler(req, state) {
    Ok(r) => r,
    Err(e) => e.into_response(),
};
middleware_chain.run(req, &mut response);
response
```

---

### 4. Composable Middleware Chain

```rust
// middlewares.rs
pub type Middleware = fn(&HttpRequest, &mut HttpResponse);

pub struct MiddlewareChain {
    middlewares: Vec<Middleware>,
}

impl MiddlewareChain {
    pub fn new() -> Self { Self { middlewares: vec![] } }

    pub fn add(mut self, m: Middleware) -> Self {
        self.middlewares.push(m);
        self
    }

    pub fn run(&self, req: &HttpRequest, res: &mut HttpResponse) {
        for m in &self.middlewares {
            m(req, res);
        }
    }
}
```

Registration in `main`:

```rust
let chain = MiddlewareChain::new()
    .add(middleware_compression)
    .add(middleware_content_length)
    .add(middleware_connection);
```

Each middleware is a standalone `fn` — easy to test, add, or reorder.

---

### 5. `AppState` Instead of `Args`

```rust
pub struct AppState {
    pub directory: Option<String>,
}
```

`Args` is a CLI concern. `AppState` is the runtime config handlers care about. You convert once in `main` after parsing, and nothing downstream imports `clap`.

---

## Summary of Structural Changes

| Current | Proposed |
|---|---|
| `status_line: &'static str` | `status: StatusCode` (typed enum) |
| `fn(...) -> HttpResponse` | `fn(...) -> Result<HttpResponse, AppError>` |
| `Middlewares::run` (hardcoded 2 fns) | `MiddlewareChain::add()` (composable) |
| Handler takes `&Args` | Handler takes `&AppState` |
| 404 on all errors | 404 / 400 / 500 based on error variant |

This keeps the existing prefix-matching router and threading model intact — it's purely a layering change, not a rewrite.
