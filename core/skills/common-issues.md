# Common Issues

Use when a review or build failure points at one of these known anti-patterns, or before adding a constant, hex helper, or debt marker.
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

## Technical Debt Markers

`TODO`/`FIXME` comments mark deferred work. When working near one, resolve it if the task's scope permits; otherwise leave it untouched rather than rewording it. Do not add new markers for work you could finish in the same change.
