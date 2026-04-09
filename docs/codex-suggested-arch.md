# Codex Suggested Architecture

## Goal

Restructure the server so these concerns are separate and predictable:

- request parsing errors vs application errors
- domain errors vs HTTP status codes
- route matching vs handler execution
- middleware behavior vs response finalization
- framework-owned headers vs user-set headers

The current codebase is small, but the boundaries are already working against that goal:

- handlers return concrete `HttpResponse` values directly, so they cannot cleanly express domain failures
- request parsing returns `std::io::Error`, which mixes transport failures with protocol failures
- the router decides the route and also runs middleware
- middleware mutates headers after routing, but there is no explicit policy for which headers are framework-controlled
- some status decisions are semantically wrong, for example mapping file I/O failure to `404 Not Found`

This document proposes a structure that keeps the project simple but gives it a real execution model.

## Current Pressure Points

These are the main places where responsibilities are blurred:

- [src/request.rs](../src/request.rs#L52): `HttpRequest::new` returns `Result<HttpRequest, io::Error>`, so malformed HTTP, unsupported versions, large payloads, and broken sockets all come back through the same error channel
- [src/router.rs](../src/router.rs#L26): the router both matches routes and runs middleware
- [src/middlewares.rs](../src/middlewares.rs#L12): middleware is a hardcoded post-processing pass rather than an application pipeline
- [src/response.rs](../src/response.rs#L7): responses use raw status lines instead of a typed status code
- [src/main.rs](../src/main.rs#L49): handlers collapse domain failures into ad hoc HTTP responses
- [src/files.rs](../src/files.rs#L8): file service errors are raw I/O errors, so callers have to guess whether they mean `403`, `404`, or `500`

## Recommended Layering

Use four layers:

1. HTTP layer
2. App layer
3. Service layer
4. Server runtime

### HTTP Layer

This layer owns wire-level concerns:

- request parsing
- response serialization
- header names and common value helpers
- protocol-specific parse errors
- status code representation

Suggested files:

- `src/http/mod.rs`
- `src/http/request.rs`
- `src/http/response.rs`
- `src/http/status.rs`
- `src/http/headers.rs`
- `src/http/error.rs`

### App Layer

This layer owns application dispatch:

- route matching
- route params
- middleware chain
- application error type
- conversion from app results into HTTP responses

Suggested files:

- `src/app/mod.rs`
- `src/app/router.rs`
- `src/app/middleware.rs`
- `src/app/context.rs`
- `src/app/error.rs`
- `src/app/state.rs`

### Service Layer

This layer owns reusable domain behavior and should not know about HTTP:

- file read/write logic
- file validation
- service-specific error types

Suggested files:

- `src/services/mod.rs`
- `src/services/files.rs`

### Server Runtime

This layer owns:

- `TcpListener`
- connection lifecycle
- keep-alive loop
- request read / app dispatch / response write

Suggested file:

- `src/server.rs`

## Core Contracts

The key change is to stop making handlers build final responses directly for every branch.

Use these contracts:

```rust
type AppResult<T> = Result<T, AppError>;
type Handler = fn(&RequestContext, &AppState) -> AppResult<Response>;
```

```rust
pub trait IntoResponse {
    fn into_response(self) -> Response;
}
```

The flow becomes:

1. Parse request into `Request`
2. Router finds a handler or returns a routing error
3. Handler returns `Result<Response, AppError>`
4. Middleware wraps the handler and can short-circuit if needed
5. `AppError` converts into `Response`
6. Response finalizer adds framework-owned headers
7. Response writes itself to the socket

That gives one place for HTTP semantics instead of spreading them across handlers.

## Typed Status Codes

Replace raw status-line strings with a typed enum.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    PayloadTooLarge = 413,
    RequestHeaderFieldsTooLarge = 431,
    InternalServerError = 500,
    HttpVersionNotSupported = 505,
}

impl StatusCode {
    pub fn reason(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Created => "Created",
            Self::BadRequest => "Bad Request",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::PayloadTooLarge => "Payload Too Large",
            Self::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            Self::InternalServerError => "Internal Server Error",
            Self::HttpVersionNotSupported => "HTTP Version Not Supported",
        }
    }
}
```

Then `Response` stores `status: StatusCode` rather than `status_line: &'static str`.

## Request Parse Errors vs Application Errors

Do not use `std::io::Error` as the public error type for request parsing.

Parsing should return a protocol-level error enum:

```rust
pub enum RequestParseError {
    ConnectionClosed,
    InvalidRequestLine,
    InvalidHeader,
    HeadersTooLarge,
    BodyTooLarge,
    UnsupportedMethod,
    UnsupportedVersion,
    Io(std::io::Error),
}
```

That lets the server runtime map errors correctly:

- `ConnectionClosed` -> break loop without a response
- `InvalidRequestLine` -> `400 Bad Request`
- `HeadersTooLarge` -> `431 Request Header Fields Too Large`
- `BodyTooLarge` -> `413 Payload Too Large`
- `UnsupportedVersion` -> `505 HTTP Version Not Supported`
- `Io(_)` -> close connection, optionally log

This is cleaner than the current `handle_connection` logic in [src/main.rs](../src/main.rs#L122), where only EOF is handled specially and all other parse failures bubble out as generic errors.

## AppError as the HTTP Boundary

Introduce a single app-facing error enum and make it responsible for HTTP mapping.

```rust
pub enum AppError {
    BadRequest(&'static str),
    Forbidden(&'static str),
    NotFound,
    MethodNotAllowed { allow: Vec<Method> },
    Internal(&'static str),
}
```

Implement `IntoResponse` for it:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(msg) => Response::new(StatusCode::BadRequest)
                .with_text_body(msg),
            Self::Forbidden(msg) => Response::new(StatusCode::Forbidden)
                .with_text_body(msg),
            Self::NotFound => Response::new(StatusCode::NotFound),
            Self::MethodNotAllowed { allow } => Response::new(StatusCode::MethodNotAllowed)
                .with_header("Allow", allow.into_iter().map(Method::as_str).collect::<Vec<_>>().join(", ")),
            Self::Internal(msg) => Response::new(StatusCode::InternalServerError)
                .with_text_body(msg),
        }
    }
}
```

The point is not to put every possible failure directly into handlers. The point is to force every application error through one deliberate mapping.

## Service Errors Stay Below HTTP

Services should return service-specific error enums, not HTTP responses.

For example:

```rust
pub enum FileError {
    InvalidPath,
    NotFound,
    Io(std::io::Error),
}
```

Then handlers map service errors to app errors:

```rust
fn get_file(ctx: &RequestContext, state: &AppState) -> AppResult<Response> {
    let dir = state.directory.as_deref()
        .ok_or(AppError::Internal("file storage not configured"))?;

    let file_name = ctx.param("file").ok_or(AppError::NotFound)?;

    let bytes = state.files.read(dir, file_name).map_err(|err| match err {
        FileError::InvalidPath => AppError::Forbidden("invalid path"),
        FileError::NotFound => AppError::NotFound,
        FileError::Io(_) => AppError::Internal("file read failed"),
    })?;

    Ok(Response::new(StatusCode::Ok)
        .with_content_type("application/octet-stream")
        .with_body(bytes))
}
```

This is the missing separation in the current handlers in [src/main.rs](../src/main.rs#L49).

## Router Behavior

The router should return one of three outcomes:

- matched route
- path exists but method is not allowed
- no route for that path

That is how you get proper `404` vs `405`.

Suggested shape:

```rust
pub enum RouteMatch<'a> {
    Matched { handler: &'a Handler, params: Params },
    MethodNotAllowed { allow: Vec<Method> },
    NotFound,
}
```

The current router in [src/router.rs](../src/router.rs#L41) only distinguishes “found exact method+prefix” from “404”. That loses the information needed to return `405 Method Not Allowed` with an `Allow` header.

For this codebase, a linear scan router is acceptable and simpler than a prefix `HashMap`. It also makes route params easier to add later.

Example registration:

```rust
Router::new()
    .route(Method::Get, "/", handlers::root)
    .route(Method::Get, "/echo/:value", handlers::echo)
    .route(Method::Get, "/user-agent", handlers::user_agent)
    .route(Method::Get, "/files/:name", handlers::get_file)
    .route(Method::Post, "/files/:name", handlers::put_file)
```

That removes the need for handlers to re-parse prefixes from `req.path`.

## Request Context and App State

Do not pass `clap::Args` directly into handlers.

Use:

```rust
pub struct AppState {
    pub directory: Option<String>,
    pub files: FileService,
}

pub struct RequestContext<'a> {
    pub request: &'a Request,
    pub params: Params,
}
```

This does two things:

- decouples CLI parsing from application logic
- gives handlers route params without making them re-slice the URL

`Args` should stay in `main` and be converted once into `AppState`.

## Middleware Model

The current `Middlewares::run(req, res)` approach in [src/middlewares.rs](../src/middlewares.rs#L12) is too limited for growth.

Use an actual middleware chain:

```rust
pub trait Middleware {
    fn handle(&self, req: RequestContext, next: Next<'_>) -> AppResult<Response>;
}
```

With:

```rust
pub struct Next<'a> {
    // remaining middleware + endpoint
}
```

This gives you both:

- pre-handler logic
- post-handler logic

That matters because some middleware wants to inspect the request before dispatch, and other middleware wants to inspect or modify the response after the handler returns.

Examples that fit this codebase:

- request logging
- request ID insertion
- content negotiation
- compression
- optional auth
- panic/error boundary if you later want one

## Header Ownership Policy

This is the most important rule if you want headers to stay sane.

Split headers into three groups:

### Framework-Owned Headers

These should be set in one place only, during response finalization:

- `Content-Length`
- `Connection`
- `Date` if you add it
- `Content-Encoding` if the framework performed compression

Handlers should not set these directly.

### App-Owned Headers

Handlers and middleware may set these:

- `Content-Type`
- `Cache-Control`
- `ETag`
- custom app headers

### Defaultable Headers

These may be set by the framework only if absent:

- `Content-Type`, if you want a default like `application/octet-stream`

This avoids fights between middleware and handlers.

## Response Finalization

Do not compute final wire headers in handlers, and do not spread that logic between `main` and middleware.

Have one finalization step:

```rust
impl Response {
    pub fn finalize(mut self, req: &Request) -> Self {
        if !self.headers.contains_key("Content-Length") {
            self.headers.insert("Content-Length", self.body.len().to_string());
        }

        if req.keep_alive() {
            self.headers.entry("Connection").or_insert_with(|| "keep-alive".into());
        } else {
            self.headers.insert("Connection", "close".into());
        }

        self
    }
}
```

The exact details may change, but the principle should not:

- middleware may still mutate the body
- finalization runs after middleware
- finalization is the only place that computes transport headers

Right now, `Content-Length` is added in [src/middlewares.rs](../src/middlewares.rs#L47) and `Connection` is added in [src/main.rs](../src/main.rs#L137). Those belong together.

## Compression Middleware

Compression is a good middleware, but only if it behaves like a real transformation with clear ownership.

Rules:

- negotiate from `Accept-Encoding`
- compress only when there is a body
- set `Content-Encoding` only if compression succeeded
- run before response finalization so `Content-Length` reflects the compressed body
- do not silently swallow compression failures into stderr if they should change the response path

For this codebase, a pragmatic approach is:

- if compression fails unexpectedly, return `AppError::Internal("response compression failed")`

That is stricter than the current code in [src/middlewares.rs](../src/middlewares.rs#L30), which logs and keeps going.

## Response Type

Make `Response` a builder around typed state rather than a bag of raw strings.

Suggested shape:

```rust
pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}
```

Methods:

- `Response::new(status)`
- `with_header(name, value)`
- `with_content_type(value)`
- `with_body(bytes)`
- `with_text_body(text)`
- `finalize(&Request)`
- `write_to(&mut impl Write)`

Prefer `write_to` over building an intermediate `Vec<u8>` in `as_bytes()`. That avoids one extra allocation per response and keeps serialization responsibility inside the response type.

## Suggested Execution Flow

The connection loop should look conceptually like this:

```rust
loop {
    let request = match Request::read_from(&mut reader) {
        Ok(request) => request,
        Err(RequestParseError::ConnectionClosed) => break,
        Err(parse_err) => {
            let response = parse_err.into_response().finalize_for_close();
            response.write_to(&mut stream)?;
            break;
        }
    };

    let response = app.handle(request).unwrap_or_else(IntoResponse::into_response);
    let response = response.finalize(&request);
    response.write_to(&mut stream)?;

    if !request.keep_alive() {
        break;
    }
}
```

The important part is that parse errors can also become valid HTTP responses when appropriate.

## Minimal Module Layout

A pragmatic module layout for this repo:

```text
src/
  main.rs
  server.rs
  http/
    mod.rs
    error.rs
    headers.rs
    request.rs
    response.rs
    status.rs
  app/
    mod.rs
    context.rs
    error.rs
    middleware.rs
    router.rs
    state.rs
  handlers/
    mod.rs
    echo.rs
    files.rs
    root.rs
    user_agent.rs
  services/
    mod.rs
    files.rs
  threadpool.rs
```

This is enough structure to clarify responsibilities without turning the project into a framework.

## Recommended Migration Order

Implement in this order:

1. Introduce `StatusCode` and change `Response` to store typed status.
2. Add `AppError` plus `IntoResponse`.
3. Change handlers to return `Result<Response, AppError>`.
4. Introduce `RequestParseError` and stop exposing raw `io::Error` from parsing.
5. Teach the router to distinguish `404` from `405`.
6. Move file logic behind a `FileService` and `FileError`.
7. Replace the static middleware pass with a real middleware chain.
8. Add response finalization for `Content-Length` and `Connection`.
9. Move handlers out of `main.rs`.

This order keeps the server working while improving the boundary one layer at a time.

## Tests Worth Adding

Once the structure changes, add tests for:

- malformed request line returns `400`
- oversized headers return `431`
- oversized body returns `413`
- unsupported HTTP version returns `505`
- unknown path returns `404`
- known path with wrong method returns `405` and correct `Allow`
- gzip response sets `Content-Encoding` and correct `Content-Length`
- handler-set `Content-Type` is preserved
- framework still injects `Connection` and `Content-Length`
- invalid file path returns `403`
- missing file returns `404`
- unexpected file I/O returns `500`

## Summary

The recommended architecture is not a rewrite for its own sake. It is a boundary cleanup:

- HTTP parsing gets its own typed error model
- handlers return domain results instead of hand-building every HTTP branch
- service errors stay below HTTP
- `AppError` becomes the single application-to-HTTP mapping point
- middleware becomes a real request/response pipeline
- response finalization owns transport headers

That gives you proper error handling, correct status code behavior, and a clean rule for “which code is allowed to set which headers”.
