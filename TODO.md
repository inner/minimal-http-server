# TODO

## Bugs

- [x] **Path traversal vulnerability** (`files.rs:13,20`) — Fixed: `FileManager::read` now uses `fs::canonicalize` to resolve the real path and verifies it starts with the base directory, catching both `..` traversal and symlinks. `FileManager::create` rejects any path segment equal to `..` (canonicalize can't be used since the file doesn't exist yet). Note: a production-grade solution would use `openat` with directory file descriptors to eliminate the TOCTOU race condition entirely, but this requires `unsafe` or a crate like `cap-std`.
- [ ] **BufReader lost on persistent connections** (`main.rs:134`, `request.rs:36`) — `HttpRequest::new` creates a new `BufReader` each loop iteration, discarding any bytes buffered from the previous read. The `BufReader` must outlive the loop and be passed into the request parser.
- [ ] **Deflate encoding declared but not applied** (`response.rs:52-66`, `http.rs:24`) — `with_encoding` sets `Content-Encoding: deflate` but `with_gzip_body` always uses gzip. Clients requesting deflate get a mismatched response.
- [ ] **`.unwrap()` panics in `with_gzip_body`** (`response.rs:36-37`) — `encoder.write_all` and `encoder.finish` can fail; panicking here will crash the worker thread. Return a `Result` or handle the error gracefully.

## Code Smells

- [ ] **Handlers re-parse paths** (`main.rs:27,54,69`) — `handle_echo`, `handle_return_file`, and `handle_read_body` all call `strip_prefix` on the path the router already split. The router should pass the remainder to handlers directly.
- [ ] **Duplicate `stream.write_all`** (`main.rs:138-144`) — the write appears in both branches of the `close_connection` check. Insert the header conditionally, then write once outside the `if`.
- [ ] **`find(":")` should be `find(':')`** (`request.rs:69`) — use a `char` literal instead of a string slice for header colon search.
- [ ] **`if self.body.len() > 0`** (`response.rs:35`) — prefer `!self.body.is_empty()`.
- [ ] **Verbose `close_connection` assignment** (`request.rs:91-94`) — replace the `let mut` + `if` block with a direct `let close_connection = headers.get("connection").is_some_and(|v| v == "close");`.

## Design

- [ ] **`Box::leak` instead of `Arc`** (`main.rs:89-108`) — `args` and `router` are leaked to get `'static` references for threads. Use `Arc<T>` for explicit, non-leaking shared ownership.
- [ ] **`workers` is `pub` on `ThreadPool`** (`threadpool.rs:42`) — nothing outside the module uses it; make it private.
- [ ] **HTTP version not parsed** (`request.rs:41-50`) — the version token is discarded. HTTP/1.0 should default `close_connection` to `true`; currently all connections behave as HTTP/1.1.

## Allocations

- [ ] **`format!("/{prefix}")` on every request** (`router.rs:31-35`) — allocates a `String` per request just for a HashMap lookup. Replace the HashMap with a `Vec<(Method, &'static str, Handler)>` and linear scan to compare `&str` directly, zero allocation per lookup.
- [ ] **`with_encoding` takes `String` instead of `&str`** (`response.rs:52`, `main.rs:39`) — `String::from(content_encoding)` allocates at the call site just to pass ownership in; the method only calls `.split(',')`. Change the signature to `&str`.
- [ ] **`&'static str` values stored as `String` in headers** (`response.rs:47-49,58-62`) — `ct.to_string()` and encoding names `"gzip"`/`"deflate"` are heap-allocated despite being static. Change the header map value type to `Cow<'static, str>` so static values avoid allocation.
- [ ] **`as_bytes()` builds an intermediate `Vec<u8>`** (`response.rs:68`) — every response allocates a full buffer before writing to the stream. Replace with `write_to(&self, w: &mut impl Write)` to write headers and body directly.
- [ ] **Args collected to `Vec<String>` for `.chunks(2)`** (`main.rs:88-99`) — `.chunks()` requires a slice, forcing a heap allocation. Use a manual peekable iterator to parse pairs without collecting.
