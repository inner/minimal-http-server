# TODO

## Bugs

- [x] **Path traversal vulnerability** (`files.rs:13,20`) — Fixed: `FileManager::read` now uses `fs::canonicalize` to resolve the real path and verifies it starts with the base directory, catching both `..` traversal and symlinks. `FileManager::create` rejects any path segment equal to `..` (canonicalize can't be used since the file doesn't exist yet). Note: a production-grade solution would use `openat` with directory file descriptors to eliminate the TOCTOU race condition entirely, but this requires `unsafe` or a crate like `cap-std`.
- [x] **BufReader lost on persistent connections** (`main.rs:134`, `request.rs:34`) — Fixed: `BufReader` is now created once outside the loop in `handle_connection`, preserving buffered bytes across requests. `HttpRequest::new` was refactored to accept `&mut R where R: BufRead` instead of owning or creating the reader, decoupling the parser from the transport and making it testable with any `BufRead` implementation. EOF is handled explicitly by breaking the loop on `ErrorKind::UnexpectedEof`.
- [x] **Deflate encoding declared but not applied** (`response.rs:52-65`, `http.rs:20-22`) — Fixed: removed the `Deflate` variant from the `Encoding` enum. `with_encoding` now only recognises `gzip`; any other value in `Accept-Encoding` is ignored, so `Content-Encoding` is never set to an encoding the server doesn't actually apply.
- [x] **`.unwrap()` panics in `with_gzip_body`** (`response.rs:36-37`) — Fixed: `with_gzip_body` now returns `Result<Self, Error>` and propagates errors with `?`. The call site handles the error gracefully by returning `HttpResponse::not_found()`.

## Code Smells

- [ ] **Handlers re-parse paths** (`main.rs:27,54,69`) — `handle_echo`, `handle_return_file`, and `handle_read_body` all call `strip_prefix` on the path the router already split. The router should pass the remainder to handlers directly.
- [x] **Duplicate `stream.write_all`** (`main.rs:144-152`) — Fixed: renamed `close_connection` field to `keep_alive` on `HttpRequest` (`request.rs:91`), making intent clearer; `write_all` appears once unconditionally, header insertion and `break` are each gated on `!request.keep_alive`.
- [x] **`find(":")` should be `find(':')`** (`request.rs:69`) — Fixed: changed to a `char` literal.
- [x] **`if self.body.len() > 0`** (`response.rs:35`) — Fixed: changed to `!self.body.is_empty()`.
- [x] **Verbose `close_connection` assignment** (`request.rs:91`) — Fixed: field renamed to `keep_alive`, collapsed to `let keep_alive = !headers.get("connection").is_some_and(|v| v == "close");`.

## Design

- [x] **`Box::leak` instead of `Arc`** (`main.rs:99-109`) — Fixed: both `args` and `router` are now wrapped in `Arc`, cloned per thread with `Arc::clone`. No memory leaks.
- [x] **`workers` is `pub` on `ThreadPool`** (`threadpool.rs:42`) — Fixed: `workers` is now private.
- [x] **HTTP version not parsed** (`request.rs:24-39,113-114`) — Fixed: added `Version` enum with `Http10`/`Http11`/`Unknown` variants. `keep_alive` now requires both HTTP/1.1 and the absence of `Connection: close`, so HTTP/1.0 connections correctly default to close.

## Allocations

- [ ] **`format!("/{prefix}")` on every request** (`router.rs:31-35`) — allocates a `String` per request just for a HashMap lookup. Replace the HashMap with a `Vec<(Method, &'static str, Handler)>` and linear scan to compare `&str` directly, zero allocation per lookup.
- [ ] **`with_encoding` takes `String` instead of `&str`** (`response.rs:52`, `main.rs:39`) — `String::from(content_encoding)` allocates at the call site just to pass ownership in; the method only calls `.split(',')`. Change the signature to `&str`.
- [ ] **`&'static str` values stored as `String` in headers** (`response.rs:47-49,58-62`) — `ct.to_string()` and encoding names `"gzip"`/`"deflate"` are heap-allocated despite being static. Change the header map value type to `Cow<'static, str>` so static values avoid allocation.
- [ ] **`as_bytes()` builds an intermediate `Vec<u8>`** (`response.rs:68`) — every response allocates a full buffer before writing to the stream. Replace with `write_to(&self, w: &mut impl Write)` to write headers and body directly.
- [x] **Args collected to `Vec<String>` for `.chunks(2)`** (`main.rs:99`) — Fixed: replaced entirely by `clap` with `#[derive(Parser)]`. Args are parsed via `Args::parse()`, handlers access `args.directory.as_deref()` directly.
