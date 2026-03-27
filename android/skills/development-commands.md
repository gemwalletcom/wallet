# Development Commands

Use Gradle for Android builds and tests, with `just` wrappers for the repo’s common workflows.

## Setup and Shared Tasks

```bash
just list
just bootstrap
just generate
just generate-models
just localize
```

For local environment prerequisites and GitHub Packages credentials, read `setup.md`.

## Build Commands

```bash
just build
just build-test
just test
./gradlew assembleGoogleDebug
just release
./gradlew clean
```

For Docker packaging, reproducible builds, and artifact verification, read `release-and-verification.md`.

## Test and Quality Commands

```bash
just test
./gradlew test
./gradlew :app:testGoogleDebugUnitTest
./gradlew assembleGoogleDebugAndroidTest
./gradlew connectedGoogleDebugAndroidTest
./gradlew check
./gradlew lint
./gradlew detekt
./gradlew ktlintFormat
```

## Command Rules

- Use Gradle commands for Android build and test execution
- Prefer the local `just` wrappers for the standard debug build, Android test build, and connected-test flows
- Run the narrowest relevant task while iterating, then finish with the appropriate broader validation
- Use `just` for generation, localization, and repo bootstrap workflows
- Keep release/reproducibility workflows separate from normal app iteration and use the dedicated release skill for them
