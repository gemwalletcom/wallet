# Development Commands

Use Gradle for Android builds and tests, with `just` wrappers for the repo’s common workflows.

## Setup and Shared Tasks

```bash
just list
just install-java
just bootstrap
just generate
just generate-models
just localize
```

For local environment prerequisites and GitHub Packages credentials, read `setup.md`.

## Build Commands

```bash
just build
just run
just start-emulator
just build-test
just test
just test-integration
./gradlew assembleGoogleDebug
just release
./gradlew clean
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

## Test and Quality Commands

```bash
just test
just test-integration
./gradlew test
./gradlew :app:testGoogleDebugUnitTest
./gradlew assembleGoogleDebugAndroidTest
./gradlew connectedGoogleDebugAndroidTest
./gradlew check
./gradlew lint
```

## Command Rules

- Use Gradle commands for Android build and test execution
- Prefer the local `just` wrappers for the standard debug build, Android test build, and connected-test flows
- Run the narrowest relevant task while iterating, then finish with the appropriate broader validation
- Use `just` for generation, localization, and repo bootstrap workflows
- Keep release workflows separate from normal app iteration
