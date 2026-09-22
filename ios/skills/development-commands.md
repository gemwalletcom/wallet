# Development Commands

Use the iOS `justfile` for every build and test. All xcodebuild recipes use the app project and share build settings, simulator, Gemstone linker flags and cache paths. A raw `xcodebuild` call with different settings also causes unnecessary recompilation; when you need a variation, start from the command `just -n <recipe>` prints.

## Build and Test

```bash
just install                           # first-time setup
just clean                             # clean DerivedData and build artifacts
just check GemstonePrimitives          # compile one UIKit-free package with SwiftPM
just check-test Primitives             # run one UIKit- and Gemstone-free package's tests with SwiftPM
just build-package Assets              # build one package or feature scheme
just build                             # build the app
just test-package Assets               # build and test one package on the simulator
just test GemTests                     # app-hosted tests through the Gem scheme
just test                              # run the whole unit test plan
just build-for-testing                 # build every test target once
just test-without-building AssetsTests # re-run tests without building; omit the target for the whole plan
just test-ui                           # run the UI test plan on a reset simulator
```

Run these commands from `ios/`, or prefix them with `just ios` from the repo root. The test recipes boot the simulator first. Package and targeted app-plan runs execute serially on that simulator; the whole app plan runs in parallel clones.

## Pick the cheapest command that can fail

| command | use it for |
|---|---|
| `just check <Package>` | compile a platform-independent package |
| `just check-test <Package>` | run a host-compatible package's tests |
| `just build-package <Package>` | compile a UI package or feature |
| `just test-package <Package>` | default test loop for package changes, including UI and Gemstone-dependent packages |
| `just test <TestTarget>` | app-hosted tests or verification that a target is registered in the app test plan |
| `just build` | verify app composition and linking |
| `just test` | run the whole app unit-test plan |

SwiftPM builds for the host, so the two `check` recipes only cover packages that import neither UIKit nor SwiftUI, and among those only ones that do not link Gemstone can run their tests — the static library is built for the simulator. Everything with a UI or a Gemstone dependency uses the simulator through `just build-package` or `just test-package` while iterating. Read the owning `Package.swift` for the package/scheme and `.testTarget` names; do not infer the package name from the test target.

`test-package Assets` is shorthand for `test AssetsTests`. It uses the existing app test plan, including its coverage settings and exclusions. For packages with multiple test targets or different naming, use `just test <TestTarget>`. Confirm the expected tests executed; a successful command with zero selected tests is not verification.

`swift build` reports every error in the package; `xcodebuild` stops at the first failing target, so a compile-fix loop driven by `just build` costs one build per error batch. `just test` builds what it needs, so a `just build` before it only adds a second build.

## Core Changes

The app links Core through `libgemstone.a` and the generated `Gemstone.swift`/`GemstoneFFI.h`, so run `just generate-stone` after any Core change before building or testing iOS. It builds the simulator library, generates the bindings from that library, and rewrites a binding file only when its content changed, so an unchanged FFI surface causes no Swift recompile. It takes about 3s when Core is unchanged and about 20s after a `gemstone` edit.

`just generate-stone release` (or `BUILD_MODE=release`) builds the device library in release. An Xcode simulator release builds the simulator library instead. `GEMSTONE_IOS_TARGETS` overrides the target list, for example `GEMSTONE_IOS_TARGETS=aarch64-apple-ios` for a debug build on a device.

## Generation and Localization

```bash
just generate               # run all generation steps
just generate-models        # regenerate model types from Rust
just generate-stone         # build the iOS Core library and its UniFFI sources
just localize               # regenerate .xcstrings catalogs and typed Localized accessors
```

From the repo root, use `just generate-stone` and `just run-ios` as the default Gemstone/iOS flow. The optional `GemStone` Xcode scheme runs the same generation before the normal app build, for the device or simulator it targets.

## SwiftUI Iteration

For presentation-only SwiftUI work, build the owning package and avoid full app builds for each visual adjustment:

```bash
just build-package Assets
just build-package Components
just build-package PrimitivesComponents
```

For ViewModel or display-model behavior, run the targeted tests; they also build the package:

```bash
just test-package Assets
just test-package GemstoneServices
```

Use `just build` when the change touches app composition, navigation wiring, generated bindings, or code that cannot be validated by a package build.

## Additional Utilities

```bash
just spm-resolve-all
```

## Command Rules

- Use `just` commands for builds and tests, not `xcrun swift test` or ad hoc `xcodebuild` invocations
- Build logs live under `build/DerivedData`
- `just install` installs `swiftformat`, `swiftlint`, and `xcbeautify`
