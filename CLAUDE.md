# Claude Code Instructions

## Code Modification Policy

Do not make any changes directly to the code. Instead, always present proposed changes to the user and wait for explicit approval before modifying any files.

When a code change is needed:
1. Explain what change you would make and why
2. Show the proposed code diff or snippet
3. Wait for the user to confirm before applying any edits

## Route Matching Approaches

### 1. Prefix/Segment Matching (current choice)

Split request path, use the first segment to look up the handler via HashMap, pass the rest to the handler.

- Register: `/echo`, `/files`
- Matches: `/echo/hello`, `/files/foo.txt`
- Pros: simple, HashMap O(1) lookup
- Cons: only works for single-depth prefixes, breaks with nested routes like `/api/v1/users/:id`

### 2. Route Patterns

Register routes like `/echo/:param`, match segment-by-segment using lazy iterators. Zero allocations during matching.

- Register: `/echo/:value`
- Matches: `/echo/hello`, `/echo/world`
- Pros: flexible, supports params at any depth, no allocations with iterator approach
- Cons: O(routes) linear scan per request

```rust
let mut pattern_segs = pattern.split('/');
let mut path_segs = req.path.split('/');

let matched = loop {
    match (pattern_segs.next(), path_segs.next()) {
        (Some(p), Some(r)) if p.starts_with(':') || p == r => continue,
        (None, None) => break true,
        _ => break false,
    }
};
```

### 3. Trie-Based Router

Tree structure where each node is a path segment. Walk one level per segment.

- Lookup: O(segments) regardless of route count
- Pros: most efficient for large route sets
- Cons: more complex to implement, more upfront memory
