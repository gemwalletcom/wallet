# Common Issues

Use when a review or build failure points at one of these known anti-patterns, or before adding a constant, hex helper, database enum value, or debt marker.
Known anti-patterns found in the codebase and their fixes.

## `alloy_primitives::hex` — Use `primitives::hex`

Always import `primitives::hex` for hex encoding and decoding. Existing direct `alloy_primitives::hex` imports remain in a handful of crates; fix them when you touch the file, and do not add new ones.

## Duplicate Constants

Before defining a new constant, check `crates/primitives/src/asset_constants.rs` for existing definitions. Reuse rather than redefine.

## Inline `use` in Diesel Query Functions

Diesel DSL imports (`use crate::schema::assets::dsl::*`) inside a query function are the one exception to the no-inline-imports rule: they are idiomatic Diesel and keep DSL names from colliding at module scope.

## `println!` in Service Code

Replace any `println!` in `apps/` with `tracing::info!` / `tracing::error!` and structured fields; see [Defensive Programming](defensive-programming.md).

## Tron Energy Estimation

`triggerconstantcontract` simulates the call against live state with the real recipient and value, so its `energy_used` already includes the first-time-recipient storage cost and the energy penalty. Estimate through `TriggerConstantContractResponse::get_energy()` in `gem_tron`, whose only addition is surfacing a failed simulation as an error. Never add `energy_penalty` or a static per-recipient activation fee on top; both double-count and roughly double USDT fee estimates.

## New Postgres Enum Value

Add the value to the `CREATE TYPE` in the migration that first declared it (`crates/storage/src/migrations/2023-09-05-011115_transactions/up.sql` for `transaction_state`) and add the matching variant to its `diesel_enum!` list in `crates/storage/src/sql_types.rs`. Never add a migration directory containing `ALTER TYPE ... ADD VALUE`; the repository has none, and `inTransit` and `refunded` were both added this way. The deployed database is altered separately, so state the exact `ALTER TYPE <type> ADD VALUE '<value>';` in the handoff.

## Technical Debt Markers

Existing `TODO`/`FIXME` comments mark deferred work. Resolve them only within the task's scope; otherwise leave them untouched. Follow the shared [comment policy](../../skills/engineering-principles.md#clean-code-principles) for new code; put out-of-scope follow-ups in the handoff.
