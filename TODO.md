# TODO

## Correctness

- [ ] **Returns 404 for unknown HTTP methods; should be 405** (`router.rs:38-42`, `main.rs:134`) — `Router::find` already returns `Match::MethodNotAllowed`, but `handle_connection` maps it to `HttpResponse::not_found()`. HTTP spec requires 405 Method Not Allowed with an `Allow` header listing valid methods for the path.
- [ ] **`Version::Unknown` silently accepted** (`request.rs:124`) — unknown version strings produce `Version::Unknown`, which is treated as non-keep-alive but not rejected. Server should reject with 505 HTTP Version Not Supported.
- [ ] **Hardcoded loopback bind address** (`main.rs:85`) — `127.0.0.1:4221` only accepts localhost connections. Should bind `0.0.0.0:4221` (or make it configurable via `--host`/`--port` args).

## Allocations

- [ ] **`&'static str` values stored as `String` in headers** (`response.rs:10`) — `ct.to_string()` and `"gzip"` are heap-allocated despite being static. Change the header map value type to `Cow<'static, str>` so static values avoid allocation.
- [ ] **`as_bytes()` builds an intermediate `Vec<u8>`** (`response.rs:32`) — every response allocates a full buffer before writing to the stream. Replace with `write_to(&self, w: &mut impl Write)` to write headers and body directly.

## Nits

- [ ] **MIME type values mis-grouped in `headers` module** (`http.rs:12-13`) — `TEXT_PLAIN` and `OCTET_STREAM` are content-type values, not header names; they don't belong alongside `CONTENT_TYPE` etc.
- [ ] **Non-deterministic header order** (`response.rs:37`) — `HashMap` iteration is randomized per-process, making response headers non-reproducible across runs. `IndexMap` or `BTreeMap` would give stable ordering for easier debugging and testing.
- [ ] **Dead `routes` HashMap field on `Router`** (`router.rs:23`) — leftover from pre-`matchit` implementation, marked `#[allow(unused)]`. Delete the field and the `HashMap` import.
