# Development Commands

All commands use the `just` task runner. Run from the workspace root unless specified.

## Build

```sh
just build                      # Build the workspace
just clean                      # Clean workspace build artifacts
just build-gemstone             # Build cross-platform library
just gemstone bindgen-swift     # Generate iOS Swift bindings (run in gemstone/)
just gemstone build-android     # Build Android AAR (run in gemstone/)
```

First-time machine setup, including the mbx build cache, is in [Setup](setup.md).

## Test

```sh
just test                       # Run workspace unit tests
just test <CRATE>               # Run unit tests for a specific crate
just test <CRATE> <FILTER>      # Run matching unit tests in one crate
just test "" <FILTER>           # Run matching workspace unit tests
just build-integration-tests   # Compile all integration suites without running them
just test-integration           # Run integration tests only
cargo test --test integration_test --package <CRATE> --features <FEATURE>  # Manual integration test
just fuzz [SECONDS]             # Fuzz every target in fuzz/ in parallel (default 600s); crash inputs land in fuzz/artifacts/<target>/
```

`just test <CRATE>` includes `--lib` and requires a library target. For binary-only packages such as `generate`, run `cargo test --locked -p generate --bins --all-features` instead.

The optional filter matches test names (for example, `just test gemstone services::assets`). Omitting it keeps the full unit suite. Crate runs pass `--all-features` and skip `integration_tests`; workspace runs rely on feature unification across members. Check the test count so an unmatched filter is not mistaken for coverage.

`just build-integration-tests` compiles workspace library tests once with the union of the chain, swapper, fiat, NFT, price, security, and database integration features. Swap integration tests belong to the `swapper` crate. It does not run live tests; `just test-integration` runs each suite separately with serial test execution.

Cargo accepts one positional test filter. Run multiple filters as separate commands. Confirm the active worktree and run commands from the directory assumed by the path arguments. If parallel Cargo commands contend on workspace locks or do not return a clear final status, rerun the closing checks individually.

## Code Quality

```sh
just format                     # Format all code (prefer per-file below)
just lint                       # Run clippy with warnings as errors and the extra lints in `clippy_lints`
just fix                        # Auto-fix clippy issues
just unused                     # Check unused dependencies with Cargo
```

**Formatting and Linting**:
```sh
just format
just lint
cargo clippy -p <crate> --all-features -- -D warnings
```

Before committing or pushing Core changes, run `just lint`, the same command as Core CI. It enables `redundant_clone`, `implicit_clone` and `redundant_closure_for_method_calls` in addition to treating warnings as errors, and runs clippy a second time in release, where testkits do not exist, so production code that calls a mock fails. Per-crate clippy checks provide focused feature coverage and do not replace this closing check.

Most chain crates declare `default = []` and gate whole modules behind features: `gem_bitcoin` keeps `signer/` behind `signer`, `gem_keystore` keeps v3 migration behind `v3`. A crate whose tests need its gated modules lists itself under `[dev-dependencies]` with those features, for example `gem_bitcoin = { path = ".", features = ["rpc", "signer"] }`, using only features the workspace build already enables. `cargo test -p <crate>` and `just test <CRATE>` then compile the tests CI runs, without the integration features. A lib-only `cargo clippy -p <crate>` skips dev-dependencies and still compiles only the default set, which finishes in seconds and reads as a pass: lint with `--all-targets` or the feature that compiles the changed path, for example `cargo clippy -p gem_bitcoin --features signer --all-targets -- -D warnings`.

The reqwest-backed clients (`ReqwestClient`, `ReqwestGrpcTransport`, `NativeProvider`) live in `gem_reqwest`, which only server apps, tools and tests depend on. Cargo unifies features across the workspace, so a normal dependency from `gem_client`, `gem_jsonrpc`, a chain crate or `swapper` on reqwest makes every crate on the app path wait for the TLS stack in `just test`, `just lint` and CI. That includes an optional dependency behind `chain_integration_tests`, because CI's build turns every integration feature on. Chain crates and `swapper` list `gem_reqwest` under `[dev-dependencies]`, which only their test binaries link, and keep the code that builds live clients under `#[cfg(test)]`.

## Dependency Hygiene

With Rust 1.100 or newer, `just unused` checks all workspace targets and features using Cargo's built-in unused-dependency warnings. Before removing a reported dependency, check target-specific uses, feature activation, and procedural macros. Cargo does not check unused dev-dependencies; use `cargo machete` as a supplementary audit when needed.

The Core Cargo configuration sets a seven-day minimum publish age for registry dependency resolution. Existing locked versions remain usable, so use `--locked` for verification. This cooldown does not apply to `cargo install` or registry entries without publication timestamps. For an intentionally reviewed urgent update, override only that command with `CARGO_RESOLVER_INCOMPATIBLE_PUBLISH_AGE=allow cargo update -p <crate> --precise <version>`.

## Cargo.lock Conflicts

Resolve `Cargo.lock` merge or rebase conflicts by taking the union of the two committed versions, never by letting Cargo regenerate the file: a fresh resolve moves unrelated crates to whatever the local registry cache offers, and that can differ from crates.io and break CI. Validate the result with `cargo metadata --offline --locked`, and run the follow-up checks with `--locked` so Cargo cannot rewrite the lock silently.

## Database

```sh
just migrate                    # Run Diesel migrations
just setup-services             # Start Docker services (PostgreSQL, Redis, Meilisearch, RabbitMQ)
```

## Mobile

```sh
just gemstone install-ios-targets       # Install iOS Rust targets (run in gemstone/)
just gemstone install-android-targets   # Install Android Rust targets and cargo-ndk (run in gemstone/)
```

Note: Mobile builds require UniFFI bindings generation and platform-specific compilation.

## Generating Bindings (When Core Changes Affect Mobile APIs)

Apply [Cross-Platform Awareness](../../skills/cross-platform-awareness.md) to decide when generation and app verification are required, including UniFFI schema or configuration changes.

For the apps, `just generate-stone` from the repo root builds the iOS library and bindings, and the Android Gradle build regenerates its own. The recipes below generate bindings on their own.

### Swift Bindings (iOS)
```sh
just gemstone bindgen-swift     # Generate Swift bindings only (run in gemstone/)
```
Generated files: `gemstone/generated/swift/`

### Kotlin Bindings (Android)
```sh
just gemstone bindgen-kotlin    # Generate Kotlin bindings only (run in gemstone/)
just gemstone build-android     # Full Android build including Kotlin binding generation (run in gemstone/)
```
Generated files: `gemstone/generated/kotlin/` → copied to `gemstone/android/gemstone/src/main/java/uniffi/`

## Utilities

```sh
just localize                   # Update English localization files only
just outdated                   # Check for outdated dependencies
```
