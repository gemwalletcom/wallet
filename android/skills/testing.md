# Testing

## Execution Rules

- Run tests through Gradle or the Android `justfile`
- Default commands:
  - `just test` — unit tests for every module (`testGoogleDebugUnitTest` for `:app` plus `testDebugUnitTest` for library modules) with `--continue`, so every failing module is reported; Gradle builds and fingerprints the host Gemstone library for JNA, including when running a single module
  - `just test-integration` — every module's tests in `integration` packages, on Robolectric (no emulator); `just test` skips them
  - `./gradlew :app:connectedGoogleDebugAndroidTest` — the app's device-only tests (requires emulator)
  - `./gradlew :app:testGoogleDebugUnitTest` — app module only
  - `./gradlew :<module>:testDebugUnitTest` — one feature or shared module
  - `./gradlew testGoogleDebugUnitTest` on its own runs the app module only: library modules have no flavored test task, so their tests are silently skipped and the build still succeeds. Use `just test` to cover both.
- Run the narrowest relevant target while iterating, then finish with broader validation
- For the app's device-only tests, start the emulator from the repo root first with `just start-emulator`, then run them from `android/`

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

- A test that needs Android — a `Context`, resources, Room, Compose UI or `android.icu` — goes in an `integration` package under the module's `src/test` and runs on the JVM with Robolectric; `just test` skips it and `just test-integration` runs it
- Annotate the class with `@RunWith(AndroidJUnit4::class)` and get a context from `ApplicationProvider`. Robolectric and the AndroidX test runner are on every module's test classpath, and every module runs at the SDK in `gradle/robolectric/robolectric.properties`
- A module whose tests read Android resources, assets or a test activity sets `testOptions.unitTests.isIncludeAndroidResources = true`; plain fixture files go in `src/test/resources`
- Robolectric runs a fake clock: advance it with `shadowOf(Looper.getMainLooper()).idleFor(…)` and the compose `mainClock` instead of waiting on real time

### Device Tests (`app/src/androidTest/kotlin/`)

- Only `:app` may have `src/androidTest`, and only for what Robolectric cannot run, such as the Android Keystore; the build fails for any other module with an `androidTest` source set

## Shared TestKit

A data mock exists once per type and is the same on both apps, so every test reuses it instead of rebuilding the value. Mocks live in the owning module's `testFixtures` source set: shared models in `gemcore/src/testFixtures/kotlin/com/gemwallet/android/testkit/`, database entities and store doubles in `data/services/store/src/testFixtures`. Consumer modules add `testImplementation(testFixtures(project(":gemcore")))` or the owning module's equivalent.

- A generated model gets its `mockType(...)` from `just generate-models` once it is listed under `mocks:` in `core/bin/generate/remote_types.yml`, written to `testkit/GeneratedMocks.kt` (iOS gets the same mock). To mock one, add it to that list and regenerate; never hand-write it. Every field is a parameter with a default, in declaration order: `""`, zero, `false`, `null`, `emptyList()`, the first enum entry (`Chain.Bitcoin`), the epoch for a date, and the nested type's or identifier's own mock. A field the rules cannot fill fails generation and names it; list its type, or override that field in `mocks:` only when the rule would build an invalid value
- A type the generator cannot reach (identifiers, hand-written app types, entities) gets its hand-written `mockType(...)` in the module's one `Mocks.kt`, under the same rules. Behavioural doubles (service fakes, stores, `PasswordStoreMock`) are classes in files of their own
- No named presets (`mockAssetSolanaUSDC()`, `mockWalletMulticoin()`): a test passes the values it asserts on, so every input it depends on is visible at the call site; use `copy()` for one-offs
- A test file declares no `mock*()`, `create*()`, `make*()`, or `build*()` helper of its own for any type, even for one test. Search the owning `testFixtures` first; merge near-duplicate factories into one and delete factories nothing calls
- Do not mock what you can construct directly; use MockK only for interfaces that cannot be constructed
- Prefer the simplest test that proves the behavior: no extra fixtures, mocks, or assertions that do not move the behavior under test

## Formatting

- Use direct assertions for short cases
- Avoid explanatory comments in tests
- Clean imports after every modification
