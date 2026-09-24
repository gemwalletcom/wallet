# Testing

## Execution Rules

- Run tests through Gradle or the Android `justfile`
- Default commands:
  - `just test` — unit tests for every module (`testGoogleDebugUnitTest` for `:app` plus `testDebugUnitTest` for library modules) with `--continue`, so every failing module is reported; Gradle builds and fingerprints the host Gemstone library for JNA, including when running a single module
  - `just test-integration` — the Room DAO and migration tests in `integration` packages, on Robolectric (no emulator); `just test` skips them
  - `./gradlew connectedGoogleDebugAndroidTest connectedDebugAndroidTest` — instrumented tests for every module (requires emulator)
  - `./gradlew :app:testGoogleDebugUnitTest` — app module only
  - `./gradlew :<module>:testDebugUnitTest` — one feature or shared module
  - `./gradlew testGoogleDebugUnitTest` on its own runs the app module only: library modules have no flavored test task, so their tests are silently skipped and the build still succeeds. Use `just test` to cover both.
- Run the narrowest relevant target while iterating, then finish with broader validation
- For local instrumented tests, start the emulator from the repo root first with `just start-emulator`, then run the instrumented tests from `android/`

## Test Structure

### Unit Tests (`src/test/kotlin/`)

- Test business logic, data aggregation, formatting, and calculations
- Use JUnit 4 with standard assertions
- Keep test names short and descriptive
- One behavior per test, small number of assertions
- Keep setup minimal: prefer shared testkit defaults and override only the inputs that the assertion actually depends on

#### Coroutines

- A view-model test sets Main to its own `TestDispatcher` and hands that same dispatcher to the view model, which takes it by injection (see [code-style.md](code-style.md)). `tearDown` cancels `viewModelScope` before `resetMain()`, so nothing survives into the next test
- Keep `runTest`'s default timeout. A test's first call into Core loads the native library, which a CI runner can take well over ten seconds to do, so a tighter global timeout fails healthy tests there
- Drive the queued work with `advanceUntilIdle()`, then assert on `.value`. `coVerify(timeout = …)`, `verify(timeout = …)`, `Thread.sleep`, and poll loops never advance the test scheduler — they only hide a race that a slower CI runner loses later (issue #1271)

### Integration Tests (`src/test/kotlin/…/integration/`)

- Test database migrations and Room queries against a real SQLite on the JVM with Robolectric, at the SDK set in the module's `robolectric.properties`
- Use `AndroidJUnit4` runner and `ApplicationProvider` for context; `MigrationTestHelper` reads the exported schemas from the test assets

### Instrumented Tests (`src/androidTest/kotlin/`)

- Test Android-specific behavior that needs a device, such as the Android Keystore
- Use `AndroidJUnit4` runner and `ApplicationProvider` for context

## Shared TestKit

A mock exists once, beside its type, so every test reuses it instead of rebuilding the value. Test data factories live in the owning module's `testFixtures` source set, one file per type: shared models in `gemcore/src/testFixtures/kotlin/com/gemwallet/android/testkit/` (`AssetMock.kt`, `AssetInfoMock.kt`, `DelegationMock.kt`), database entities and store doubles in `data/services/store/src/testFixtures` (`DbAssetInfoMock.kt`, `StoreTransactionRunnerMock.kt`). Consumer modules add `testImplementation(testFixtures(project(":gemcore")))` or the owning module's equivalent.

- `mockType()` returns a sensible default; expose only the fields tests vary, override one or two at the call site, and use `copy()` for one-offs
- A test file declares no `mock*()`, `create*()`, `make*()`, or `build*()` helper of its own for any type, even for one test. Search the owning `testFixtures` first and extend an existing factory rather than adding a near-duplicate; merge near-duplicate factories into one with sensible defaults and delete factories nothing calls
- A concrete shape used by more than one test becomes a named fixture (`mockAssetSolanaUSDC()`, `mockAssetMetaData(isStakeEnabled = true, stakingApr = 5.0)`), not a repeated `mockAsset(chain = ..., symbol = ..., ...)` call. A shape used once stays an override at the call site
- Do not turn a mock helper into a second constructor by passing every field. Do not mock what you can construct directly; use MockK only for interfaces that cannot be constructed
- Prefer the simplest test that proves the behavior: no extra fixtures, mocks, or assertions that do not move the behavior under test

## Formatting

- Use direct assertions for short cases
- Avoid explanatory comments in tests
- Clean imports after every modification
