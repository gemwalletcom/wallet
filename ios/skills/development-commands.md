# Development Commands

Use the iOS `justfile` commands by default.

## Build and Test

```bash
just install                # first-time setup
just clean                  # clean DerivedData and build artifacts
just check GemstonePrimitives # compile ONE package with SwiftPM (~1-2s) — use this while iterating
just check-test Primitives  # run ONE package's tests with SwiftPM (~2s)
just build                  # build the app
just build-for-testing      # build once for repeated test runs
just build-package Primitives # build one Swift package/scheme
just test                   # run unit test plans
just test-without-building  # re-run tests after build-for-testing
just test AssetsTests       # run a specific test target
just test-integration       # run iOS integration/UI tests
just test-ui                # run iOS integration/UI tests
```

## Pick the cheapest command that can fail

Measured on a warm tree, so the gap is iteration cost, not first-build cost:

| command | warm | use it for |
|---|---|---|
| `just check <Package>` | 1-2s | does this package still compile — the default while editing a platform-independent package |
| `just check-test <Package>` | 2s | that package's tests, same limits as above |
| `just test <TestTarget>` | ~35s | one test target, including Gemstone-dependent ones |
| `just build` | ~30s even with nothing to do | the app links, before a commit |
| `just test` | ~50s | the whole suite, before a commit |

SwiftPM builds for the host, so the two `check` recipes only cover packages that import neither UIKit nor
SwiftUI, and among those only ones that do not link Gemstone can run their tests — the static library is
built for the simulator. Everything with a UI or a Gemstone dependency goes through `just build` and
`just test <TestTarget>`.

`swift build` reports every error in the package; `xcodebuild` stops at the first failing target, so a
compile-fix loop driven by `just build` costs one full build per error batch. Two habits worth keeping:
`just test` builds what it needs, so running `just build` first is a wasted 30s, and `just generate-stone`
is only needed when Core's FFI surface changed, not after every Core edit.


## Generation and Localization

```bash
just generate               # run all generation steps
just generate-models         # regenerate model types from Rust
just generate-stone         # regenerate UniFFI sources and iOS Rust static libraries
just localize               # regenerate .xcstrings catalogs and typed Localized accessors
```

From the repo root, use `just generate-stone` and `just run-ios` as the default Gemstone/iOS flow. The optional `GemStone` Xcode scheme combines cached Gemstone generation with the normal app build.

## SwiftUI Iteration

For presentation-only SwiftUI work, build the owning package first and avoid repeating full app builds for each visual adjustment:

```bash
just build-package Assets
just build-package Components
just build-package PrimitivesComponents
```

For ViewModel or display-model behavior, pair the package build with the narrowest matching test target:

```bash
just test AssetsTests
just test LockManagerTests
```

For repeated test debugging, build once and re-run without rebuilding:

```bash
just build-for-testing
just test-without-building
```

Use `just build` when the change touches app composition, navigation wiring, generated bindings, or code that cannot be validated by a package build.

## Additional Utilities

```bash
just spm-resolve-all
```

## Command Rules

- Use `just` commands for builds and tests, not `xcrun swift test`
- Build logs live under `build/DerivedData`
- If `just build` is insufficient for debugging, use `xcodebuild` directly against `Gem.xcodeproj`
- `just install` installs `swiftformat`, `swiftlint`, and `xcbeautify`
