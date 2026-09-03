# Development Commands

All commands use the `just` task runner. Run from the workspace root unless specified.

## Build

```sh
just build                      # Build the workspace
just clean                      # Clean workspace build artifacts
just build-gemstone             # Build cross-platform library
just gemstone build-ios         # Build the core iOS GemTest project
just gemstone bindgen-swift     # Generate iOS Swift bindings (run in gemstone/)
just gemstone prepare-ios-package # Prepare the core iOS GemTest package (run in gemstone/)
just gemstone build-android     # Build Android AAR (run in gemstone/)
```

First-time machine setup, including the kache build cache, is in [Setup](setup.md).

## Test

```sh
just test                       # Run workspace unit tests
just test <CRATE>               # Run unit tests for a specific crate
just test-integration           # Run integration tests only
just gemstone test-ios          # Run iOS integration tests (run in gemstone/)
cargo test --test integration_test --package <CRATE> --features <FEATURE>  # Manual integration test
```

Cargo accepts one positional test filter. Run multiple filters as separate commands. Confirm the active worktree and run commands from the directory assumed by the path arguments. If parallel Cargo commands contend on workspace locks or do not return a clear final status, rerun the closing checks individually.

## Code Quality

```sh
just format                     # Format all code (prefer per-file below)
just lint                       # Run clippy with warnings as errors
just fix                        # Auto-fix clippy issues
just unused                     # Find unused dependencies with cargo-machete
```

**Formatting and Linting**:
```sh
just format
cargo clippy -p <crate> -- -D warnings
```

Most chain crates declare `default = []` and gate whole modules behind features: `gem_bitcoin` keeps `signer/` behind `signer` (enabled by `unit_tests`), `gem_keystore` keeps v3 migration behind `v3`, `swapper` keeps live clients behind `reqwest_provider`. `just test <CRATE>` passes `--all-features`, but bare `cargo test -p <crate>` and `cargo clippy -p <crate>` compile only the default set and finish in seconds with nothing from the gated modules, which reads as a pass. Lint and test with the feature that compiles the changed path, for example `cargo clippy -p gem_bitcoin --features unit_tests --all-targets -- -D warnings`, or pass `--all-features`.

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

> **IMPORTANT**: Regenerate platform bindings when a change affects the mobile API or generated models. Internal implementation changes in `gemstone/`, `swapper/`, `signer/`, or other Core crates do not require regeneration when the exposed interfaces and generated outputs are unchanged.

### Swift Bindings (iOS)
```sh
just gemstone bindgen-swift     # Generate Swift bindings only (run in gemstone/)
just gemstone prepare-ios-package # Copy Swift bindings into the GemTest local package
```
Generated files: `gemstone/generated/swift/`; the core iOS example copies these into `gemstone/tests/ios/Packages/Gemstone`.

### Kotlin Bindings (Android)
```sh
just gemstone bindgen-kotlin    # Generate Kotlin bindings only (run in gemstone/)
just gemstone build-android     # Full Android build including Kotlin binding generation (run in gemstone/)
```
Generated files: `gemstone/generated/kotlin/` → copied to `gemstone/android/gemstone/src/main/java/uniffi/`

### When to Regenerate Bindings
1. After adding/modifying public functions in `gemstone/src/lib.rs`
2. After changing any UniFFI-exposed types or interfaces
3. After changing TypeShare models consumed by either app
4. When UniFFI schema or configuration changes
5. When platform build inputs or app-side integration must change with Core

## Utilities

```sh
just localize                   # Update English localization files only
just localize-all               # Update all localization files
just generate-ts-primitives     # Generate TypeScript types from Rust
just outdated                   # Check for outdated dependencies
```
