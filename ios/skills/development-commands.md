# Development Commands

Use the iOS `justfile` commands by default.

## Build and Test

```bash
just bootstrap              # first-time setup
just clean                  # clean DerivedData and build artifacts
just build                  # build the app
just build-package Primitives
just test                   # run unit test plans
just test AssetsTests       # run a specific test target
just test-integration       # run UI/integration tests
just test-ui                # run the iOS UI test suite
just lint                   # run SwiftLint with autofix
just format                 # run SwiftFormat
```

## Generation and Localization

```bash
just generate               # run all generation steps
just generate-models         # regenerate model types from Rust
just generate-stone         # regenerate Rust-to-Swift bindings
just generate-swiftgen      # regenerate assets and localization code
just localize               # update localization files
```

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
