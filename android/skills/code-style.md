# Code Style

## Core Rules

- Follow Kotlin coding conventions and existing project patterns
- Use Jetpack Compose for UI work
- Use Hilt for dependency injection
- Implement repository-based data access
- Write unit tests for business logic changes
- Prefer the concrete patterns below over inventing new local conventions

## Repository Expectations

- Use only Gradle for Android builds and tests
- Always run tests for the affected scope
- Do not add unnecessary code comments
- Clean imports after every modification

## Security and Hygiene

- Never commit secrets or API keys
- Keep sensitive local configuration in `local.properties`
- Follow Android security best practices
- Prefer the smallest change that satisfies the requirement

Shared clean-code and code-review principles live in `../../skills/engineering-principles.md`.

## Concrete Patterns

Use current Android code as the baseline for Compose, Hilt, and module structure.

### Activity + Hilt Entry

Inject screen state through `viewModels()` inside an `@AndroidEntryPoint` activity:

```kotlin
@AndroidEntryPoint
class MainActivity : FragmentActivity(), AuthRequester {
    private val viewModel: MainViewModel by viewModels()
    private val walletConnectViewModel: WalletConnectViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            MainContent()
            RootWarning()
        }
    }
}
```

Reference: `android/app/src/main/kotlin/com/gemwallet/android/MainActivity.kt`

### Compose State Collection

Collect view-model state with lifecycle-aware APIs and keep scenes mostly stateless:

```kotlin
@Composable
fun MainContent() {
    val navController = rememberNavController()
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val intent by viewModel.intent.collectAsStateWithLifecycle()
    WalletApp(navController)
}
```

Reference: `android/app/src/main/kotlin/com/gemwallet/android/MainActivity.kt`

### DI Modules

Wire shared services and persistence through Hilt modules:

```kotlin
@InstallIn(SingletonComponent::class)
@Module
object DatabaseModule {
    @Singleton
    @Provides
    fun provideRoom(
        @ApplicationContext context: Context,
        passwordStore: PasswordStore
    ): GemDatabase = Room.databaseBuilder(
        context = context,
        klass = GemDatabase::class.java,
        name = "gem.db",
    ).build()
}
```

Reference: `android/data/services/store/src/main/kotlin/com/gemwallet/android/data/service/store/database/di/DatabaseModule.kt`

### Stateless Feature Scenes

Feature composables should take prepared state and callbacks rather than fetching dependencies internally:

```kotlin
@Composable
fun PerpetualPositionScene(
    perpetual: PerpetualDetailsDataAggregate,
    position: PerpetualPositionDetailsDataAggregate?,
    chartData: List<ChartCandleStick>,
    period: ChartPeriod,
    onChartPeriodSelect: (ChartPeriod) -> Unit,
    onOpenPosition: (PerpetualDirection) -> Unit,
    onClose: () -> Unit,
)
```

Reference: `android/features/perpetual/presents/src/main/kotlin/com/gemwallet/features/perpetual/views/position/PerpetualPositionScene.kt`

### ViewModel Unit Test

Test ViewModels by verifying state emissions. Use JUnit 4, direct construction, and deterministic inputs:

```kotlin
class CalculatePriceImpactTest {
    @Test
    fun highSlippage_returnsExpectedImpact() {
        val result = calculatePriceImpact(
            inputAmount = BigDecimal("100"),
            outputAmount = BigDecimal("95"),
        )
        assertEquals(BigDecimal("5.00"), result)
    }
}
```

Keep tests focused on one behavior. Use helper functions (e.g., `createAssetInfo()`) for reusable test data. See [testing.md](testing.md) for full conventions.
