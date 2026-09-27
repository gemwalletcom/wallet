# Architecture

Use when adding or changing a provider, mapper, repository, RPC client, or UniFFI-exposed type.
## Key Principles

- One crate per blockchain with the unified RPC client pattern; family crates stay chain-agnostic (see [New Chain Checklist](new-chain-checklist.md))
- UniFFI-exposed APIs are designed for mobile: `Send + Sync`, no lifetimes, typed errors
- `BigDecimal` / `BigUint` for financial values, never floats
- async/await on Tokio throughout
- Diesel ORM with automatic migrations for backend storage
- Mobile performance matters: batch RPC calls and avoid chatty request sequences

## Provider and Mapper

When adding or refactoring multiple providers, keep sibling implementations consistent with [fiat providers](../crates/fiat/src/providers/) and [price providers](../crates/prices/src/providers/): one directory per provider, a thin `mod.rs` for module declarations and re-exports, `provider.rs` for the shared trait implementation and orchestration, and `mapper.rs` for pure transformations. Compare all affected siblings during review so one provider does not accumulate a different layout or contract.

Share traits and configuration types at the family level. Reuse existing clients and nested config structs instead of duplicating wrappers or flattened configuration. Add provider-local client, target, model, and testkit modules only when needed; remove unused fields, imports, dependencies, and exports. Do not create empty modules or unsupported trait methods merely to match another provider's file list.

Each chain crate has a `provider/` directory with the `chain_traits` implementations. A provider method fetches raw RPC data and hands it to a pure function in the sibling `*_mapper.rs` file, which returns the domain type. Mappers are unit-tested with fixtures; providers are covered by gated live tests.

Keep network calls, response assembly, and provider-specific orchestration in the client/provider layer. Put deterministic response-to-domain transformations and reusable pure calculations in the mapper or owning domain type.

Do not substitute network-wide data for provider-specific policy. A public chain queue, fee, or contract value does not establish a provider's batching, liquidity, minimum, or completion behavior unless that provider contract explicitly derives from it.

Reference: `crates/gem_hypercore/src/provider/balances.rs` and `balances_mapper.rs`.

## Backend Layers

`api` and `daemon` are transport, `services` orchestrates, domain crates decide, infra crates reach our own systems.

- `api` routes and `daemon` consumers, workers and parser decode input, call a service, and map the result. They hold no queries or business rules.
- `services` owns every backend use case: load from storage or cache, call domain crates, save, publish. `Services::new(settings)` builds the backend graph for both apps. Each table has one writing module; other modules call it.
- Domain crates (`fiat`, `nft`, `prices`, `swapper`, chain crates, …) hold pure rules and stateless third-party provider clients. They take and return `primitives` types and receive config values as parameters.
- Infra crates (`storage`, `cacher`, `streamer`, `search_index`, `pusher`) reach Postgres, Redis, RabbitMQ, Meilisearch and Gorush with `primitives` in and out and no business rules. Only `services` depends on them; `just check-boundaries` enforces it.
- Consuming is transport and stays in the apps: RabbitMQ queues in daemon consumers (`streamer` readers and `run_consumer`, the one infra dependency the daemon keeps) and the api websocket's Redis pub/sub subscription. The consumer passed to `run_consumer` and all publishing come from `services`.
- A database transaction closure is sync: fetch from providers first, then open the transaction.
- Traits define provider families (`FiatProvider`, `ListProvider`, chain providers), even while a family has one implementation; no ports around the database or our other infra.

## Repository Pattern

Backend code reaches Postgres through `Database::run(|client| …)`, or `Database::transaction(|client| …)` when several writes must commit together. The closure runs on a blocking thread with one pooled connection, so async workers never block on diesel. Put the queries of one unit of work in one closure and keep network calls outside it. Repository traits are implemented on `DatabaseClient` and take and return `primitives` types; row models, `sql_types` wrappers and `schema` stay `pub(crate)` to `storage`. When no primitive fits (surrogate ids, partial projections), return a small plain struct from the repository module (`DeviceRecord`, `PriceAsset`). Resolve surrogate keys inside storage; business logic stays in the service that composes the repositories.

Reference: `crates/storage/src/lib.rs` (`Database`).

## RPC Clients

- Follow [Architecture § 12](../../docs/ARCHITECTURE.md#12-a-clients-requests-are-one-enum-the-client-only-sends) for request targets, client responsibilities, deliberate transport exceptions, and reference implementations
- `gem_jsonrpc::JsonRpcClient` for blockchain RPC; `batch_request()` for batches; errors propagate as `JsonRpcError`
- `primitives::hex` for hex encoding, not `alloy_primitives::hex`; RPC calls take hex strings directly, avoid double encoding
- Never wrap an immutable request client in a shared `Mutex` or hold that client lock across network or database I/O. Use mutexes only for narrowly scoped mutable coordination

## UniFFI

Wrap external models with `#[uniffi::remote(Record)]` on a type alias instead of a duplicate struct plus `From` impls. Reference: `gemstone/src/transfer_amount.rs`.

An exported object that keeps state behind a `Mutex` reads it once per call into a local and derives the whole answer from that snapshot. A guard created inside a larger expression, such as one field of a struct literal, lives until the expression ends, so a later field that locks the same mutex again blocks the calling app thread forever.

## Shared Utilities

- `U256` <-> `BigUint`: `u256_to_biguint` / `biguint_to_u256` in `crates/gem_evm/src/u256.rs`
