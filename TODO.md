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

## New Findings (2026-04-03)

### Bugs

- [x] **Path traversal via absolute path in `FileManager::create`** (`files.rs:17`) — `path.join(file_name)` replaces the base entirely when `file_name` starts with `/` (e.g. `POST /files//etc/passwd` → `file_name = "/etc/passwd"` → writes to `/etc/passwd`). The `..` segment check doesn't catch this. `read()` is safe because it canonicalizes and checks `starts_with(path)`; `create()` needs an equivalent guard (reject if `file_name` starts with `/`, or canonicalize the parent and verify). Fixed: component check rejects any non-`Normal` segment including `RootDir`.
- [x] **Gzip always applied even when no encoding was negotiated** (`main.rs:39-46`, `response.rs:52-65`) — Fixed: encoding moved to `Middlewares::run` middleware, which only calls `with_encoding` when `accept-encoding` header is present. `with_gzip_body` removed entirely.
- [x] **Empty body skips gzip but `Content-Encoding: gzip` is already set** (`response.rs:33-45`) — Fixed: `Content-Encoding` header is now set inside the `if !self.body.is_empty()` guard in `with_encoding`.
- [ ] **No size limit on the request line** (`request.rs:52-56`) — `MAX_HEADER_SIZE` (8 KB) guards headers, but the request line (method + path + version) has no limit. A client can send an arbitrarily large path and the server reads it all into a `String` before any check. Add a limit (e.g. 8 KB) before `read_line` on the first line.
- [x] **`thread.join().unwrap()` panics if a worker panicked** (`threadpool.rs:95`) — Fixed: changed to `if thread.join().is_err()` with an `eprintln!`, avoiding double-panic on shutdown.

### Correctness

- [ ] **Returns 404 for unknown HTTP methods; should be 405** (`router.rs:38-42`) — HTTP spec requires 405 Method Not Allowed with an `Allow` header listing valid methods for the path.
- [ ] **`Version::Unknown` silently accepted** (`request.rs:113`) — unknown version strings produce `Version::Unknown`, which is treated as HTTP/1.0. Server should reject with 505 HTTP Version Not Supported.
- [ ] **Hardcoded loopback bind address** (`main.rs:100`) — `127.0.0.1:4221` only accepts localhost connections. Should bind `0.0.0.0:4221` (or make it configurable via `--host`/`--port` args).
- [x] **`create_dir` fails if parent doesn't exist** (`files.rs:14`) — if `--directory /tmp/a/b` is given and `/tmp/a` doesn't exist, `create_dir` errors, which surfaces as a 404. Consider `create_dir_all` or a clearer error response. Fixed: `create_dir` logic removed; `canonicalize(path)` now fails early if the directory doesn't exist, which is correct behaviour.

### Nits

- [x] **`Worker::new` returns `Result` but can't fail** (`threadpool.rs:14`) — Fixed: return type changed to `Self`; `?` removed from call site. `ThreadPool::new` also simplified to return `Self`.
- [ ] **Redundant `trim_end().trim()`** (`request.rs:93`) — `trim()` already strips both ends; the leading `trim_end()` is a no-op.
- [ ] **MIME type values mis-grouped in `headers` module** (`http.rs:12-13`) — `TEXT_PLAIN` and `OCTET_STREAM` are content-type values, not header names; they don't belong alongside `CONTENT_TYPE` etc.
- [ ] **Non-deterministic header order** (`response.rs:72`) — `HashMap` iteration is randomized per-process, making response headers non-reproducible across runs. `IndexMap` or `BTreeMap` would give stable ordering for easier debugging and testing.
