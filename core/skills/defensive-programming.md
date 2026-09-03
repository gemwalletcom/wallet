# Defensive Programming

Use for any production Rust change.
Safety rules that are not negotiable in production Rust. Everything beyond them is over-defensive; see `skills/engineering-principles.md` § No Over-Defensive Code.

- **Exhaustive `match`, never `matches!`.** `matches!` silently returns `false` for variants added later; an exhaustive `match` makes the compiler point at every site that must decide about the new variant. When a boolean query over an enum is common, expose it as an accessor method on the type.
- **No `#[allow(dead_code)]`.** Delete the code instead.
- **No `todo!()` or `unimplemented!()`.** Implement the case or return a typed error such as `Error::UnsupportedChain(chain)`, so the failure is a `Result`, not a crash.
- **No `println!` in service code.** Use `tracing::info!` / `tracing::error!` with structured fields so logs carry levels, timestamps, and monitoring context.
- **No `.unwrap()` or `.expect()` in production code.** Propagate with `?` and a typed error; convert options with `.ok_or(Error::MissingKey("key"))?`. `.unwrap()` is fine in tests (see [Tests](tests.md)).
- **No fallback that hides a failure.** `unwrap_or(0)`, `unwrap_or_default()`, or an empty collection on error masks the bug at its source; return the `Result`.
- **Prefer immutable bindings.** `mut` only where ownership requires it, with the narrowest scope.
