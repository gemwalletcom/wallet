# Error Handling

Use when defining, mapping, or propagating errors, or extracting values from untyped JSON.
## Principle

Plain `Error` enums with `Display`, `std::error::Error`, `From` impls, and constructor methods; no `thiserror`. Propagate with `?`.

For errors exposed through UniFFI, keep the app-facing contract typed. Mobile code and tests must match the generated error variant or typed payload, never a localized description or message string. Messages may change without changing the domain error.

Reference: `crates/primitives/src/signer_error.rs` (`SignerError`).

## Rules

- **Reuse existing constructors.** Use lowercase constructors in `ok_or_else` / `map_err` and existing `_err` helpers for early returns. Add a constructor or `_err` twin only when current call sites need it; do not generate helpers for every variant.
- **`From` impls enable `?`.** When callers convert the same library error repeatedly (`serde_json::Error`, `HexError`), add `impl From<...>` on the domain error and drop the `map_err`. `Error::from_display` wraps any `Display` error where a dedicated `From` is overkill.
- **Do not let error plumbing bury the logic.** When several checks in one block share context (the chain, the address type), build the message once in a local closure (`let message = |reason: &str| format!("{} {reason}", chain)`) and write guard clauses (`if hash != sender_hash { return Error::invalid_input_err(message("...")); }`) instead of `cond.then_some(()).ok_or_else(...)?`. Each line should read as logic with the error as a suffix.
- **One message per rule.** If an earlier guard already bounds a value, a later "cannot fail" conversion must not repeat the guard's message. Surface the library error (`map_err(Error::from_display)`) so a future limit change is not masked by a stale, wrong message. When the conversion cannot fail at all, add no error path; see [Defensive Programming](defensive-programming.md).
- **JSON parameters go through `primitives::ValueAccess`.** `params.get_value("transactions")?.at(0)?.string()?` replaces manual `.get().ok_or()...as_str().ok_or()` chains. Put accessor methods on parent types (`TransactionLoadInput::get_data_extra()`) instead of pattern-matching at every call site.
- **Database models stay separate from domain primitives.** Convert with `as_primitive()`; see [Architecture](architecture.md) § Repository Pattern.
