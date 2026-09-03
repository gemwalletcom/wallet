# Tests

Use when writing or changing any Core test, fixture, or integration test gate.
## Conventions

- Integration tests in `tests/`, `#[tokio::test]` for async, names prefixed `test_`, `Result<(), Box<dyn std::error::Error + Send + Sync>>` for test errors
- Configure integration tests with `test = false` and `required-features` so they run only on demand
- Prefer real networks for RPC client tests (Ethereum mainnet); unit tests use pure fixtures or injected clients
- `.unwrap()` in tests, not `.expect("...")`; the test name already says what failed
- Compare whole values with `assert_eq!` against a constructed expected value (derive `PartialEq` on test-relevant types). Do not destructure with `let ... else { panic!() }`, and do not `assert!(x.contains(...))`
- One test function with many assertions per behavior, named `test_<function_name>`

## What a Test Must Protect

- An independent domain rule, invariant, or failure boundary, never implementation details or serialized output. If the test still passes when the rule flips or the function returns a hardcoded constant, remove or fix it
- Do not unit-test static lookup tables, fixture catalogs, enum-to-variant wiring, or literal configuration by copying their values into assertions. Test the behavior that consumes the data, an invariant shared across entries, or a validation boundary; with no independent behavior, add no test
- No tolerance-based assertions against live network values or values recomputed from separate RPC/API calls; they are flaky and low-signal. Integration tests assert stable invariants; exact numeric behavior belongs in unit tests with deterministic inputs

## Test Data

JSON longer than about 20 lines lives in the crate's `testdata/` and loads with `include_str!`; never embed request, response, or transaction JSON with `serde_json::json!` in test files. Per-crate layout is `src/`, `tests/`, `testdata/`.

## Testkit Mocks

- Reusable fixtures live in the owning crate's `testkit` module as `impl Type { pub fn mock() -> Self }`; parameterize with `mock_with_*` or a clearly named variant only when needed. A fixture needed outside one assertion goes in testkit first, never as a helper in a test module.
- Consume another crate's fixtures by enabling its `testkit` feature under `[dev-dependencies]`; do not re-create a local copy.
- Share a fixture only when its domain meaning is shared. Identical address or payload literals used for different provider roles are not automatically the same fixture. Keep provider-specific wallet or identity fixtures in that provider's testkit, give constants a `TEST_` prefix, and gate live-test-only fixtures with the same feature as their consumers.
- A small local constructor function for a frequently built enum variant inside one test module is fine.

Reference: `crates/primitives/src/testkit/asset_mock.rs`, `crates/storage/src/testkit/scan_address_mock.rs`, `crates/gem_hypercore/src/testkit.rs`.

In `gemstone`, a service folder's store and port doubles live in that folder's `testkit.rs` (`#[cfg(test)] pub(crate) mod testkit;`), named after the trait (`MemoryPreferencesStore`, `MemoryConnectionStore`); tests in other folders import them from there rather than re-implementing the trait.

## Contract Return Data and Signer Vectors

Build mocked `eth_call` return data with the generated contract bindings (`<Contract>::<method>Call::abi_encode_returns(...)`, see the `gem_tempo` fee calculator tests) instead of hand-rolled hex. Signer tests assert the byte-exact signed transaction hex (see `gem_evm/src/signer/chain_signer.rs`); do not add decoder helpers to re-derive fields a vector already pins.

## Integration Testing

- Add integration tests for RPC functionality to verify real network compatibility
- Gate live chain RPC tests behind `chain_integration_tests` and live swap provider tests behind `swap_integration_tests`; keep ordinary unit tests deterministic
- Prefer recent blocks for batch operations (more reliable than historical blocks)
- Verify both successful calls and proper error propagation
- Use realistic contract addresses (e.g., USDC) for `eth_call` testing
- Treat `--no-run` as compilation coverage only. Confirm that the intended live test actually ran before reporting provider compatibility.
