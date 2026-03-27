# Testing

## Execution Rules

- Run tests through Gradle or the Android `justfile`
- Default commands:
  - `just test` or `./gradlew test` — all unit tests
  - `./gradlew :app:testGoogleDebugUnitTest` — app module only
  - `./gradlew connectedGoogleDebugAndroidTest` — instrumented tests (requires emulator)
- Run the narrowest relevant target while iterating, then finish with broader validation

## Test Structure

### Unit Tests (`src/test/kotlin/`)

- Test business logic, data aggregation, formatting, and calculations
- Use JUnit 4 with standard assertions
- Keep test names short and descriptive
- One behavior per test, small number of assertions

```kotlin
@Test
fun calculatePriceImpact_highSlippage() {
    val result = calculatePriceImpact(inputAmount, outputAmount)
    assertEquals(expected, result)
}
```

### Instrumented Tests (`src/androidTest/kotlin/`)

- Test database migrations, Room queries, and Android-specific behavior
- Use `AndroidJUnit4` runner and `ApplicationProvider` for context
- Use `MigrationTestHelper` for schema migration tests

```kotlin
@RunWith(AndroidJUnit4::class)
class Migration_63_64Test {
    @get:Rule
    val helper = MigrationTestHelper(
        InstrumentationRegistry.getInstrumentation(),
        GemDatabase::class.java
    )

    @Test
    fun migrate() {
        // create db at old version, apply migration, verify schema
    }
}
```

## Test Data

- Use helper functions (e.g., `createAssetInfo()`, `createAssetPriceInfo()`) for reusable test data
- Keep fixtures small, valid, and deterministic
- If a fixture is only used once, an inline literal is fine
- For BigDecimal and numeric edge cases, test boundary values explicitly

## Mocks

- Prefer existing test helpers and builders over ad hoc mocks
- Keep mock setup concise — extract builders when setup is repeated across tests
- Do not mock what you can construct directly

## Formatting

- Use direct assertions for short cases
- Break long setup into multiline formatting when it improves readability
- Avoid explanatory comments in tests
- Clean imports after every modification
