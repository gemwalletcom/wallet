# Gemstone Services

Core-owned services live in [`core/gemstone/src/services/`](../gemstone/src/services/) as `<name>/{mod,model,rules,store}.rs` (only the files a service needs); every service and store returns the shared [`GemServiceError`](../gemstone/src/services/error.rs). A service owns the flow (API + rules); each app implements the `Gem*Store` trait over its database or preferences and constructs the service in DI ([`ServicesFactory.swift`](../../ios/Gem/Services/ServicesFactory.swift), Hilt modules under [`android/data/repositories/.../di`](../../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/di/) and [`android/data/coordinators/.../di`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/di/)). Read [How a service is built](#how-a-service-is-built) before adding or changing one.

## How a service is built

[`GemPriceAlertService`](../gemstone/src/services/price_alert/mod.rs) is the reference: it calls the device API, reads a preference, writes a database table and asks the platform for a permission, so it exercises every seam a service can have. New services copy its shape; existing ones move toward it.

### 1. Core owns the flow

`core/gemstone/src/services/<name>/` holds only the files that service needs:

| File | Holds |
| --- | --- |
| `mod.rs` | the `#[derive(uniffi::Object)]` service, its `#[uniffi::constructor]`, and the exported methods — each one a short *API call → rule → store write* sequence |
| `rules.rs` | the decisions, as pure functions with unit tests |
| `store.rs` | the `#[uniffi::export(rust, foreign)]` trait the apps implement |
| `model.rs` | the `uniffi::Record`/`uniffi::Enum` types the service returns |
| `error.rs` | only when [`GemServiceError`](../gemstone/src/services/error.rs) cannot express a case |

The service holds `Arc`s of other Core services and of store traits — never an app type:

```rust
#[derive(uniffi::Object)]
pub struct GemPriceAlertService {
    api: Arc<GemDeviceApiClient>,
    preferences: Arc<GemPreferencesService>,
    store: Arc<dyn GemPriceAlertStore>,
    device: Arc<GemDeviceService>,
    permissions: Arc<dyn GemNotificationPermissions>,
}
```

A method reads remote and local state, asks a rule what changed, and writes only that:

```rust
pub async fn sync(&self, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
    let remote = self.api.client.get_price_alerts(...).await.map_err(GemApiError::from)?;
    let local = self.store.get_price_alerts(asset_id).await?;
    let changes = rules::reconcile(local, remote);
    if changes.delete_ids.is_empty() && changes.alerts.is_empty() {
        return Ok(());
    }
    self.store.update_price_alerts(changes.alerts, changes.delete_ids).await
}
```

Everything that decides belongs in `rules.rs` with a test that fails if the rule flips. A failure is either impossible (a rule with a built-in default), surfaced (returned as `GemServiceError`), or recorded through `services::failures::record` — never swallowed.

**There is no `*RulesService`.** A domain has one service, and a rule that domain's service owns is a method on it. When a rule has to be reachable from code that can never hold an injected service — a SwiftUI `Identifiable`, a Compose scene, a value-type extension, an enum property, a `Gem*Store` adapter (Core constructs the service *from* it) — the caller that *can* hold the service resolves it and passes the answer down — a view model asks `GemExplorerService` for the link and hands the URL to the Compose scene or the UI-model factory; the scene never reaches for a service. A constructible service is **not** a licence to hold one at file scope. `private let addressService = GemAddressService()` / `private val assetConfig = GemAssetConfigService()` above a value-type extension is a hidden global: nothing can substitute it, and the extension reaches outward instead of being handed what it needs. There are around fifteen of these on each platform (`gemcore/ext/*`, `GemstonePrimitives/Sources/Extensions/*`) and they are all going. The fix is the same every time — the caller that can hold the injected service resolves the answer and passes it down, as `FeeRateUIModel` now takes `totalFee` rather than computing it. Where an extension genuinely has no caller that can hold a service, that is a signal the logic belongs on the feature's service and the extension should not exist. The I/O service never carries a second copy of a rule, and a second *exported* object is not the answer either: a feature's clients see exactly one type, its service. `Explorer` and `GemStakeConfig` were both dropped from the FFI for this reason — `Explorer` stays as a plain Rust struct that `GemExplorerService` uses internally, and stake's rules moved onto `GemStakeService`.

Know the cost before choosing it. A rule reachable only through an I/O service cannot be exercised by an app test without mocking it, and a mocked rule is a premise, not a check — so the rule's real test has to exist in `rules.rs`, mutation-checked, before the app-side one is downgraded. Moving stake's rules onto `GemStakeService` traded three Android rule tests for that: `AssetInfoUIModelFactoryTest` now states what the service reports and asserts only the formatting, because Core's `stake::rules` tests already cover which chain counts frozen versus staked.

### 2. Pick the store the value belongs in

| What the service needs | Trait | Shape | iOS | Android |
| --- | --- | --- | --- | --- |
| rows in the database | one `Gem<Name>Store` per service ([example](../gemstone/src/services/price_alert/store.rs)) | `async`, every method returns `Result<_, GemServiceError>` | GRDB store under [`GemstoneServices/Sources/Stores/`](../../ios/Packages/GemstoneServices/Sources/Stores/) | Room DAO under [`data/repositories/.../gemstone/`](../../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/stores/) |
| a value the user set | [`GemPreferencesStore`](../gemstone/src/services/preferences/store.rs) through `GemPreferencesService` | sync; `get` returns `Option<String>` and **cannot fail** | `GemstonePreferencesStore` over `UserDefaults` | `GemstonePreferencesStore` over `SharedPreferences` |
| the same, per wallet | `GemWalletPreferencesStore` through `GemWalletPreferencesService` | sync, keyed by `WalletId` | same file layout | same file layout |
| a secret | [`GemSecureStore`](../gemstone/src/services/preferences/store.rs) | sync; **every read can fail** | `GemstoneSecurePreferencesStore` over the Keychain | `TinkGemPreferences` over Tink |
| something only the OS can do | a foreign trait of its own (`GemNotificationPermissions`, `GemStreamConnection`) | whatever the platform needs | app class | app class |

- One trait per table. A second trait over the same rows is how the two apps drift apart.
- A new preference is a `const` key plus typed accessors on `GemPreferencesService` — single-word keys (`_` separates the settings hierarchy in environment variables), never a raw key string in an app.
- The preference read is infallible on purpose: getters return plain values, so neither app writes `try?`/`runCatching` around them. Secure reads are fallible and their failure must propagate — a swallowed secure read regenerates identity or loses a key.
- Store methods follow the vocabulary in [Conventions](#conventions): `get_*`, `is_*`, `set_*`, `save_*`, `add_*`, `update_<items>(items, delete_ids)`, `delete_*`, `clear*`.

### 3. Each app implements the store — and nothing else

iOS, `ios/Packages/GemstoneServices/Sources/Stores/<Name>Store.swift`, class `Gemstone<Name>Store`, converting with `.json()` and `Primitives.<T>(_:)`:

```swift
public final class GemstonePriceAlertStore: GemPriceAlertStore, @unchecked Sendable {
    private let store: PriceAlertStore

    public func updatePriceAlerts(alerts: [Gemstone.PriceAlert], deleteIds: [String]) async throws {
        try store.diffPriceAlerts(deleteIds: deleteIds, alerts: alerts.map { try Primitives.PriceAlert($0) })
    }
}
```

Android, `android/data/repositories/.../gemstone/<Name>Store.kt`, class `Gemstone<Name>Store`, converting with `toJson()` and `decodeJson()`:

```kotlin
class GemstonePriceAlertStore(
    private val priceAlertsDao: PriceAlertsDao,
) : GemPriceAlertStore {

    override suspend fun updatePriceAlerts(alerts: List<String>, deleteIds: List<String>) {
        priceAlertsDao.update(alerts.map { it.decodeJson<PriceAlert>().toRecord() }, deleteIds)
    }
}
```

The two adapters are mirrors: same methods, same conflict behaviour (upsert where the other upserts), same "write only rows whose values differ" rule, same treatment of a missing row. A difference between them is a bug in one of them, not a platform choice. Types that cross as JSON (`Asset`, `Account`, `Wallet`, `SimulationResult`, …) arrive as `String` typealiases and are decoded at the adapter boundary, never deeper in the app.

### 4. Construct it once

iOS builds the store and the service in [`ServicesFactory.swift`](../../ios/Gem/Services/ServicesFactory.swift) and publishes the service through `AppResolver` and an `@Entry` on the environment:

```swift
let gemPriceAlertStore = GemstonePriceAlertStore(store: storeManager.priceAlertStore)
let priceAlertService = Gemstone.GemPriceAlertService(
    api: gemDeviceApiClient,
    preferences: preferencesService,
    store: gemPriceAlertStore,
    device: deviceService,
    permissions: GemstoneNotificationPermissions(service: pushNotificationEnablerService),
)
```

Android provides the store and the service from one Hilt module ([`PriceAlertsModule`](../../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/di/PriceAlertsModule.kt)):

```kotlin
@Singleton @Provides
fun provideGemPriceAlertStore(priceAlertsDao: PriceAlertsDao): GemPriceAlertStore = GemstonePriceAlertStore(priceAlertsDao)

@Singleton @Provides
fun provideGemPriceAlertService(...): GemPriceAlertService = GemPriceAlertService(api, preferences, store, device, permissions)
```

### 5. Call it from the app — the service itself on iOS, a case on Android

**iOS: the view model holds the Core protocol.** Nothing sits in between.

```swift
@Observable
@MainActor
public final class PriceAlertsSceneViewModel: Sendable {
    private let priceAlertService: any GemPriceAlertServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol
    public let query: ObservableQuery<PriceAlertsRequest>

    public init(
        priceAlertService: any GemPriceAlertServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
    ) {
        self.priceAlertService = priceAlertService
        self.preferencesService = preferencesService
        isPriceAlertsEnabled = priceAlertService.isEnabled()
        query = ObservableQuery(PriceAlertsRequest(), initialValue: [])
    }
}
```

The screen reads the service out of the environment and passes it in — `@Environment(\.priceAlertService)`, built once in `ServicesFactory`. **Do not write an app service around a Core service.** The distinction is which way the dependency points: a class that *implements* a Core trait is an adapter and is required (the `Gemstone*Store` classes, `GemstoneNotificationPermissions`, the WalletConnect signer); a class that *calls* a Core service and re-exposes it is a wrapper, and the view model should hold the protocol instead. The wrappers still in the app are listed under [iOS](#ios) below, and they go.

**Android: the view model holds cases, never a repository.** A case is three files: the interface the screen asks for, the implementation that holds the Core service, and the view model that injects the interface.

`gemcore/.../application/pricealerts/cases/SetPriceAlertsEnabled.kt` — the case:

```kotlin
interface SetPriceAlertsEnabled {
    suspend operator fun invoke(enabled: Boolean)
}
```

`data/coordinators/.../pricealerts/PriceAlertsEnabledCoordinator.kt` — the implementation, holding nothing but the Core service and the signal that makes the read re-emit:

```kotlin
class PriceAlertsEnabledCoordinator(
    private val priceAlertService: GemPriceAlertService,
) : GetPriceAlertsEnabled, SetPriceAlertsEnabled {

    private val changes = MutableSharedFlow<Unit>()

    override fun isPriceAlertsEnabled(): Flow<Boolean> = changes
        .onStart { emit(Unit) }
        .map { priceAlertService.isEnabled() }

    override suspend fun invoke(enabled: Boolean) {
        priceAlertService.setEnabled(enabled)
        changes.emit(Unit)
    }
}
```

Hilt builds it once and binds both interfaces to it, so the view model asks for the case it needs and nothing else:

```kotlin
@Provides @Singleton
fun providePriceAlertsEnabledCoordinator(priceAlertService: GemPriceAlertService) = PriceAlertsEnabledCoordinator(priceAlertService)

@Provides
fun provideGetPriceAlertsEnabled(coordinator: PriceAlertsEnabledCoordinator): GetPriceAlertsEnabled = coordinator

@Provides
fun provideSetPriceAlertsEnabled(coordinator: PriceAlertsEnabledCoordinator): SetPriceAlertsEnabled = coordinator
```

**Two shapes, and the name says which:** `*Impl` is one case with no state — it forwards to Core, or returns the Room `Flow` the database already makes reactive (`GetAssetPriceAlertState`). `*Coordinator` implements the read *and* the write for one subject and owns the signal between them; it is the right shape when Core answers with a point read that screens must observe. `PriceAlertsEnabledCoordinator` is one (the enabled flag is a preference, so `isEnabled()` cannot be observed) and `AppUpdateCoordinator` is another (the update offer has no row behind it).

Prefer the coordinator over refreshing state in each view model: `setPriceAlertsEnabled` has a second writer — `IncludePriceAlertImpl` turns alerts on when one is added — and a screen holding its own copy goes stale the moment someone else writes. Routing both directions through one object is what keeps every observer correct. iOS gets away with the simpler thing (`isPriceAlertsEnabled = priceAlertService.isEnabled()` after its own `setEnabled`) because its screens re-read on appear. If Core ever publishes preference changes as a stream, every coordinator of this kind collapses back into two stateless `*Impl`s.

**A case composing other cases is always fine** — `SyncAssetPriceAlertsImpl` holds `HasAssetPriceAlerts` and `UpdatePriceAlerts` — that is how a flow is assembled; what a case must not hold is a repository.

`features/settings/price_alerts/.../PriceAlertViewModel.kt` — the screen's view model, which injects the cases and never the service or a repository:

```kotlin
@HiltViewModel
class PriceAlertViewModel @Inject constructor(
    private val getPriceAlertsEnabled: GetPriceAlertsEnabled,
    private val setPriceAlertsEnabled: SetPriceAlertsEnabled,
) : ViewModel() {

    val priceAlertEnabled = getPriceAlertsEnabled.isPriceAlertsEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun togglePriceAlerts(enable: Boolean) = viewModelScope.launch {
        setPriceAlertsEnabled(enable)
    }
}
```

A case may compose other cases ([`SyncAssetPriceAlertsImpl`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/pricealerts/SyncAssetPriceAlertsImpl.kt) calls `HasAssetPriceAlerts` and `UpdatePriceAlerts`), and it holds the `Gemstone*Store` adapter when the screen needs an observed read — never the DAO, and never a repository for data a Core service owns. The repositories left in the graph are legacy and shrink as services land; `SessionRepository` is the one still in wide use, and Core's `GemWalletSessionService` is replacing it.

**A screen asks one thing, and the thing it asks is a Core service.** A view model that takes a `Gem*Service` *and* a rule service and combines them is doing the feature's job in the view layer — but the answer is not an app-side wrapper around both. Put the decision in Core and let the screen call it.

Rules belong in `gemstone`, not in an app class wrapping several services — `GemGateway.check_node` owns the node network-id check `AddNodeSceneViewModel` used to assemble. One constraint: `GemNodeService` cannot hold the gateway, because the gateway's transport picks node URLs *through* `GemNodeService`. Check where a service sits in that graph before giving it a new collaborator.

The same rule with each platform's noun, for the cases Core genuinely cannot answer:

- **iOS** — a feature service in `Features/<Feature>/Sources/Services/`, built in the app layer and passed into the view model's initializer. A feature package cannot read the app's `@Environment` service keys, so the service reaches it through the view model it is handed, never through an ambient registry.
- **Android** — a case in `gemcore` `application/<area>/cases/` with its implementation in `data/coordinators/<area>/`, injected into the view model by Hilt.

Still open: `AddNodeViewModel` on Android is the untouched twin — `NodeStatusService`, `AddNodeCase`, `SetCurrentNodeCase` and a raw `GemChainService` for the same network-id check, which `GemGateway.check_node` now answers. It belongs to the nodes work in flight.

**An app service holds Core services, not the tables Core owns.** Reaching into a `Gem*Store`'s table from the app is a second read path the owner cannot see — the same violation as a case taking a Room DAO. The exception is a table Core has no concept of: the recent-activity list is the app's own, so `RecentActivityStore` (iOS) and `RecentAssetsService` (Android) are the platform's query layer and stay.

Android already follows this: no view model holds a `Gemstone*Store`, and the fifty that do are cases in `data/coordinators`, which is the documented home for an observed read. iOS's confirm flow does not, and the reason is structural rather than careless — every method on a Core service crosses UniFFI as `async`, and the confirm screen builds its first state inside a synchronous initializer, so it reads the store directly instead:

| Reader | Reads | Owner it bypasses |
| --- | --- | --- |
| `ConfirmSimulationService.simulationAssets` | `AssetStore.getAssets` | `GemAssetsService` |
| `FeeAssetProvider` | `AssetStore.getAssetData`, `getAssetsData` | `GemAssetsService` |
| `TransferMetadataProvider` | `BalanceStore.getBalance`, `PriceStore.getPrices` | `GemBalanceService`, `GemPriceService` |
| `ConfirmService.addressName` | `AddressStore.getAddressName` | `GemNameService` |

The fix has a precedent in the same codebase: `GemWalletStore.get_wallets` and `get_wallet` are **synchronous** trait methods, so `GemWalletService` answers a session lookup without `await`. The reads above are all point reads of a single row or a short list, so they can be synchronous the same way — the trait method loses `async`, the service exposes it synchronously, and the confirm flow asks the owner. Do that before the confirm seam, because the seam's own migration assumes those reads come from Core.

`DeveloperViewModel` holds five stores to dump and reset them; it is a developer screen and stays as it is.

**Observed reads.** Core has no observation primitive, so a screen that must update as rows change observes the app's own database — iOS with `ObservableQuery` over a GRDB request, Android with a Room `Flow` returned by the case. Everything else — writes, remote sync, point reads, every decision — goes through the service.

**Tests.** iOS mocks the protocol from [`GemstoneServices/TestKit`](../../ios/Packages/GemstoneServices/TestKit/); Android fakes the case interface, or mocks `Gem*Service` with MockK, using fixtures from `gemcore` `testFixtures`. Never mock a constructible service (`GemAssetConfigService`, `GemChainService`, …) — construct the real one, or the test asserts the mock. Never fabricate I/O to reach a rule either: an offline `AlienProvider`, in-memory preference and secure stores and empty row stores, stood up so a test can touch rules that use none of them, is always the wrong answer — pass the answer in from the caller that owns the service, or mock the service and state the premise plainly. Neither app tests a rule that lives in Core — that test belongs in `rules.rs`.

### Done means

- Core has the flow, the rules and their tests; the app code it replaced is deleted in the same commit.
- Both apps implement the same store trait the same way, and both build and pass their suites.
- No app-side copy of a Core decision, no raw preference keys, no swallowed store failure, and no app service reading a table a `Gem*Store` owns.
- Nothing was added to reach it: iOS injects the protocol, Android calls a case — no wrapper service, no repository. A feature service or case that gathers several collaborators for one screen is not a wrapper; a class that forwards one call is.
- No `private let`/`private val` holding a `Gem*Service` at file scope. A service comes from the initializer or from Hilt, so a test can substitute it.
- The service is listed in [Core services](#core-services) with its store and both adapters, and its line in the plan below is removed.

## App services

What stays on the app side, because it is a platform concern with no Core counterpart:

| Service | Notes |
| --- | --- |
| [`AppService/RateService`](../../ios/Packages/FeatureServices/AppService/RateService.swift) | App Store review prompt |
| [`AppService/AppLifecycleService`](../../ios/Packages/FeatureServices/AppService/AppLifecycleService.swift) | Scene phase orchestration of observers |
| [`ConnectionStatusService`](../../ios/Packages/FeatureServices/ConnectionStatusService) | Connectivity |
| [`SystemServices`](../../ios/Packages/SystemServices) | Connectivity, image gallery, local store |

Every other app service is gone and its callers hold the Core service. Every `Gem*Service` Core exports is referenced by both apps — there is no Core service without an app consumer, and no app missing one.

## Remaining

- **iOS `Packages/GemAPI`** — one endpoint, one caller: `GemPriceWidget` reads asset prices with it. It stays. Routing the widget through Core would link the Rust library into an app extension that runs under a tight memory budget and makes a single GET, so the trade is wrong; nothing else in the app or the feature packages depends on the package. Android's equivalent is already down to the alien provider itself: `data/services/native-provider` is `NativeProvider` plus its cache, named after the trait it implements the way iOS's `NativeProviderService` package is.
- **iOS `Packages/GemstonePrimitives` is 2,656 lines and mostly load-bearing.** A sweep for declarations with no reader outside the package found twelve, of which six were genuinely dead and are gone (`Chain.tokenActivateFee`, `AssetId.getAssetType`, `GemKeystore.mapToPreview`, `StakeChain.supportRedelegate`/`supportWithdraw`/`supportClaimRewards`); the other six are used inside the package. What is left is the JSON bridge conformances, the chain and stake config accessors, and typed wrappers over Core's JSON-string APIs — it shrinks when primitives stop crossing as JSON strings, not before, so treat the number as a consequence of the confirm seam rather than a target of its own.
- iOS `Primitives` keeps hand-written views of Core types (`GasPriceType`, `FeeRate`, `Fee`, `FeeSelection`, `CustomFeeEstimate`, `TransferAmount`, `BalanceRequirement`, and ids such as `WalletId`/`TransactionId`/`AssetId`). They stay: they are typed views bridged once at the seam, not a second source of truth.

## TODO — finish Core as the single owner of logic

Every app service forwards to a Core service or is platform glue, and the rules that used to live in view models live in `gemstone` with unit tests. Every exported `Gem*Service` has a consumer on both platforms, all 26 store traits have an adapter on both (Android's node store is the one gap, owned by the nodes work in flight), and Android has no repositories left.

What is left is below, in priority order: the confirm seam is the last large migration, then the per-feature reads, then the platform-side gaps. Work it top to bottom inside each section; each item lands with the app-side code it replaces deleted, a Core test that would flip if the rule flipped, and its line removed from this file in the same commit. Presentation and localization work goes last. When the list is empty, audit again and write a new one.

### Not yet migrated — every place that moves, both platforms

Each entry is one batch. Work it end to end: put the rule in Core with a test that would flip if the rule flipped, wire both platforms onto it, delete the app code it replaces, then remove the entry. The iOS file is the one that owns the logic today; the Android column is its counterpart, which is usually already thinner — converge on whichever reading is better and say which in the commit.

| # | Service | iOS | Android | What moves |
|---|---|---|---|---|
| 1 | Wallet creation and import | [`WalletService`](../../ios/Packages/GemstoneServices/Sources/Wallet/WalletService.swift) (95) — `nextWalletIndex`, `createWallet`, `importWallet`, `sorted(wallets:)`, `delete`, `setup(chains:)`, `acceptTerms`, `migrateV3Keystores` | [`ImportWalletService`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/wallet_import/services/ImportWalletService.kt) (54), `application/wallet/cases/*` (10 cases), `CreateWalletViewModel`, `ImportViewModel` | `GemWalletService` already validates the import; what is app-side is the ordering (validate → keystore → store → session), the wallet sort and the next-index rule. All three are Core's. iOS holds `ObservablePreferences` for one flag (`isAcceptTermsCompleted`) that `GemPreferencesService` owns. |
| 2 | Current wallet session | [`WalletSessionService`](../../ios/Packages/GemstoneServices/Sources/Wallet/WalletSessionService.swift) (47) + `WalletSessionManageable` | [`SessionCoordinator`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/session/SessionCoordinator.kt) (103), `application/session/cases/*` (6 cases) | Both wrap `GemWalletSessionService` and add the same thing: "the current wallet, observed". Android's coordinator is the shape — one owner of the observed session — but the wrapping is still app-side on both. Decide whether `currentWallet`/`getWallets`/`getWallet` are re-exports worth keeping at all. |
| 3 | Confirm | [`ConfirmService`](../../ios/Features/Transfer/Sources/Services/ConfirmService.swift) + `ConfirmSimulationService`, `ConfirmTransferInputProvider`, `FeeAssetProvider`, `TransferMetadataProvider`, `TransferTransactionProvider` | `coordinators/confirm/*` (`BuildConfirmPropertiesImpl`, `ChainFeeAssetProvider`, `ConfirmTransactionImpl`, `GetFeeAssetsImpl`), `ConfirmViewModel` | The seam below. Also: `ConfirmService` holds `RecentActivityStore` — a table Core does not own, read from a confirm screen. Move that read behind the service that owns it. |
| 4 | Hyperliquid streaming | [`HyperliquidObserverService`](../../ios/Packages/GemstoneServices/Sources/Perpetual/HyperliquidObserverService.swift) (134) | [`HyperliquidObserverService`](../../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/perpetual/HyperliquidObserverService.kt) (113) + `PerpetualStreamConnection` | Two 120-line copies of the same reconnect/subscribe/account-mode dance over `GemPerpetualStreamService`. Only the socket is platform. See **Hyperliquid streaming belongs in Core** below. |
| 5 | Device observation | [`DeviceObserverService`](../../ios/Packages/GemstoneServices/Sources/Device/DeviceObserverService.swift) (37) + `DeviceSyncPreflight` (16) | `services/DeviceSyncPreflight.kt` (11), `SyncService` | "Register if needed, then watch subscriptions" is one rule; `GemDeviceService.needs_sync` already answers the diff. The observer wiring is what is left. |
| 6 | Device platform values | [`DevicePlatform`](../../ios/Packages/GemstoneServices/Sources/Device/DevicePlatform.swift) (59) | [`DevicePlatform`](../../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/device/DevicePlatform.kt) (145) | Both implement the `GemDevicePlatform` foreign trait, which is correct — but Android's is 145 lines to iOS's 59. The difference is app-side logic that belongs in `GemDeviceService`, not in the adapter. |
| 7 | Biometry and the app lock | [`BiometryAuthenticationService`](../../ios/Packages/GemstoneServices/Sources/Keystore/BiometryAuthenticationService.swift) (64) | `SystemAuthPolicy.kt`, `LockTimer.kt`, `application/security/cases/AuthRequester.kt`, `MainViewModel` (240) | The policy — whether auth is required, the lock interval, whether the interval has elapsed, what a retryable prompt error is — is Core's. Only the platform prompt (`LAContext`, `BiometricPrompt`) stays. **The elapsed-interval half is done**: `LockTimer` holds the pause timestamp and asks `GemSecurityService.shouldRelock`, so it no longer reimplements the rule. What is left is the retryable-prompt-error classification and Android's spread across `SystemAuthPolicy`, `AuthRequester` and `MainViewModel`. |

### Surveyed divergences — the consolidation backlog

Rows were verified against the code and adversarially re-checked; **V5, N4, N6, V7 and T4 were examined and found wrong — do not re-add them**. Read each row's action, not just its description.

Found by reading both platforms side by side. Ranked within each group by value: a difference the **user or an attacker can observe** outranks pure duplication. Each row is a batch; delete the row when it lands.

#### Security and key material — do these first

| # | What | iOS | Android | The difference |
|---|---|---|---|---|
| S1 | Keystore password scope | `Stores/KeystorePasswordStore.swift:14-16` drops the `walletId` and returns one global Keychain secret | `stores/KeystorePasswordStore.kt:12-28` reads per wallet, but on create it writes **the same global secret** under the wallet id as a compatibility alias | Core's `GemKeystorePassword` contract is per-wallet and neither platform honours it — checked, and the survey's claim that Android limits the blast radius is wrong, the secret is identical across wallets on both. Still worth deciding: either make it genuinely per-wallet (needs a re-encrypt migration) or drop `wallet_id` from the trait so the contract stops implying something untrue. |
| S3 | Biometric gate placement | Keychain ACL on the item itself (`LocalKeystorePassword.swift:100-107`), so every secret read prompts | A separate UI call at each call site (`ConfirmScreen.kt:150`, `WalletConnectReviewScene.kt:49`, `routes/Wallet.kt:51`); `PasswordStore` itself is unauthenticated | On Android the gate is advisory — any new caller reaching `PasswordStore` bypasses it. Core should mark which operations require authentication. |
| S5 | WalletConnect replay suppression | `MessageTracker.swift:5-15` + `WalletConnectorService.swift:202-208` reject a repeated request id | none — `ActiveWalletConnectRequest.kt:23-39` overwrites, `WCRequestViewModel.kt:71-73` replaces | A relay retry re-presents the same signing prompt on Android. `handle_request` should own the seen-id set. |
| S6 | Origin rejection | `WalletConnectorService.swift:153-164` switches in Swift, plus a nil-verify-context rejection at `:116-119` | `WalletConnectOriginVerifier.kt:16-34` re-implements `isScam`, applied at three call sites | Core already rejects `Invalid \| Malicious` for proposals (`wallet_connect/mod.rs:104-110`) but not requests, so both apps rebuilt it — three copies of one predicate. |
| S8 | Privacy lock | Setting + `shouldCoverScreen` rule (`LockSceneViewModel.swift:79-87`), overlay window | none; nearest is an unrelated `FLAG_SECURE` toggle | A whole app-lock setting exists on one platform. The cover predicate is Core's; the overlay/`FLAG_SECURE` is platform. |
| S9 | WalletConnect one-click auth (SIWE) | none | `WCAuthViewModel.kt:238-323` builds issuer, payload and message; method table hand-written in `Namespace.kt:9-75` | An entire signing surface — including *what the user is asked to sign* — exists on Android only, with its rules in UI code. |

#### Notifications, device and streaming

| # | What | iOS | Android | The difference |
|---|---|---|---|---|
| N1 | Permission prompt | `NotificationPermissions.swift:13-15` + `PushNotificationService.swift:25-40` — granted → register, undetermined → prompt, denied → Settings | `NotificationPermissions.kt:12-22` — anything but "already enabled" jumps to system Settings; the runtime prompt lives in the UI gate instead | Partly fixed: Core refuses to enable when permission is refused, and the Android gate asks before acting instead of after, so a fresh install no longer gets Settings and the prompt at once. Still open: the adapter opens Settings for a user who has never been asked, because it holds an application Context and cannot tell that from a denial — it needs an activity-scoped requester, and Core should own the three-state decision rather than a single boolean. |
| N5 | Lazy token recovery | none | `DevicePlatform.kt:109-119` re-requests a missing token from inside the trait call, then re-enters Core | Android self-heals a lost token, iOS registers with an empty one. Android's re-entry runs against Core's own `sync_lock`. |
| N9 | Build capability flag | none | `NotificationsAvailable.kt` per flavour, consumed in six places | Core cannot reason about "this build can never have a token". The dead `RequestPushToken.initRequester` should go with it. |

#### Timers, reconnects and refresh cadence — all silently drifting

| # | What | iOS | Android | The difference |
|---|---|---|---|---|
| T2 | Socket keepalive | none | `WebSocketConnection.kt:42,45,113` — 30 s OkHttp ping | iOS sits on a half-open socket until a read fails, so price and candle streams can stall silently. |
| T3 | Swap quote refresh | `SwapScene.swift:89` 30 s, debounce 250 ms, keeps retrying after an error | `RequestSwapQuotes.kt:15-16` 30 s / 500 ms, **breaks the loop on error** | Two hardcoded cadences, a debounce that has already drifted (also 250 vs 500 ms for name resolution), and different behaviour after a failed quote. |
| T5 | In-app polling | four hardcoded timers (positions 1 min, charts 1 min, asset 5 min, activity 5 min) | none — pull-to-refresh only | iOS polls on top of the live socket; Android trusts the stream. Core's own `PRICES_UPDATE_INTERVAL_SECONDS` is exposed to both and used by neither. |
| T6 | Pending-transaction tracking | started twice, never stopped | `TransactionStateTracker.kt:27-33` start/stop on process lifecycle | iOS keeps polling while backgrounded. |
| T7 | Live price subscription | `PriceUpdater.swift` called from the asset screen **and both swap legs** | `SyncAssetInfoImpl.kt:35` only | Swap-screen fiat values only move on Android if something else subscribed the asset. Core should decide when prices are subscribed; then `PriceUpdater.swift` goes. |
| T8 | Perpetual connect gate | Core `shouldConnectPerpetuals` / `syncEnablement` | `ObservePerpetualWallet.kt:18-23` re-implements it over Android's own `UserConfig` | Android never gets `clear_markets`/`sync_markets_if_stale` from the connect path. |
| T11 | Stream registration gate | reads `isRegistered()`, syncs only if unregistered | syncs unconditionally on every foreground | Same intent, two preconditions. |
| T12 | App-start ordering | `OnstartService` + `RootSceneViewModel`, device sync raced, wallet setup gated on unlock | `SyncService.kt:16-24` sequential, no unlock gate | `GemAppStartService.run()` covers only config/banners/assets; everything else is bolted on per platform in a different order. |

#### Screen rules the two apps answer differently

| # | What | iOS | Android | The difference |
|---|---|---|---|---|
| V6 | Recents filters | Core's full `actionFilters` | `HasBalance` only, or a downgraded set | Android's swap/send recents include hidden assets and fully-staked balances. |
| V10 | Stake row | `isStakeEnabled \|\| staked balance > 0` | `type == NATIVE && StakeChain.isStaked(chain)` | An asset with staking disabled still shows a Stake row on Android, with an APR string iOS never shows. |
| V14 | Reserved-fee hint | shown when the typed amount equals max | only when the Max button was pressed | Typing the max by hand shows the note on iOS only. |
| V15 | Pinned section | `metadata.isPinned` | `pinned && balanceEnabled` in the picker, plain `pinned` on home — inconsistent with itself | A pinned-but-hidden asset lands in different sections. |

Found while landing the batches above, not yet fixed:

- **The reconnect cap was a real decision, not a default.** Android's 30 s was set deliberately (`d1cdb74`, "Cap stream reconnect backoff at 30s") so the price stream resumes within half a minute; iOS's 60 s was the untouched default from the initial import. Core took 30 s. Worth knowing before "restoring" 60 s.
- **Deleting a wallet was leaking v3 secrets, and the backlog description was wrong about why.** S10 said "Core already deletes the keystore, so neither app should". Core deleted the **v4** file only; the legacy **v3** file was app-side on both platforms, so removing the app calls without moving v3 cleanup into Core would have orphaned WalletCore secrets on installs that never finished migrating. The two apps also disagreed on how to find it: iOS scanned by name, suffix and JSON `id`; Android checked one exact filename, so it would leak a v3 file written under WalletCore's own `UTC--…--<uuid>` name. Core owns the locator now, content-matching gated on a successful keystore parse because Android's keystore base directory is the app's whole data directory.
- **Stake push notifications go to different places.** iOS `case .stake: break` only selects the wallet tab, behind a TODO; Android opens the asset via `prepareAssetRoutes`. A navigation decision rather than payload decoding, so it was left alone — but it is a real behavioural difference and wants a decision.
- **Dead typeshare models.** `PushNotificationAsset`, `SwapAsset`, `WalletAsset`, `Transaction` and `Reward` are unused by both apps now (only `PushNotificationTypes` survives, in one Android test). Dropping their `#[typeshare]` attributes would delete the generated models, but needs `just generate-models` on both platforms.
- **Keystore migration tests collide under concurrent `cargo test`.** `migrate_v3_*` use fixed `std::env::temp_dir()/gemstone_migration_<name>` paths, so two runs in the same checkout fail spuriously. Hardening candidate, and worth knowing before trusting a red run in a shared checkout.
- **Count the copies before adding one.** The candle interval mapping was described here as two app-side copies; it was three — `candle_interval` in `gem_hypercore` has always driven the REST candlestick fetch. Writing a fresh rule would have made a fourth, free to drift from the fetch path, and the subscribed interval must equal the fetched interval or the chart never updates. Check `core/crates/` before writing a rule that sounds new.
- **`GemGateway.get_perpetual_candlesticks` unwraps a parsed period** (`gateway/mod.rs:169`). Unreachable today because its only caller passes `period.as_ref()`, but it is an `unwrap` on a value arriving as a `String`.
- **Same button, two strings, and a product call to make.** iOS says "Use Minimum Amount" (`swap_use_minimum_amount`); Android says "Minimum amount" (`stake_minimum_amount`), borrowed from staking because there is no swap-specific key. Nobody has guessed at new copy.
- **Core's `ValueFormatter` is not locale-aware.** Formatting the activation-fee amount in Core would have regressed non-`en` locales, so Core hands over `{ value, decimals, symbol }` and each app keeps its own formatter. Applies to any future amount rule.

#### Live defects found by the verification pass, not yet fixed

| What | Where | Why it matters |
|---|---|---|
| Android silently re-enables push 30 days after a deliberate opt-out | `SettingsViewModel.kt:123-128` calls `stopAskNotifications()` on **disable**, restarting the 30-day timer; `AppViewModel.kt:83-89` then re-asks and `PushRequest.kt:25-29` finds `POST_NOTIFICATIONS` still granted, so it enables with no dialog | Reverses an explicit privacy choice. Needs somewhere to record "the user said no", which does not exist yet |
| Android's `is_push_enabled` never consults runtime notification authorization | `DevicePlatform.kt:118` is preference AND flavour flag; iOS reads live `UNAuthorizationStatus` since `25a6a38bfa` | A user who revokes `POST_NOTIFICATIONS` keeps reporting enabled with a live token. The runtime check already exists in the same module (`NotificationPermissions.kt:13`), wired only to price-alert banners |
| iOS's swap-pay recents query references an unjoined table | `RecentActivityRequest.swift` adds the balances join only for `.hasBalance`/`.enabledBalance`, but `AssetsRequest.applyFilter` emits `balances[availableAmount] > 0` for `.hasAvailableBalance` — swap-pay's set. `RecentActivityRequestTests.swift:49` exercises only `[.hasBalance]` | Must be fixed before iOS is used as the reference for V6 |
| Pull-to-refresh on perpetual markets is a no-op for up to an hour, both platforms | `MARKETS_REFRESH_INTERVAL_SECONDS = 3600` throttles both the Android pull and iOS's 1-minute timer | An explicit user pull should arguably bypass the staleness gate |
| Android's fiat/buy amount has no debounce | `FiatViewModel.kt:140-146` combines `amount` straight into `mapLatest`; iOS debounces 250 ms | Same class as the swap and name debounce drift; in no row |

#### Tests that cannot fail

- `SettingsViewModelTest.kt:72-75` asserts the `stateIn` seed under `StandardTestDispatcher` with no `advanceUntilIdle`, so it does not cover the clause it appears to protect.
- `Migration_88_89Test.kt:35` seeds a multi-sig banner with `asset_id NULL`, the pre-`46889318bc` contract.

Pure duplication, no divergence found (lower priority, still one rule each): swap slippage bounds, min-receive BPS math, swap ETA truncation, the critical-warning gate, collections availability, and the custom-fee minimum check — each written once per platform on top of a Core call that already exists.

### No service is constructed at a call site

A `Gem*Service()` in a field initialiser or at file scope is a second instance of something the graph already owns, and it is where app-side variants creep back in. Every one is injected: iOS registers it in `ServicesFactory`, exposes it through `@Entry`, and passes it into the view model; Android provides it in `RulesModule` and injects it. A stateless Compose component or a value-type extension takes the *answer* as a parameter — it does not reach for the service. Done so far: the amount service, balance calculator and transaction formatter. Still open: the `GemAssetConfigService`, `GemAddressService`, `GemPaymentService`, `GemDeeplinkService`, `GemTransferService`, `GemFeeService`, `GemSwapQuoteService`, `GemChainService` and `GemApplicationMetadataService` holders in `gemcore/ext/*`, `GemstonePrimitives/Sources/Extensions/*` and the Compose components.

### Design decisions to apply everywhere

- **One preferences owner.** `GemPreferencesService` (Core, over the key-value `GemPreferencesStore`) owns every product preference; iOS `GemstonePreferencesStore`/`ObservablePreferences` and Android `UserConfig`/`ConfigStore` are storage adapters plus SwiftUI/Compose observation, nothing else, and no service or view model injects both an app preferences object and `GemPreferencesServiceProtocol`. Two things to know when adding a key: iOS namespaces every key as `gemstone_*` except the handful whose pre-move values must carry over (`currency`, `appearance`, `is_perpetual_enabled`, `is_push_notifications_enabled`, `swap_slippage_bps`, the `perpetual_*` defaults, the per-chain `explorer_name_*` keys, and the four aliased ones — `current_wallet_id`, `price_alerts_enabled`, `is_hide_balance_enabled`, `is_accept_terms_completed`), and `currency` is mirrored into the `group.com.gemwallet.ios` app group for the price widget. The app lock is not a preference: `authRequired` and `getLockInterval` live in secure storage on both platforms (iOS `KeystorePassword`, Android `TinkGemPreferences` over `GemSecureStore`), each still falling back to its pre-move DataStore/`ConfigStore` value until the next write.
- **Amounts are typed, and named for their type.** A big-integer quantity in atomic units is a `value`; an `f64` is an `amount` — `GemBalanceValue.amount: f64` and `margin_amount: f64` against `minimum_value`, `available_value`, `max_value`. A model field holding a big integer is typed `GemBigInt`, never `String` parsed at point of use: `GemBigInt` is a UniFFI custom type (`models/custom_types.rs:48-56`) that lowers to a `String` and lifts with a **fallible** `try_lift`, so the generated Swift and Kotlin signatures stay `String` while a malformed value fails at the boundary instead of becoming a silent zero. That deletes the `unwrap_or_default()` and `is_ok_and(...)` helpers that otherwise accumulate around each field. Two exceptions, both about wire format rather than taste: a type that also derives `Serialize`/`Deserialize` crosses as JSON, and `num_bigint`'s serde form is not a string, so retyping one changes a format shared with the apps or the backend; and `primitives::` typeshare structs are the backend's contract. Leave both as `String` and parse at the edge. Known violations still to fix: `GemPerpetualOrderInput.usdc_value` is named correctly but still typed `String` — retyping it makes `perpetual::rules::order` infallible, which turns the exported `GemPerpetual.order` non-throwing and changes two iOS call sites plus Android, so it lands with both app builds green, not on its own. `Permit2Detail.amount` and `SwapperError::InputAmountError.min_amount` are big-integer strings named `amount`, but they live in the `swapper` crate behind `#[uniffi::remote]`; `Permit2Detail.amount` additionally mirrors the EIP-712 `PermitDetails` field name and stays.
- **Encoding to the boundary cannot fail; decoding from it can.** Core's `json_bridge!` lowers with an infallible closure that `debug_assert!`s and yields an empty string, and lifts with a fallible one. Android's `toJson` does not throw and `decodeJson` does. iOS now matches on both sides: `JsonCodable.json()` is non-throwing — it asserts in debug and returns `""` in release, exactly as Core does — while `init(_ json:)` still throws. A `try` in front of `.json()` was never catching anything a caller could act on, and the four `GemTransactionFormatter` helpers that existed only to absorb it are gone.
- **No `try? … ?? default` on Core calls.** Core preference reads are infallible (`GemPreferencesStore.get`/`GemWalletPreferencesStore.get` return `Option`), so getters return plain values; the only allowed fallback is the JSON→enum decode of a Core-provided enum in `GemPreferencesService+GemstonePrimitives.swift` (`currencyValue`, `chartPeriodValue`, …), never in a view model. Keychain-backed values are not preferences: the gateway's secure store implements the separate, fallible `GemSecureStore` trait (iOS `GemstoneSecurePreferencesStore`, Android `TinkGemPreferences`). A Core method either cannot fail (rule with a built-in default, like `includes_perpetual_collateral`) or its failure is surfaced (thrown, mapped to a localized error, or recorded through `services::failures::record`). Deliberate fail-open fallbacks (keep, do not "fix"): transaction scan outage, address-name lookup → cached names, perpetual account mode → stored mode, explorer name → first explorer, corrupt cached config → refetch, per-chain token search → no match for that chain. Same on Android: no bare `runCatching { coreCall }` without logging or a user-visible state — use `runCatchingCancellable` and handle the failure.
- **A model field is optional only when absent means something.** `AssetInfo.metadata` is `AssetMetaData? = null` on Android while iOS's `AssetData.metadata` is a plain `AssetMetaData` built with a default, and nothing distinguishes "metadata not loaded" from "metadata says no" — so the null is not carrying information, it is deferring a decision to whoever dereferences it. Around forty `metadata?.` call sites then answer it independently and inconsistently: `?.isPinned != true` defaults to not-pinned, `?.isBuyEnabled == true` defaults to false, `?.stakingApr ?: 0.0` and `?.rankScore ?: 0` invent their own zeros. Give Android the same non-null field with one default instance, and every one of those collapses to a plain read. Where a Core rule takes such a flag (`shows_stake_balance(chain, is_stake_enabled, balance)`) the flag itself is right to pass — it is per-asset backend metadata, not derivable from `Chain` — but the missing-value decision belongs in Core or in the model, not re-answered at each call site in two languages.
- **`GemConfirmInput` is not comparable by equality.** `GemTransactionInputType` carries `asset` and `stakeType` as embedded JSON strings, and serde reformats them on decode, so an input that has been through `encode_confirm_input`/`decode_confirm_input` compares unequal to the one it came from while meaning the same thing. The wire form *is* idempotent — pack, unpack, pack returns the same string — and that is the property to assert. Nothing may build navigation dedupe or `distinctUntilChanged` on `GemConfirmInput ==`; `ConfirmInputPropertiesTest` pins both halves of this.
- **A mapping copies fields; it never substitutes a literal.** A constructor that fills one field with `"0"` where the source type has that field is a silent-zero bug wearing the clothes of a conversion — `GemStakeBalance(_ balance:)` did exactly that with `rewards`, understating every staked balance by the user's unclaimed rewards and, for an asset that is not stake-enabled, removing the stake row from the screen entirely. When a source field genuinely has no counterpart, that is a modelling gap to raise, not a literal to invent.
- **Swift warnings are invisible from the build recipe.** `just ios build` pipes xcodebuild through `xcbeautify --quieter`, which drops warnings entirely, so dead `try`, unreachable `catch` and their kind accumulate unseen — 42 had built up before anyone looked. To see them, run xcodebuild raw with the same flags, after touching the sources (Swift only re-emits warnings for files it recompiles).
- **Store adapters are mirrors.** For every `Gem*Store` trait method the iOS and Android adapters must do the same reads/writes with the same filters, batching and transactions; the divergences below are bugs until proven otherwise. Settled so far: `add_missing_balances` filters to stored assets in Core, so neither adapter repeats the lookup and neither can insert a balance that violates its asset foreign key; a perpetual's asset row is written by Core through `GemAssetStore.save_assets` (`perpetual_asset_basics`), so neither perpetual adapter touches the asset table and Android stops silently ignoring an updated row; `set_swappable_assets`/`set_stakeable_assets` are additive on both platforms because `sync_swappable_chains` and `sync_default_assets` top up the same flag that `sync_availability` replaces; a transaction's derived swap asset links are rebuilt on both platforms whenever metadata arrives, inside the same database transaction as the row it belongs to; the banner row id is Core's `banner_identifier` on both platforms; balances are filtered and written in one round trip on both (`getVisibleAssetIds` in SQL, `setWalletAssetsVisibility` and `updateBalances` inside one database transaction); pending transactions read their wallets in a single query; search lists are delete-then-insert per type on both, so an empty result clears the stale rows instead of leaving them; address-name deletes are one transaction.
- **One device identity owner.** `GemDeviceKeyService` (Core, over `GemSecureStore`) owns get-or-create: it stores the private key as hex under `device_private_key`, derives the public key rather than trusting a stored copy, and caches the pair. It refuses to mint a new identity when the store errors or the stored key is unreadable — the failure propagates instead, because a silent regeneration re-registers the device and orphans its push subscriptions. The two app implementations are deleted (`GetDeviceId`/`GetDeviceIdImpl`, `SecurePreferences.getOrCreateDeviceKeyPair`/`getDeviceId`), and each app's `GemSecureStore` adapter carries the legacy read-through: iOS reads the unnamespaced keychain `devicePrivateKey` `Data` and rewrites it as hex under the namespaced key, keeping `whenUnlockedThisDeviceOnly`; Android reads `TinkSecurityStore` (`gem_device_keys`, and through it the pre-Tink DataStore layer) and rewrites it into the Core store.
- **One device owner.** `GemDeviceService` owns registration, the pushed-device diff and the in-flight lock; both apps hold it directly, and Android's `DeviceRepository` is only what it really is — the `GemDevicePlatform` implementation, reading the currency from `GemPreferencesService` like iOS. Android calls `synchronize_if_needed` everywhere while iOS forces `synchronize` after a token, currency or subscription change; Core's `needs_sync` already diffs all three, so the cheaper call is the right one and the difference is deliberate.
- **One currency owner.** `GemPreferencesService` holds the currency on both platforms, because Core-internal reads (`GemBalanceService`, price syncs) must see the same value the UI shows — a second copy silently runs them on the default. Android keeps the Room `session.currency` row in step only because the `asset_info` view joins prices on it.
- **Ephemeral state that the UI observes belongs to the store adapter, not to Core and not to a class of its own.** Core has no way to publish a change, so the `Gem*Store` trait *is* its push channel: `GemSupportStore.update_typing` arrives from the stream and the adapter holds the agent where SwiftUI and Compose can observe it (`GemstoneSupportStore.typingAgent` on both apps).
- **Tracking a transaction is fire-and-forget on purpose.** `GemTransactionStateService.track` polls until the transaction settles, so a caller that awaits it blocks for the lifetime of the transaction. Both apps start it in a detached task and log the failure; do not "fix" this by folding `track` into `add_notification_transaction` in Core.
- **A case holds one Core service, or composes other cases — never a repository.** See [How a service is built](#5-call-it-from-the-app--the-service-itself-on-ios-a-case-on-android); the repositories still in the Android graph are legacy and listed under [Android](#android).

### The confirm seam

Every service and store trait is migrated; what is left of the migration lives in one screen. Both apps rebuild Core's confirm records as their own aggregates, map into them, and map back out to call Core. Core already exposes the whole flow: `GemConfirmInput`, `GemConfirmLoadOptions`, `GemConfirmData`, `GemSendInput`, `GemExecuteResult`, `GemTransferData`, `GemRecipient`, `GemTransactionInputType`, `GemTransactionLoadFee`, `GemFeeRate`, `GemGasPriceType`, `GemFeeOptions`, `GemTransactionLoadMetadata`, `GemSignerInput`, `GemSignedTransaction`, plus the `GemTransferService` derivations (`transaction_type`, `asset`, `fee_asset`, `output`, `approval`, `available_value`) and the free rules `acquire_asset_flow`, `default_fee_priority`, `is_insufficient_network_fee`.

App types in the seam, with their Core home:

| App type | Platform | References | Core record |
| --- | --- | --- | --- |
| `ConfirmParams` (sealed, 16 subclasses) + `Builder` | Android | 249 | `GemConfirmInput` + `GemTransferData` |
| `AmountParams` | Android | 166 | `GemTransferData` fields |
| `TransferData` mappers (`toTransferData`, `toDto`, `toConfirmParams`) | Android | 62 | `GemTransferData`, `GemTransactionInputType` |
| `SignerParams` | Android | 14 | now carries `GemConfirmData` itself, plus the input, the typed `Fee` and the signed amount |
| `Fee` (sealed), `FeeSelection`, `DestinationAddress` | Android | — | `GemTransactionLoadFee`, `GemConfirmFeeSelection`, `GemRecipient` |
| `ConfirmTransferPreload` | iOS | 8 | `GemConfirmData` |
| `TransferAmount` | iOS | 18 | `GemTransferAmount` — the typed view of Core's calculator output |
| `Fee`, `FeeRate`, `GasPriceType`, `TransferData`, `TransferDataExtra` (Primitives) | iOS | — | `GemTransactionLoadFee`, `GemFeeRate`, `GemGasPriceType`, `GemTransferData`, `GemTransferDataExtra` |

Android converts the same payload seven times per confirm screen: `pack` → `unpack` → view model re-pack → `SignerPreloaderProxy.toConfirmInput` → `CalculateTransferAmount` → `availableValue` → `toSendInput`.

Blockers, in the order they have to be solved:

1. **Typed reads over JSON strings.** Every Core primitive crosses UniFFI as a JSON `String` typealias (`Asset`, `AssetId`, `Account`, `Wallet`, `ApplicationMetadata`, `ApprovalData`, `StakeType`, `PerpetualType`, `TransactionType`, `SimulationResult`, `ScanTransaction`). The UI reads `asset.decimals`, `asset.symbol`, `assetId.chain` everywhere, so carrying `GemConfirmInput` end to end means decoding at each read site or keeping one decoded shadow per screen. This is the largest blocker and it applies to both apps.
2. **`GemConfirmData` does not echo its input.** Android's `SignerParams.input` and iOS's `ConfirmTransferInput` exist only to keep the input beside the result. Either the apps hold the `GemConfirmInput` they sent, or Core returns it.
3. **The signed amount has no carrier.** Android's `SignerParams.finalAmount` (written once, read once) and iOS's `TransferAmount.value` both feed `GemSendInput.value`; nothing in Core carries it between `load` and `execute`.

4. **Fee priority is split.** `GemTransactionLoadFee` has no priority; `GemConfirmData.selected_priority` carries it separately and both apps fuse them into their own `Fee` (iOS's `Fee` has no priority at all — the selection carries it, which is the shape Android should reach). A custom gas price still comes back as `selected_priority: normal` from `select_fee_rate`, so `Fee.priority` lies for custom fees until Android stops storing the priority in `Fee`.
5. **Fee subclass identity.** Android's `Fee` sealed subclasses (Plain/Regular/Eip1559/Solana) are chosen from the chain type and then unchecked-cast the gas price; Core models this as one `GemGasPriceType` enum. iOS keeps `BigInt` fee math against Core's decimal strings.
6. **`ConfirmParams` is narrower than Core's enum.** `TokenApprove` and `Earn` have no `ConfirmParams` subclass, so `unpack` returns `null` and the confirm screen cancels; the Earn half waits on the Android Earn surface.
7. **Presentation code switches on app subclasses.** `ConfirmProperty.Destination.map`, `BuildConfirmPropertiesImpl.getValidator` and the swap/perpetual detail builders do exhaustive `when` over `ConfirmParams`; they have to be rewritten against `GemTransactionInputType`.
8. **`GemConfirmData.scan` is never read** by either app — only the fatal verdicts surface, as errors from Core.

Defects found in this path (fixed ones are removed from this list): the WalletConnect `extra.gas_price` from a dapp is dropped by Android's `toGenericParams`/`Generic.toDto` round trip (latent — Core has no consumer yet); `GemTransferData.minimum_value` is never read by `toConfirmParams` and survives only because swaps re-derive it from the quote; `ConfirmParams` overrides `hashCode` without `equals`, so every re-emission looks like a change.

### One service per feature, the same one on both platforms

A screen should ask one thing for its data, and that thing should be the same on both platforms: on iOS the Core `Gem*Service` for the feature, on Android the coordinator that implements that feature's cases. Where a feature's decisions need no store or API they live on its constructible service (`GemStakeConfig`, `GemAssetConfigService`, `GemChainService`); where it needs I/O it holds the I/O service too, and the split stays visible.

The features to shape this way, in the order the screens were built:

| Feature | iOS today | Android today | Target |
| --- | --- | --- | --- |
| Validator selection | `ValidatorSelectSceneViewModel` + `StakeSceneViewModel` each call `GemStakeConfig` | `GetValidators`, `GetRecommendedValidator`, `GetRecommendedValidatorIds` cases | one `GemValidatorService` in Core over `GemStakeStore` (selectable, recommended, ids, by id), so neither app filters or sorts validators itself |
| Delegation | `DelegationSceneViewModel` holds `GemStakeConfig` | `GetDelegation`/`GetDelegations` cases over `StakeDao` | delegation reads move behind `GemStakeService`; actions and claimability stay on the constructible `GemStakeConfig` |
| Price alerts | `PriceAlertsSceneViewModel` holds `GemPriceAlertServiceProtocol` | `PriceAlertsCoordinator` implements the five write/flag cases; the three reads hold `GemstonePriceAlertStore` | the reads move onto `GemPriceAlertService` (it already owns the store trait), leaving those three cases as one-line forwards or nothing |
| Transactions | `TransactionsViewModel` over GRDB requests | `GetTransactions`/`GetTransaction` over `GemstoneTransactionStore` | the row's title and its sub-line are `GemTransactionFormatter.title`/`.subtitle` on both platforms; still open — `GemTransactionsService` answers the point reads, and the value sign and the swap pair follow them into the formatter. Only the observed list stays platform-side |
| Perpetuals | `PerpetualSceneViewModel` + observer trio | `PerpetualRepository` + observer trio | see the Hyperliquid item below; the reads join `GemPerpetualService` |
| NFT, contacts, support, banners, notifications | view models hold the Core service | cases over the DAO | same shape: the Core service answers, the app observes |

- **A case must not take a Room DAO.** It is the same violation as taking a repository: the case then owns a second read path Core cannot see, and the two drift. Every case now holds the platform *store adapter* instead — the same `Gemstone*Store` Core writes through, which is where `GemstoneSupportStore.typingAgent` already lives. The adapters carry the observed reads (`GemstoneStakeStore.observeValidators`, `GemstoneBannerStore.observeMultiSign`, `GemstoneNftStore.observeNftData`, `GemstoneAddressStore.observeAddressName`, `GemstoneFiatStore.observeTransactions`, `GemstonePriceStore.observeUsdPrice`, …), so each DAO has one reader again. What is left for these cases is the second half: moving the point reads onto the Core service that already owns the matching `Gem*Store`, leaving the app only what Core cannot express — an observed list.

### Hyperliquid streaming belongs in Core

Done, in part. `GemPerpetualStreamService` ([perpetual/stream.rs](../gemstone/src/services/perpetual/stream.rs)) owns the subscription set, the frames and the dispatch: `connected`, `disconnected`, `subscribe`, `unsubscribe` and `handle`, which applies a socket message and hands back only a chart candle. Each app keeps its socket behind the `GemPerpetualStreamConnection` foreign trait and its chart sink, which deleted `HyperliquidSubscriptionService` and `HyperliquidEventHandler` on both platforms (about 160 lines) along with the two app tests that were re-asserting the framing `WebSocketSubscriptions` already tests in Rust.

The websocket URL rule went with it: `GemNodeService.websocket_node_url(chain)` swaps the scheme and appends `"ws"` once, which deleted iOS `PerpetualNodeService` and Android's `toWebSocketUrl`. What is left of the stack is the connection lifecycle, and it is smaller than it was. `GemPerpetualService.connection(wallet)` answers what both observers used to work out for themselves — which account signs for perpetuals, and its account mode, syncing positions first and falling back to the stored mode when that fails. Neither observer resolves an address or a mode any more. What remains is genuinely each platform's own: [iOS `HyperliquidObserverService`](../../ios/Packages/GemstoneServices/Sources/Perpetual/HyperliquidObserverService.swift) is an actor that holds one wallet and cancels its observe task; [Android's](../../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/perpetual/HyperliquidObserverService.kt) combines a foreground flag with the observed wallet and collects. Those are the idioms, not the rules, and they stay.

One divergence the comparison surfaced and did not fix: Android syncs the perpetual market list when the wallet changes (a second `scope.launch` calling `syncPerpetuals`) and iOS does not. It needs a decision rather than a port — markets are global rather than per-wallet, and the sync is throttled by `MARKETS_REFRESH_INTERVAL_SECONDS` anyway, so the question is whether the Android call earns its keep, not whether iOS is missing one.

### Rust (core/gemstone)

- A service reaches its own rules. `GemNameService.recipients()` hands back the `GemRecipientService` it already holds, so nothing holding the domain service needs a second one registered beside it. Prefer this over adding a sibling service to the resolver or the Hilt graph. The limit is construction cost, not taste: a caller that needs only rules must not be made to hold a service that needs a gateway, an API client and stores — that is what `GemStakeService` needs, which is why stake's rules live on the constructible `GemStakeConfig` and the service delegates to it rather than carrying a second copy.
- Free functions become services. A `#[uniffi::export] pub fn` is a rule with no home: each app reaches it through its own extension (`Chain.matches(query:)`, `List<Chain>.filter(query)`), and those extensions are where app-side variants creep back in. `GemChainService` (`get_chains`, `get_matching_chains`, `is_valid_network_id`) is the shape: a `uniffi::Object` with a `new()` constructor, held like any other service. This started at 82 exports; 10 are left and all are deliberate: `create_auth_message`, `generate_device_key_pair`, `decode_private_key`, `encode_private_key` and `supports_private_key_import` are the key-material boundary calls, and `connection_status`, `service_status_timeout_seconds`, `banner_identifier` and `parse_support_message_display_content` are the singles at the bottom of the table. The record of where each group went:

| Target service | Functions to fold in |
| --- | --- |
| Left as free functions | `parse_support_message_display_content`, `banner_identifier`, `connection_status`, `service_status_timeout_seconds` — one function each, no siblings to group with, and every caller is value code (a message composable, a store adapter). A service needs at least two methods to be worth holding. `default_contact_chain` left this list: it became `GemContactService.default_chain`, and the two data-class defaults that called it now take the chain from the view model |
| Keep as free functions | `generate_device_key_pair`, `decode_private_key`, `encode_private_key`, `supports_private_key_import`, `create_auth_message` — key material, called once at a boundary that already owns the secret |

- Where a service is built follows what it needs. A service with a store, an API client or a gateway is built once in `ServicesFactory` (iOS) or a Hilt module (Android) and injected — nothing constructs one at a call site. A **constructible** service (`GemChainService`, `GemAddressService`, `GemAssetConfigService`, `GemStakeConfig`, `GemTransferService`, `GemFeeService`, `GemAmountService`, `GemPaymentService`, `GemDeeplinkService`, `GemRecipientService`, `GemSwapQuoteService`, `GemApplicationMetadataService`, `GemTransactionFormatter`, `GemSimulationFormatter`, `PriceAlertFormatter`, `BalanceCalculator`, `GemPerpetual`, `GemKeystore`) is reachable from value code that has no injector, so both apps hold one as a file-level `private let`/`private val` — the same pattern, file for file. Every site holds one now — the only remaining `GemAssetConfigService()` in an expression is iOS `SelectAssetViewModel.popularIds`, a `static let` evaluated once.

- One-sided exports still open: `wallet_connect::authentication_chain_ids` waits on iOS WalletConnect authentication; `nft::report` has only the iOS `ReportNftViewModel` and waits on an Android report screen; `wallet_preferences::is_initial_load_completed` drives the iOS wallet empty state and has no Android equivalent; `reset_transactions_timestamp` is an iOS developer action.
- Transfer model: generate the `TransactionInputType` enum from typeshare so the primitives tuple enum, the gemstone named-field enum and the Swift/Kotlin enums collapse (685 Core, 52 Android, 5 iOS references — do it after both apps carry Core records through confirm, transaction construction is wallet-critical). **Not started.**

### iOS

- Store reads that belong to Core: `ConfirmSimulationService`, `FeeAssetProvider`, `TransferMetadataProvider` and `ConfirmService` read `AssetStore`, `BalanceStore`, `PriceStore` and `AddressStore` directly because the confirm screen builds its first state in a synchronous initializer and every Core method is `async`. Make those point reads synchronous on the store traits the way `GemWalletStore.get_wallet` already is, expose them synchronously on `GemAssetsService`, `GemBalanceService`, `GemPriceService` and `GemNameService`, and the four readers ask the owner. Do it before the confirm seam.
- Confirm view models still assemble `GemSendInput` from app aggregates. See **The confirm seam** below — it is the last migration item before the `TransactionInputType` collapse.
- The rule for a failing Core call on iOS: a Core read whose `Optional` models a real state (no wallet yet, no selected node) returns that optional and says nothing; a failure the app recovers from is **logged** and the recovery is deliberate; a failure the user must act on is thrown and mapped to localized text. Deliberate recoveries, keep them: `PerpetualService.accountMode` → the stored mode, `AmountDataProvidable.limits` → zero available (fail-closed for a send), `String+Keystore` → utf8 bytes for pre-hex passwords, `LocalKeystore.keystorePassword` → `""` is the "no password stored yet" sentinel, not a swallowed error. The 513 `try?` left outside tests are value conversions — JSON round trips, formatter parses, `wallet.account(for:)` — where `nil` is the render path, plus the developer screen; leave them.
- Two store differences remain, and neither is an adapter bug: `banners.walletId`/`banners.assetId` are foreign keys and SQLite does not apply `INSERT OR IGNORE` conflict resolution to foreign keys, so a banner for an asset the app has not stored throws where Android stores it; `PortfolioStore` reads through `asset_info`, which joins `accounts`, so a wallet without an account row for the chain contributes nothing.
- Dead code: the two TODOs are not stale — `NavigationHandler`'s `.stake` deep link is an unimplemented branch and `TransactionScene`'s corner radius is an open iOS 26 styling question; both mark real gaps and stay until someone closes them. The two "delete in 2026" `FileMigrator` calls (`LocalKeystore`, `DB.swift`) move the keystore and the database from the documents directory to application support on launch; deleting them strands anyone who has not opened the app since the move — losing their keystore — so this needs install-base data, not a code decision.
- Consistency: the `JSONEncoder`/`JSONSerialization` in `WalletConnectorService` and `WalletConnectResponseType` sit on the dapp-JSON boundary and belong there.
- Naming, still open: `GemstoneNftStore` wraps `NFTStore` and `ConnectionStore` wraps `ConnectionsStore` (one name per store), and untyped `.map()` conversions where Android has `toPrimitives()`.

### Android

In priority order. The last three are settled or deliberately held; presentation and localization come last.

- Earn flow: Android has no Earn surface yet (no `StakeProviderType.Earn` reader, no `AmountParams.Earn`, no `ConfirmParams.Earn`, `GemDelegationAction.DEPOSIT` maps to nothing); build the Earn scene, the amount provider and the confirm params on top of `GemStakeService.sync_earn`/`get_earn_data`, `GemAmountType::Earn` and `GemTransactionInputType::Earn` (iOS `EarnSceneViewModel` + `AmountEarnViewModel` are the reference). This is a feature, not a consolidation, so plan it as its own batch.
- Store adapter divergences (mirror iOS or fix iOS, per method). Settled: `asset_info` no longer joins prices on `session.currency` — the guard meant every balance read as empty between `setCurrency` writing the session row and Core's `convert_prices` stamping the price rows, and `TransactionsDao` already joined the same table without it. The currency has one owner again (`GemPreferencesService`); the session row's copy is a legacy read-through for the one-time `setup_currency` migration and nothing else. Still open: `AssetStore.saveAsset` bumps `updatedAt`, `TransactionStateStore` writes swap amounts and `NftStore` fills two legacy image columns — all three are confirmed dead `NOT NULL` columns with no iOS counterpart, but minSdk 28 has no `ALTER TABLE DROP COLUMN`, so removing them means recreating tables (`asset` behind its foreign keys) and instrumented migration tests do not run in CI; batch them with a migration that has another reason to touch those tables. `PriceStore` still stamps `prices.currency`, which is now only the label `AssetPriceInfo.currency` reads, and `mapNotNull`s unparsable ids where iOS maps straight through. Not a divergence: `ConnectionStore`'s three extra methods are the observed reads the case pattern puts on the adapter — iOS gets the same from `ConnectionsRequest`.
- Layering: the portfolio screen's presentation is Core's now — `GemPortfolioService.portfolio_data` decides the charts, the statistics and the available periods for a wallet or a perpetual account, so iOS `PortfolioDataService` and Android `GetPortfolioDataImpl` only pass an input and decode. `GetWalletSummaryImpl` is down to five collaborators: the perpetual half is `PerpetualBalanceCoordinator`, which owns every read of the collateral balance — the raw one, the formatted one and the one that counts toward the wallet total (`getCollateralIncludedInTotal`, the gate iOS applies inside `AssetFiatValuesRequest`). It replaced the `GetPerpetualBalance`/`GetPerpetualBalances` pair, whose two impls read the same row. `SearchSwapAssetsImpl` and `ConfirmTransactionImpl` were on this list for mixing a Core service with an app one, but `AssetsSearchService` and `RecentAssetsService` are the platform's query layer over tables Core does not own (a filtered search, the recent-activity list), which is the same role a `Gemstone*Store` adapter plays; `TokensRepository` and `BannersRepository` are gone — `SearchTokensImpl` holds `GemSearchService` and `GemAssetsService` and picks the call per site the way iOS does. `data/services/native-provider` (only the alien provider) is a candidate to fold. An observed read belongs on the `Gemstone*Store` adapter, and the case holds the adapter — `GetShowWelcomeBannerImpl`, `ObserveFiatTransactionsImpl` and `GetAssetPriceUsdImpl` are the shape to copy.
- Node screens: `AddNodeViewModel` holds `NodeStatusService`, `AddNodeCase`, `SetCurrentNodeCase` and a raw `GemChainService`, and does the network-id check itself — `GemGateway.check_node` answers it now, as it does for iOS. It belongs to the nodes work in flight.
- Case naming: the legacy `com.gemwallet.android.cases.<area>/` tree is down to `cases/nodes/` (seven `*Case` interfaces, owned by the nodes work in flight); everything else lives in `application/<area>/cases/` with the `Case` suffix dropped. New areas came out of the fold: `addresses`, `contacts`, `tokens`, `security`. `bridge` merged into `wallet_connect` — one area, matching iOS.
- `UserConfig` is the preferences adapter now; only the app-lock keys are left outside Core, in secure storage by design. What remains is deleting the `ConfigStore` fallback for `auth` once enough installs have written the secure value.
- Consistency: still open — `toChain()` (nullable) and `requireChain()` (throws) are picked arbitrarily at call sites, and `*Service` classes live inside the coordinators module. Settled: the JSON bridge is one convention now, `decodeJson` (throws) and `decodeJsonOrNull` (null is the render path, for dapp and push payloads only); `tx` is gone from Kotlin names (`DbTransactionSwapMetadata.transactionId`, `transactionProperties`) with the database columns kept verbatim; no `fetch*`/`resolve*` names are left; the `AssetsDao` overload pair is `getAssetsInfo`/`getAssetsInfoByIds`. Not a divergence: every file in the `gemstone` package implements a Core foreign trait — `GemNotificationPermissions`, `GemStreamConnection`, `GemFileStore`, `GemKeystorePassword`, `GemPerpetualStreamConnection` are adapters like the stores, and `WalletConnectSigner` is `WalletConnectPendingRequests` in `application/wallet_connect/`.
- Repositories: **all gone.** `grep -rn Repository android/` finds nothing. `PerpetualRepository`, `WalletsRepository`, `AssetsRepository`, `SessionRepository` and `WalletConnectorService` were replaced by cases over the `Gemstone*Store` adapters, plus three coordinators where one object had to own shared state: `WalletAssetsCoordinator` (the one current-wallet asset flow), `SessionCoordinator` (the observed session and the current currency), `WalletConnectCoordinator` (the client initialization and the merged event stream). Two more followed for the same reason: `PerpetualBalanceCoordinator` (every read of the collateral row) and `PriceAlertsCoordinator` (the alert writes and the observed enabled flag). The module they lived in is `data/services/gemstone` now — the layer that implements Core's foreign traits, named the way Core and iOS name it.
- Error handling: suspend work uses `runCatchingCancellable` and logs what failed. The plain `runCatching` calls left are around non-suspend work (URI parsing, `valueOf`, `startActivity`, focus requests, JSON decode helpers), where cancellation cannot be swallowed — leave them.
- Compose/platform deprecations: `PullToRefreshDefaults.containerColor` is `indicatorContainerColor` now. Two are deliberate holds. `rememberModalBottomSheetState` (5 sites) is deprecated for `rememberBottomSheetState(initialValue = Hidden)`, but the deprecated wrapper pins `isBottomSheetPartiallyExpandedDeterministicEnabled = false` while the replacement reads the live Compose flag — migrating changes how every sheet settles, on an alpha Material3 API, for no user benefit; wait for the flag to stabilize. `FirebaseMessaging.token` in `RequestrPushToken` is deprecated with no replacement in firebase-messaging 25.1.0 — `getToken()` is still the only API on the class. The clipboard ones are gone: the 14 screens take the platform `ClipboardManager` from the context (`Context.clipboardManager()`), so neither `Clipboard.nativeClipboard` nor the `NativeClipboard` typealias is used.
- Localization: `DevelopScene`/`PaymentsScene` are developer-only screens and stay English. Still open: 59 hardcoded `dp` values (worst: `SupportMessageBubble`, `ReceiveScreen`, `ImportScreen`, `WalletTypeTab`, `FiatScene`).

### How to work this list (for whoever picks it up)

- One change at a time: implement in Core → regenerate bindings only if an exported signature changed (`just generate-stone` and `just generate-android-stone` from the repo root, never while an iOS build is running) → wire both apps → delete the app code it replaces → verify → commit and push to `main` directly (no PR, no `Co-Authored-By`/session trailers) → arm a CI watch for the pushed commit and fix red CI before anything else.
- Every commit that finishes an item removes its line from this file in the same commit; do not add checkboxes or "done" notes.
- Rules go in `rules.rs` with a unit test that would fail if the rule flipped; services are `services/<name>/{mod,model,rules,store,error}.rs` with only the files they need. No code comments. No `utils`/`helper`/`fetch`/`resolve` names, no `tx`.
- Compare the pre-change app logic (`git show <sha>^:<path>`) with the Core rule before deleting it; when iOS and Android disagree, pick the iOS rule unless it is clearly wrong, and say so in the commit message.
- Stores only write rows whose values differ; a case holds one Core service or composes other cases, never a repository.

### Verification

- Core: `cargo fmt --all && cargo clippy -p gemstone --all-targets --all-features -- -D warnings && cargo test -p gemstone --lib --all-features`; regenerate bindings only when an exported signature changes. CI also compiles the workspace with `--features unit_tests` and `chain_integration_tests`.
- Android: root `./gradlew compileGoogleDebugKotlin compileGoogleDebugUnitTestKotlin` plus the touched modules' `testDebugUnitTest` (the root task does not compile every module's unit tests).
- iOS: `just build && just test` from `ios/`.
- After each batch: compare the pre-change app logic (`git show <sha>^:<path>`) with the Core rule and add the Core test that would flip if the rule flipped.

## Conventions

- Identifiers cross the FFI typed: `WalletId`, `AssetId`, `Chain`, `NFTAssetId`, `Currency`; store row ids stay `String`.
- Store methods: `get_*` reads, `is_*` boolean reads, `set_*` preferences and stored flags or sets (`set_buyable_assets`, `set_assets_enabled`, `search::set_assets`), `save_*` upserts, `add_*` inserts that must not overwrite existing rows, `update_<items>(…, items, delete_ids)` for reconcile writes, `delete_*` removals, and `clear*` for wiping a whole scope (`preferences::clear`, `support::clear_typing`).
- Rules live in `rules.rs` with unit tests built from the `testkit` mocks (`NFTData::mock_with`, `Asset::mock`, …), not hand-written literals; add the mock to `crates/primitives/src/testkit/` when one is missing. `primitives` types stay policy-free.
- Chain icons come from the chain config, never an app-side list: `icon_chain` (an Ethereum layer 2 draws the Ethereum icon, SeiEvm draws Sei's) and `badge_chain` (the layer 2's own icon as the badge), both following `EVMChain::is_ethereum_layer2`.
