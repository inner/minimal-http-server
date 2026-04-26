docs/codex-suggested-arch.md still describes older router behavior in places even
though src/router.rs now uses matchit, route params, and distinguishes 404 from
405.
src/router.rs duplicates route insertion logic between add() and route().
src/request.rs now depends on HttpResponse for parse-error actions, which works
but blurs the request/response layering boundary.
