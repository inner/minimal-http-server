# TODO

## Allocations

- [ ] **`&'static str` values stored as `String` in headers** (`response.rs:10`) — `ct.to_string()` and `"gzip"` are heap-allocated despite being static. Change the header map value type to `Cow<'static, str>` so static values avoid allocation.
- [ ] **`as_bytes()` builds an intermediate `Vec<u8>`** (`response.rs:32`) — every response allocates a full buffer before writing to the stream. Replace with `write_to(&self, w: &mut impl Write)` to write headers and body directly.

## Nits

- [ ] **MIME type values mis-grouped in `headers` module** (`http.rs:12-13`) — `TEXT_PLAIN` and `OCTET_STREAM` are content-type values, not header names; they don't belong alongside `CONTENT_TYPE` etc.
- [ ] **Non-deterministic header order** (`response.rs:37`) — `HashMap` iteration is randomized per-process, making response headers non-reproducible across runs. `IndexMap` or `BTreeMap` would give stable ordering for easier debugging and testing.
