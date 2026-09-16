# Defensive Programming

Use for any production Rust change.
Apply these production rules together with [Security](../../skills/security.md). Avoid repeated checks only after the relevant invariant is established; see [No Over-Defensive Code](../../skills/engineering-principles.md#no-over-defensive-code).

- **Exhaustive `match`, never `matches!`.** `matches!` silently returns `false` for variants added later; an exhaustive `match` makes the compiler point at every site that must decide about the new variant. When a boolean query over an enum is common, expose it as an accessor method on the type.
- **No `#[allow(dead_code)]`.** Delete the code instead.
- **No `todo!()` or `unimplemented!()`.** Implement the case or return a typed error such as `Error::UnsupportedChain(chain)`, so the failure is a `Result`, not a crash.
- **No `println!` in service code.** Use `tracing::info!` / `tracing::error!` with structured fields so logs carry levels, timestamps, and monitoring context.
- **No `.unwrap()` or `.expect()` in production code.** Propagate with `?` and a typed error; convert options with `.ok_or(Error::MissingKey("key"))?`. `.unwrap()` is fine in tests (see [Tests](tests.md)).
- **No fallback that hides a failure.** `unwrap_or(0)`, `unwrap_or_default()`, or an empty collection on fallible or external data masks the bug at its source; return the `Result`.
- **No error variant for a state that cannot occur.** When an earlier layer already established the invariant, a conversion that cannot fail takes the default (`BigInt::to_u64().unwrap_or_default()` on a fee this crate's own preload produced) instead of an invented `ok_or_else(|| Error::invalid_input(...))`. Reserve `Result` for conditions a caller can actually hit: user input, network data, insufficient balance.
- **Prefer immutable bindings.** `mut` only where ownership requires it, with the narrowest scope.
