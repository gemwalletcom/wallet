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

Each chain crate has a `provider/` directory with the `chain_traits` implementations. A provider method fetches raw RPC data and hands it to a pure function in the sibling `*_mapper.rs` file, which returns the domain type. Mappers are unit-tested with fixtures; providers are covered by gated live tests.

Keep network calls, response assembly, and provider-specific orchestration in the client/provider layer. Put deterministic response-to-domain transformations and reusable pure calculations in the mapper or owning domain type.

Do not substitute network-wide data for provider-specific policy. A public chain queue, fee, or contract value does not establish a provider's batching, liquidity, minimum, or completion behavior unless that provider contract explicitly derives from it.

Reference: `crates/gem_hypercore/src/provider/balances.rs` and `balances_mapper.rs`.

## Repository Pattern

Backend services reach the database through `DatabaseClient` accessors, one per domain (`assets()`, `devices()`, `subscriptions()`, `prices()`, `transactions()`, and so on), each implementing that domain's repository trait. Repositories return primitives, not database models; business logic stays in the service that composes several accessors.

Reference: `crates/storage/src/database/mod.rs`.

## RPC Clients

- `gem_jsonrpc::JsonRpcClient` for blockchain RPC; `batch_call()` for batches; errors propagate as `JsonRpcError`
- `primitives::hex` for hex encoding, not `alloy_primitives::hex`; RPC calls take hex strings directly, avoid double encoding
- Never wrap an immutable request client in a shared `Mutex` or hold that client lock across network or database I/O. Use mutexes only for narrowly scoped mutable coordination

## UniFFI

Wrap external models with `#[uniffi::remote(Record)]` on a type alias instead of a duplicate struct plus `From` impls. Reference: `gemstone/src/transfer_amount.rs`.

## Shared Utilities

- `U256` <-> `BigUint`: `u256_to_biguint` / `biguint_to_u256` in `crates/gem_evm/src/u256.rs`
