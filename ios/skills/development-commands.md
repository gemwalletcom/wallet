# Development Commands

Use the iOS `justfile` commands by default.

## Build and Test

```bash
just bootstrap              # first-time setup
just clean                  # clean DerivedData and build artifacts
just build                  # build the app
just build-for-testing      # build once for repeated test runs
just build-package Primitives # build one Swift package/scheme
just test                   # run unit test plans
just test-without-building  # re-run tests after build-for-testing
just test AssetsTests       # run a specific test target
just test-integration       # run iOS integration/UI tests
just test-ui                # run iOS integration/UI tests
just lint                   # run SwiftLint with autofix
just format                 # run SwiftFormat
```

## Generation and Localization

```bash
just generate               # run all generation steps
just generate-models         # regenerate model types from Rust
just generate-stone         # regenerate UniFFI sources and iOS Rust static libraries
just generate-swiftgen      # regenerate assets and localization code
just localize               # update localization files
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
just format                 # format Swift sources when needed
```

## Command Rules

- Use `just` commands for builds and tests, not `xcrun swift test`
- Build logs live under `build/DerivedData`
- If `just build` is insufficient for debugging, use `xcodebuild` directly against `Gem.xcodeproj`
- `just bootstrap` installs `swiftgen` and `swiftformat`
