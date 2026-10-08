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
- Consumers receive narrow traits for the provider and infrastructure operations they use. Concrete adapters stay in the composition root; do not add a generic forwarding port that merely mirrors an infrastructure client.
- A use case that rejects requests for business reasons returns a typed service error ([`FiatServiceError`](../crates/services/src/fiat/error.rs), [`RewardsServiceError`](../crates/services/src/rewards/error.rs)); the API maps each variant explicitly. The boxed `ApiError` fallback only maps storage, cache and upstream failures.

## Repository Pattern

PostgreSQL adapters reach the database through `Database::run(|client| …)`, or `Database::transaction(|client| …)` when several writes must commit together. The closure runs on a blocking thread with one pooled connection, so async workers never block on diesel. Put the queries of one unit of work in one closure and keep network calls outside it. Storage-level repository traits are implemented on `DatabaseClient` and take and return `primitives` types; row models, `sql_types` wrappers and `schema` stay `pub(crate)` to `storage`. When no primitive fits (surrogate ids, partial projections), return a small plain struct from the repository module (`DeviceRecord`, `PriceAsset`). Resolve surrogate keys inside storage; business logic stays in the service that composes the repositories.

A service receives an async repository port for the operations it uses rather than holding `Database` or `DatabaseClient`. Its PostgreSQL adapter owns `Database`, imports the storage-level repository traits and preserves each existing `run` or `transaction` unit. One domain module has one `repository::Repository`; reuse and extend it only for operations with current consumers rather than making a forwarding trait per service. [`assets::repository`](../crates/services/src/assets/repository.rs) is the example. Only these adapters (`repository.rs`, or a `repository/` module when a domain needs helper files) and the composition roots (`backend.rs`, `workers.rs`, `consumers.rs`) touch `Database`, `DatabaseClient` or storage-level repository traits; `just check-boundaries` enforces it. When a unit interleaves reads, a rule and a write, the adapter keeps the unit and calls the rule from domain or service code; when it only reads, it returns the facts and the service decides.

### Repository injection example

Keep the domain repository, service and shared test double in separate files:

```text
services/src/
├── assets/
│   ├── repository.rs
│   └── asset_rank_updater.rs
└── testkit/
    └── asset_repository.rs
```

The domain's `repository.rs` owns one narrow async port and its PostgreSQL adapter. The port speaks primitives and use-case input records; only the adapter imports `Database`, `DatabaseClient` or storage-level repository traits:

```rust
#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn enabled_assets_at_or_below(&self, rank: AssetRank) -> Result<Vec<AssetBasic>, DatabaseError>;
    async fn disable_assets_with_ranks(&self, changes: Vec<RankChange>) -> Result<usize, DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn disable_assets_with_ranks(&self, changes: Vec<RankChange>) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                changes.into_iter().try_fold(0, |count, change| {
                    let updated = client.update_assets(change.asset_ids, vec![AssetUpdate::Rank(change.rank.threshold()), AssetUpdate::IsEnabled(false)])?;
                    Ok::<_, DatabaseError>(count + updated)
                })
            })
            .await
    }
}
```

The service holds only the port, makes the domain decision and sends the resulting persistence changes through one repository call:

```rust
pub struct AssetRankUpdater {
    repository: Arc<dyn Repository>,
    classification_rules: AssetClassificationRules,
}

let count = self.repository.disable_assets_with_ranks(changes).await?;
```

The composition root builds the concrete adapter. A call site never constructs it:

```rust
AssetRankUpdater::new(
    Arc::new(PostgresRepository::new(self.database.clone())),
    self.classification_rules.clone(),
)
```

The reusable double lives in the crate testkit beside the other doubles, not inside a service test. It records every write and any read input the test needs to verify:

```rust
pub(crate) struct MemoryAssetRepository {
    candidates: Vec<AssetBasic>,
    ranks: Mutex<Vec<AssetRank>>,
    changes: Mutex<Vec<RankChange>>,
}

#[async_trait]
impl Repository for MemoryAssetRepository {
    async fn enabled_assets_at_or_below(&self, rank: AssetRank) -> Result<Vec<AssetBasic>, DatabaseError> {
        self.ranks.lock().unwrap().push(rank);
        Ok(self.candidates.clone())
    }

    async fn disable_assets_with_ranks(&self, changes: Vec<RankChange>) -> Result<usize, DatabaseError> {
        let count = changes.iter().map(|change| change.asset_ids.len()).sum();
        self.changes.lock().unwrap().extend(changes);
        Ok(count)
    }
}
```

Register and re-export the double from `src/testkit/mod.rs`, then import that shared type in the service test:

```rust
mod asset_repository;

pub(crate) use asset_repository::MemoryAssetRepository;
```

A service test supplies literal inputs, calls the real service and asserts the recorded repository interaction. A one-off double that delays or counts a single call may stay inline with that one test; a generally useful repository double belongs in `src/testkit`.

Reference: `crates/storage/src/lib.rs` (`Database`).

## Cacher Pattern

Redis is reached through cachers in the `cacher` crate, one `src/cachers/<name>.rs` per contract: a `…Cacher` trait with the async operations its consumers use and its `impl` on `CacherClient`. A cacher takes and returns `primitives` types, or a small plain struct from its module when none fits (`CachedFiatQuote`, `SafeScanTarget`). Keys, TTLs, serialization and pub/sub channels stay inside the crate: `CacheKey` and the raw client methods are `pub(crate)`. Because a cacher is already a narrow async port, a service receives `Arc<dyn …Cacher>` from the composition root directly, without a second domain port around it; decisions such as generated ids, defaults and what a missing entry means stay in the service. `Store` names persistence ports, not caches.

Reference: `crates/cacher/src/cachers/mod.rs`.

## RPC Clients

- Follow [Architecture § 12](../../docs/ARCHITECTURE.md#12-a-clients-requests-are-one-enum-the-client-only-sends) for request targets, client responsibilities, deliberate transport exceptions, and reference implementations
- `gem_jsonrpc::JsonRpcClient` for blockchain RPC; `batch_request()` for batches; errors propagate as `JsonRpcError`
- `primitives::hex` for hex encoding, not `alloy_primitives::hex`; RPC calls take hex strings directly, avoid double encoding
- Never wrap an immutable request client in a shared `Mutex` or hold that client lock across network or database I/O. Use mutexes only for narrowly scoped mutable coordination

## UniFFI

Wrap external models with `#[uniffi::remote(Record)]` on a type alias instead of a duplicate struct plus `From` impls. Reference: `gemstone/src/transfer_amount.rs`.

Fallible foreign methods return `Result`; their error type converts unexpected callback errors into a declared variant instead of panicking. App callbacks map known platform errors, such as offline failures, and let UniFFI forward unexpected errors to this fallback. Keep shared errors such as `AlienError` remote and implement `From<uniffi::UnexpectedUniFFICallbackError>` in the owning crate behind an optional UniFFI feature enabled by gemstone. Gemstone-owned errors implement the same fallback directly.

An exported object that keeps state behind a `Mutex` reads it once per call into a local and derives the whole answer from that snapshot. A guard created inside a larger expression, such as one field of a struct literal, lives until the expression ends, so a later field that locks the same mutex again blocks the calling app thread forever.

## Shared Utilities

- `U256` <-> `BigUint`: `u256_to_biguint` / `biguint_to_u256` in `crates/gem_evm/src/u256.rs`
