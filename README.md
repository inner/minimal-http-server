# Rust HTTP Server

A small HTTP server written in Rust on top of `TcpListener`/`TcpStream`, originally built as part of the Codecrafters "Build Your Own HTTP Server" challenge.

It is intentionally small in scope. The goal is not to compete with production web frameworks, but to demonstrate hands-on Rust work in networking, parsing, error handling, and systems-oriented design.

## Why This Project Exists

This project explores the fundamentals of building an HTTP server in Rust from raw TCP I/O up through routing and response generation.

It was built to work directly with:

- TCP sockets
- HTTP request parsing
- routing and method dispatch
- response encoding
- filesystem I/O
- concurrency with a thread pool
- Rust ownership, borrowing, and `Result`-based error handling

## What It Currently Supports

The current server implements a compact subset of HTTP behavior:

- `GET /` returns `200 OK`
- `GET /echo/{echo}` returns the route parameter as the response body
- `GET /user-agent` returns the incoming `User-Agent` header
- `GET /files/{file}` reads a file from a configured directory
- `POST /files/{file}` writes the request body into a file in a configured directory
- `405 Method Not Allowed` with an `Allow` header when the path exists for other methods
- `Content-Length` response header generation
- `Connection: close` handling
- gzip response compression when the client advertises `Accept-Encoding: gzip`
- multiple simultaneous clients via a simple thread pool

## Design Notes

This code deliberately keeps the architecture straightforward:

- request parsing is done manually from a buffered TCP stream
- routing is handled by `matchit`
- route handlers are plain function pointers, which keeps dispatch simple for this project
- middleware is applied after the route handler to add cross-cutting response behavior such as compression and content length

That simplicity is intentional. For a learning project, I preferred code that is easy to trace end-to-end over a more abstract framework-style design.

## Limits

This is a learning project, not a production-ready HTTP implementation.

Some examples of what it does not aim to cover yet:

- full HTTP specification compliance
- async I/O
- chunked transfer encoding
- streaming responses
- TLS
- advanced error handling and structured logging
- a comprehensive automated test suite

## Running It

Prerequisites:

- Rust `1.91+`

Start the server:

```sh
cargo run -- --directory ./tmp
```

The server listens on `0.0.0.0:4221`.

Example requests:

```sh
curl -i http://127.0.0.1:4221/
curl -i http://127.0.0.1:4221/echo/hello
curl -i http://127.0.0.1:4221/user-agent
curl -i --data 'hello' http://127.0.0.1:4221/files/demo.txt
curl -i http://127.0.0.1:4221/files/demo.txt
```

## Repository Structure

- [src/main.rs](src/main.rs) wires together the TCP listener, router, and request handling loop
- [src/request.rs](src/request.rs) parses incoming HTTP requests
- [src/router.rs](src/router.rs) resolves paths and methods to handlers
- [src/response.rs](src/response.rs) builds HTTP responses
- [src/middlewares.rs](src/middlewares.rs) applies response middleware such as compression and content length
- [src/files.rs](src/files.rs) contains file read/write logic with path safety checks
- [src/threadpool.rs](src/threadpool.rs) provides the worker pool used for concurrent connections

## Authorship And AI Usage

The code in this repository was written by me.

AI tools were used in a supporting role, closer to searchable reference material or an interactive technical assistant than a code generator. I used AI to sanity-check ideas, compare design options, and clarify protocol or Rust-specific details, but not to generate the implementation wholesale.

## What I Would Improve Next

If I continue developing this project, the next areas I would focus on are:

- stronger protocol validation and error responses
- targeted tests for parser and routing edge cases
- cleaner response writing without intermediate allocations
- more explicit application state instead of passing CLI args into handlers
- clearer separation between protocol concerns and application behavior
