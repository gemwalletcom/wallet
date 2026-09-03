# Testing

## Execution Rules

- Run tests through Gradle or the Android `justfile`
- Default commands:
  - `just test` — unit tests for every module (`testGoogleDebugUnitTest` for `:app` plus `testDebugUnitTest` for library modules); builds the host gemstone library first because gemstone-dependent tests load it through JNA
  - `just test-integration` — instrumented tests for every module (requires emulator)
  - `./gradlew :app:testGoogleDebugUnitTest` — app module only
  - `./gradlew :<module>:testDebugUnitTest` — one feature or shared module
- Run the narrowest relevant target while iterating, then finish with broader validation
- For local instrumented tests, start the emulator from the repo root first with `just start-emulator`, then run `just android test-integration`

## Test Structure

### Unit Tests (`src/test/kotlin/`)

- Test business logic, data aggregation, formatting, and calculations
- Use JUnit 4 with standard assertions
- Keep test names short and descriptive
- One behavior per test, small number of assertions
- Keep setup minimal: prefer shared testkit defaults and override only the inputs that the assertion actually depends on

### Instrumented Tests (`src/androidTest/kotlin/`)

- Test database migrations, Room queries, and Android-specific behavior
- Use `AndroidJUnit4` runner and `ApplicationProvider` for context

## Shared TestKit

Reusable test data factories live in the `:gemcore` `testFixtures` source set (`gemcore/src/testFixtures/kotlin/com/gemwallet/android/testkit/`, one file per type: `AssetMock.kt`, `AssetInfoMock.kt`, `DelegationMock.kt`, and so on). Consumer modules add `testImplementation(testFixtures(project(":gemcore")))`.

- `mockType()` returns a sensible default; expose only the fields tests vary, override one or two at the call site, and use `copy()` for one-offs
- For shared domain types (wallet, account, asset, asset info, prices) never add local `mock*()` or `create*()` helpers in feature or data module tests. If a fixture is missing, add it to the owning module's `testFixtures` and depend on it
- A concrete shape used by more than one test becomes a named fixture (`mockAssetSolanaUSDC()`, `mockAssetMetaData(isSwapEnabled = true)`), not a repeated `mockAsset(chain = ..., symbol = ..., ...)` call. A fixture used once is inlined
- Do not turn a mock helper into a second constructor by passing every field. Do not mock what you can construct directly; use MockK only for interfaces that cannot be constructed
- Prefer the simplest test that proves the behavior: no extra fixtures, mocks, or assertions that do not move the behavior under test

## Formatting

- Use direct assertions for short cases
- Avoid explanatory comments in tests
- Clean imports after every modification
