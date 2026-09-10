# Code Style

Use for any Kotlin or Compose change.
## Core Rules

- Kotlin conventions and existing project patterns; Jetpack Compose for UI; Hilt for dependency injection; repository-based data access
- Follow the shared [comment policy](../../skills/engineering-principles.md#clean-code-principles); clean imports after modifications
- In suspend or flow code, a `catch (e: Throwable)` must rethrow `CancellationException` before mapping to an error state; otherwise a cancelled collection surfaces as a failure (see `InAppUpdateServiceImpl.kt`)
- No blocking store, Room, or gemstone call on the main thread. A synchronous getter reached from a view model constructor, a Hilt provider, or composition throws and kills the process; expose it as a suspend function or flow and collect it. `allowMainThreadQueries()` belongs to migration tests only
- Row models for a list a screen scrolls are built off the main thread (`flowOn(Dispatchers.Default)` before `stateIn`), and every string a row displays is computed in that mapper, not in composition. The main thread only collects finished rows, so a list of any size keeps scrolling while its source keeps emitting
- Prefer the smallest change that satisfies the requirement

## Gradle Dependencies

- A module declares dependencies used by its sources or resources, including XML theme parents. `:app` owns the Material Components dependency for `Theme.Material3.DayNight.NoActionBar`; Compose Material3 does not provide that XML theme. A dependency reachable through another direct dependency's `api` counts as declared (`:ui` exports the Compose stack and `:ui-models` this way); a public class that extends a type from another module exposes it with `api`. `runtime-ktx`-style artifacts arrive transitively, test dependencies belong only to modules with test sources, and `hilt-android` plus `ksp(hilt-compiler)` belong only to modules with Hilt annotations; a presents module that calls `hiltViewModel()` needs `hilt-lifecycle-viewmodel-compose` alone
- Runtime plugins are the exception: `camera-camera2`, `coil-network-okhttp` and `coil-svg` register through service loading and are never imported

## Security and Hygiene

- Never commit secrets or API keys; keep sensitive local configuration in `local.properties`

## Patterns

- **Entry points**: `@AndroidEntryPoint` activities obtain view models with `by viewModels()` and set content once (`app/src/main/kotlin/com/gemwallet/android/MainActivity.kt`)
- **State collection**: `collectAsStateWithLifecycle()` in composables. Scenes stay stateless and take prepared state plus callbacks instead of fetching dependencies (`features/perpetual/presents/src/main/kotlin/com/gemwallet/android/features/perpetual/views/position/PerpetualPositionScene.kt`)
- **Dependency injection**: shared services and persistence come from Hilt modules installed in `SingletonComponent` (`data/services/store/src/main/kotlin/com/gemwallet/android/data/service/store/database/di/DatabaseModule.kt`)
- **Hilt aggregation reads the app's compile classpath**: `:app` declares a direct `implementation` on every module that carries a `@Module`, `@HiltViewModel` or entry point, including each `:features:*:viewmodels` and `:data:coordinators`, even though no app source imports them. Presents reach viewmodels through `implementation`, which never reaches the app's compile classpath. Removing one of these "unused" dependencies breaks DI at build time
- **ViewModel tests**: JUnit 4, direct construction, deterministic inputs, one behavior per test; fixtures from the `:gemcore` testFixtures (see [testing.md](testing.md))

## Room Schema

- Table names are plural `snake_case` (`nft_collections`, `nft_assets`, `nft_assets_associations`)
- Entity fields are Kotlin `camelCase` and the column keeps the property name. `@ColumnInfo` maps only the older entities that already own `snake_case` columns; a new entity does not add it. Reference: `data/services/store/src/main/kotlin/com/gemwallet/android/data/service/store/database/entities/DbContact.kt`
- When an equivalent iOS store model exists, mirror its schema naming instead of inventing Android-only variants; keep one naming scheme within a table

Shared clean-code principles live in `../../skills/engineering-principles.md`.
