# Tests

Use when writing or changing any Core test, fixture, or integration test gate.
## Conventions

- Integration tests in `tests/`, `#[tokio::test]` for async, names prefixed `test_`, `Result<(), Box<dyn std::error::Error + Send + Sync>>` for test errors
- Configure integration tests with `test = false` and `required-features` so they run only on demand
- Test deterministic RPC behavior with fixtures or injected clients; use gated live tests for network compatibility
- `.unwrap()` in tests, not `.expect("...")`; the test name already says what failed
- Compare whole values with `assert_eq!` against a constructed expected value (derive `PartialEq` on test-relevant types). Do not destructure with `let ... else { panic!() }`, and do not `assert!(x.contains(...))`
- One test function with many assertions per behavior, named `test_<function_name>`

## What a Test Must Protect

- Follow the shared [test-intent rule](../../skills/engineering-principles.md#tests). Exact bytes or serialized output belong in tests when they are the contract: signing vectors, request encoding, and compatibility fixtures
- Do not unit-test static lookup tables, fixture catalogs, enum-to-variant wiring, or literal configuration by copying their values into assertions. Test the behavior that consumes the data, an invariant shared across entries, or a validation boundary; with no independent behavior, add no test
- No tolerance-based assertions against live network values or values recomputed from separate RPC/API calls; they are flaky and low-signal. Integration tests assert stable invariants; exact numeric behavior belongs in unit tests with deterministic inputs

## Test Data

JSON longer than about 20 lines lives in the crate's `testdata/` and loads with `include_str!`; never embed request, response, or transaction JSON with `serde_json::json!` in test files. Per-crate layout is `src/`, `tests/`, `testdata/`.

## Testkit Mocks

A mock exists once, beside its type, so every test reuses it instead of rebuilding the value.

- A test takes a value, a double, or the unit under test from a mock in the testkit of the crate or gemstone folder that defines the type: `impl Type { pub fn mock() -> Self }`, plus `mock_<shape>()` or `mock_with_<field>(..)` for shapes current tests use. A foreign type gets a free `mock_<type>()` in the consuming crate's testkit. Defaults are the usual case; parameterize only what current tests vary and set a one-off field at the call site with `Type { field, ..Type::mock() }`.
- A test module defines no fixture of its own: no `fn document()`, `fn tag(..)`, `fn service()`, `sample_*`, `create_*` or `make_*` helper, and no closure that assembles a value, even for one test. Search the owning testkit first and extend an existing mock rather than adding a near-duplicate; merge near-duplicate mocks into one with sensible defaults and delete mocks nothing calls.
- A store or port double records every write it accepts and answers reads from what it recorded. A method that takes a write and returns `Ok(())` without keeping it makes "the row was saved" pass whether or not the code saved anything, and hides the bug the test was written to catch.
- The test keeps literal inputs, `testdata/` payloads, direct calls to the real constructor, helpers that assert or decode rather than construct, and a double that probes one test's behavior (counts writes, delays a read).
- Layout: `src/testkit/mod.rs` with one `<type>_mock.rs` per type, or `src/testkit.rs` for a few mocks, declared `#[cfg(any(test, feature = "testkit"))] pub mod testkit;` behind a `testkit` feature. Binary apps declare `#[cfg(test)] mod testkit;`. A gemstone folder uses `testkit.rs` declared `#[cfg(test)] pub(crate) mod testkit;`, with store and port doubles named after the trait (`MemoryPreferencesStore`); cross-cutting doubles live in `gemstone/src/testkit.rs`.
- Consume another crate's mocks by enabling its `testkit` feature under `[dev-dependencies]`; never re-create a local copy. Enable a testkit only across a boundary the consumer's own code owns: `api` reaches data through clients and never enables `storage/testkit`. A test that has to fake another layer's rows is testing at the wrong layer or repeating that layer's test; move or delete it.
- Share a fixture only when its domain meaning is shared. Identical address or payload literals used for different provider roles are not automatically the same fixture. Keep provider-specific wallet or identity fixtures in that provider's testkit, give constants a `TEST_` prefix, and gate live-test-only fixtures with the same feature as their consumers.

Reference: `crates/primitives/src/testkit/asset_mock.rs`, `crates/storage/src/testkit/tag_mock.rs`, `crates/search_index/src/testkit/asset_list_mock.rs`, `crates/gem_hypercore/src/testkit.rs`.

## Contract Return Data and Signer Vectors

Build mocked `eth_call` return data with the generated contract bindings (`<Contract>::<method>Call::abi_encode_returns(...)`, see the `gem_tempo` fee calculator tests) instead of hand-rolled hex. Signer tests assert the byte-exact signed transaction hex (see `gem_evm/src/signer/chain_signer.rs`); do not add decoder helpers to re-derive fields a vector already pins.

## Integration Testing

- Add integration tests for RPC functionality to verify real network compatibility
- Gate live chain RPC tests behind `chain_integration_tests` and live swap provider tests behind `swap_integration_tests`; keep ordinary unit tests deterministic
- Prefer recent blocks for batch operations (more reliable than historical blocks)
- Verify both successful calls and proper error propagation
- Use realistic contract addresses (e.g., USDC) for `eth_call` testing
- Treat `--no-run` as compilation coverage only. Confirm that the intended live test actually ran before reporting provider compatibility.
