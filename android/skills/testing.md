# Testing

## Execution Rules

- Run tests through Gradle or the Android `justfile`
- Default commands:
  - `just test` or `./gradlew testGoogleDebugUnitTest` — Google Debug unit tests
  - `just test-integration` or `./gradlew connectedGoogleDebugAndroidTest` — instrumented tests (requires emulator)
  - `./gradlew :app:testGoogleDebugUnitTest` — app module only
- Run the narrowest relevant target while iterating, then finish with broader validation

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

Reusable test data factories live in `:gemcore`'s `testFixtures` source set (`gemcore/src/testFixtures/kotlin/com/gemwallet/android/testkit/`). Use these instead of duplicating private `create*()` helpers in each test file. If a local helper starts getting reused or reviewed as shared test data, promote it into `testFixtures`.

Hard rules:
- For shared domain types like wallet, account, asset, asset info, and prices, do not add local `mock*()` helpers inside feature or data module tests. Use the existing shared fixture first.
- If a shared fixture is missing, add it to the owning module's `src/testFixtures/...` (`:gemcore` for shared domain models) and depend on that fixture from consumer tests. Do not recreate local adapters like `dbAssetInfo(...)`.
- Do not turn a mock helper into a second constructor by passing every field inline. If a test needs a concrete reusable asset or metadata shape, add a named fixture like `mockAssetHyperCoreHype()` or `mockAssetMetaData(isSwapEnabled = true)` and override only the one or two fields that matter.
- Prefer the simplest test that proves the behavior. Avoid extra fixtures, extra mocks, and extra assertions that do not move the behavior under test.

Consumer modules add: `testImplementation(testFixtures(project(":gemcore")))`

```kotlin
import com.gemwallet.android.testkit.mockAssetPriceInfo

@Test
fun `day period uses 24h change`() {
    val model = buildChartUIModel(
        prices = listOf(ChartValue(timestamp = 1, value = 100.0f)),
        priceInfo = mockAssetPriceInfo(price = 200.0, priceChangePercentage24h = 4.2),
        period = ChartPeriod.Day,
        currency = Currency.USD,
    )
    assertTrue(model.currentPoint!!.percentage.contains("4.2"))
}
```

Rules: `mockType()` returns a sensible default, expose only fields tests vary, use `copy()` for one-offs, one file per type. If a fixture is used once, inline it. Avoid call sites that pass every property into `mockAsset(...)` or construct `AssetMetaData(...)` inline when the same shape can live in shared testkit.

If the same concrete asset shows up in more than one test, add a named fixture for it in testkit like `mockAssetSolana()` or `mockAssetSolanaUSDC()` instead of repeating `mockAsset(chain = ..., symbol = ..., ...)` at call sites.

## Test Data

- Use shared testkit factories for common types (`mockAsset()`, `mockAssetInfo()`, `mockAssetPriceInfo()`, `mockDelegation()`, etc.)
- Use MockK for complex interfaces that can't be constructed directly
- Do not mock what you can construct directly

```kotlin
// gemcore/src/testFixtures/kotlin/com/gemwallet/android/testkit/AssetMock.kt
fun mockAsset(
    chain: Chain = Chain.Bitcoin,
    name: String = "Bitcoin",
    symbol: String = "BTC",
    decimals: Int = 8,
    type: AssetType = AssetType.NATIVE,
) = Asset(id = AssetId(chain), name = name, symbol = symbol, decimals = decimals, type = type)
```

## Formatting

- Use direct assertions for short cases
- Avoid explanatory comments in tests
- Clean imports after every modification
