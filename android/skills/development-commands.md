# Development Commands

Use Gradle for Android builds and tests, with `just` wrappers for the repo’s common workflows.

## Setup and Shared Tasks

```bash
just list
just install
just generate
just generate-models
just localize
```

For local environment prerequisites, read `setup.md`.

## Build Commands

```bash
just build
just clean
just run
just start-emulator
just build-test
just test
just test-integration
./gradlew assembleGoogleDebug
just release
```

For release builds, read `release-and-verification.md`.

From the repo root, use `just start-emulator`, then `just run-android` as the default Android run flow.

## Compose Iteration

For presentation-only Compose work, build the owning module first and avoid repeating full app builds for each visual adjustment:

```bash
./gradlew :features:asset:presents:assembleDebug
./gradlew :features:settings:settings:presents:assembleDebug
./gradlew :ui:assembleDebug
```

For ViewModel or display-model behavior, pair the module build with the narrowest matching unit test:

```bash
./gradlew :features:asset:viewmodels:testDebugUnitTest
./gradlew :features:wallets:presents:testDebugUnitTest
```

Use `./gradlew assembleGoogleDebug` when the change touches app composition, navigation wiring, flavor-specific code, generated bindings, or code that cannot be validated by a module build.

## Test Commands

```bash
./gradlew :features:asset:viewmodels:testDebugUnitTest # one library module while iterating
./gradlew :app:testGoogleDebugUnitTest                 # the app module
just test                                              # every module, before a commit
./gradlew assembleGoogleDebugAndroidTest               # compile instrumented tests
just test-integration                                  # tests in integration packages, on Robolectric
./gradlew :app:connectedGoogleDebugAndroidTest          # the app's device-only tests on a running emulator
```

`just test` runs every module's unit tests with `--continue`, so one run reports every failing module. It skips tests in `integration` packages, which `just test-integration` runs. Every Gradle JVM test task depends on `:gemstone:buildGemstoneHost` and fingerprints the host library loaded through JNA, including module-scoped tests. Cargo checks its own incremental state on each invocation; the host library is a Gradle test input, so native-only changes invalidate previous test results. Unchanged native output keeps bindings and tests reusable. Android JNI packaging likewise lets Cargo check its own toolchain and compiler flags rather than duplicating its fingerprint rules in Gradle.

## Core Changes

The Gradle build regenerates the Kotlin bindings and the native libraries on its own whenever Core sources change, so there is no separate generation step for app builds. A local debug build compiles Core for `arm64-v8a` only, which covers the Apple Silicon emulator and current devices; release builds compile `arm64-v8a` and `armeabi-v7a`. Set `GEMSTONE_ANDROID_ABIS` (for example `GEMSTONE_ANDROID_ABIS=arm64-v8a,armeabi-v7a`) to install a debug build on a 32-bit device.

## Command Rules

- Use Gradle commands for Android build and test execution
- Prefer the local `just` wrappers for the standard debug build, Android test build, and connected-test flows
- Run the narrowest relevant task while iterating, then finish with the appropriate broader validation
- Use `just` for generation, localization, and repo bootstrap workflows
- Keep release workflows separate from normal app iteration
