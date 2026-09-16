# Code Style

Use for any Rust change; it is the crate-level style contract.
Follow the existing code style patterns unless explicitly asked to change.

## Formatting

Line length 180 (`rustfmt.toml`), 4-space indentation, imports reordered by rustfmt. Format with `just format`.

## Naming

- Files and modules `snake_case`; crates prefixed (`gem_*` for blockchains, `security_*` for security); functions and variables `snake_case`; types `PascalCase`; constants `SCREAMING_SNAKE_CASE`.
- Rely on scope instead of repeating the module or crate prefix inside it: `is_spot_swap` inside `gem_hypercore::core_signer`, not `is_hypercore_spot_swap`.
- Name functions after the domain action and result they own: `parse_destination_tag`, `build_transfer_message`, `sign_trust_set`, `map_balance_assets`. Avoid `util`, `utils`, `normalize`, `resolve`, `process`, `handle`, `manage`, `perform`, `execute` unless a framework or protocol owns the signature.
- No type suffixes (`_str`, `_int`, `_vec`); the type system already says it.
- Follow the shared [comment policy](../../skills/engineering-principles.md#clean-code-principles), including in `mod.rs`. Omit redundant local type annotations; keep those required for inference or a meaningful type boundary.
- Keep `value` for variables derived from `input.value`; use `amount` when the external protocol field requires that name.

## Imports

- Every `use` declaration at the top of the file, ordered standard library, external crates, local crate, then `pub use` re-exports.
- Never import inside a function and never write a fully qualified path inline, including in impl targets, bounds, fields, parameters, and return types. Import the type first. The one exception is Diesel DSL imports inside query functions (see [Common Issues](common-issues.md)).

## Code Organization

- **Modular structure**: Break down long files into smaller, focused modules by logical responsibility
- **Thin module entrances**: For multi-file modules, prefer a directory module with a thin `mod.rs` (or crate `lib.rs`) that only declares submodules and re-exports the public surface; keep implementation details in focused child files
- **Avoid duplication**: Search for existing implementations before writing new code; reuse existing code or crates
- **Extraction**: Follow [Abstractions Must Earn Their Place](../../skills/engineering-principles.md#abstractions-must-earn-their-place); prefer the existing owning crate
- **Avoid `mut`**: Prefer immutable bindings; use `mut` only when truly necessary
- **No `#[allow(dead_code)]`**: Remove dead code instead of suppressing warnings
- **Avoid `#[serde(default)]`**: Only use when the field is genuinely optional in the API response; if the field is always present, omit it
- **Enum accessors**: Reuse existing typed accessors for repeated extraction. Implement variant-dependent behavior with exhaustive matching; add an accessor only for a current domain operation
- **No `assert!` with `contains`**: Use `assert_eq!` with concrete values; `assert!(x.contains(...))` gives useless failure messages
- **No fallback, fail fast**: Follow [Defensive Programming](defensive-programming.md); propagate errors with `?` rather than masking them with a default.
- **Helper ownership**: Follow [Architecture § 6](../../docs/ARCHITECTURE.md#6-where-derived-domain-answers-live): methods for intrinsic behavior, pure functions in existing mapper/rules modules when there is no natural receiver. Never create a struct or generic `utils`, `normalizer`, or `codec` module just to house a helper
- **No unused fields**: Remove unused fields from structs/models; don't keep fields "for future use"
- **Constants for magic numbers**: Extract magic numbers into named constants with clear meaning
- **Minimum interface**: Don't expose unnecessary functions; if client only needs one function, don't add multiple variants
