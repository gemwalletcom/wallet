# Gemstone Services

Core-owned services live in [`core/gemstone/src/services/`](../core/gemstone/src/services/) as `<name>/{mod,model,rules,store,error}.rs` with only the files they need. Services and stores use the shared [`GemServiceError`](../core/gemstone/src/services/error.rs) unless the app needs a structured feature error to render or branch on. A service owns the I/O flow; pure answers live on an honest domain receiver or in private rules according to [`ARCHITECTURE.md` § 6](ARCHITECTURE.md#6-where-derived-domain-answers-live). Each app implements the `Gem*Store` trait over its database or preferences and constructs the service in DI ([`ServicesFactory.swift`](../ios/Gem/Services/ServicesFactory.swift), Hilt modules under [`android/data/services/gemstone/.../di`](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/di/) and [`android/data/coordinators/.../di`](../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/di/)). Read [How a service is built](#how-a-service-is-built) before adding or changing one.

## How a service is built

[`GemPriceAlertService`](../core/gemstone/src/services/price_alert/mod.rs) is the reference: it calls the device API, reads a preference, writes a database table and asks the platform for a permission, so it exercises every seam a service can have. New services copy its shape; existing ones move toward it.

### 1. Core owns the flow

`core/gemstone/src/services/<name>/` holds only the files that service needs:

| File | Holds |
| --- | --- |
| `mod.rs` | the `#[derive(uniffi::Object)]` service, its `#[uniffi::constructor]`, and short exported orchestration methods |
| `rules.rs` | pure feature decisions with unit tests |
| `store.rs` | the `#[uniffi::export(rust, foreign)]` trait the apps implement |
| `model.rs` | feature records/enums and intrinsic behavior; only types crossing FFI derive UniFFI |
| `error.rs` | only when [`GemServiceError`](../core/gemstone/src/services/error.rs) cannot express a case |

The service may own its feature store and holds `Arc`s of other Core services, never another
domain's store or an app type. Narrow foreign traits are allowed for platform capabilities:

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

Everything that decides belongs in a pure function or receiver method with a test that fails if the
rule flips. A failure is either impossible through a built-in default, surfaced as a typed error,
or recorded through `services::failures::record` — never swallowed.

**There is no `*RulesService`.** A pure answer with one honest receiver belongs on that receiver,
even when it takes additional plain values: `inputType.transactionAsset()`,
`stakeBalance.showsStakeBalance()` and `contactAddressInput.addAddress(addresses)`. A pure rule
with no honest receiver stays private in `rules.rs` and is called by the service. I/O, stores and
platform ports belong on the service that owns the flow. See the complete decision matrix in
[`ARCHITECTURE.md` § 6](ARCHITECTURE.md#6-where-derived-domain-answers-live).

A constructible service is **not** a licence to hold one at file scope. `private let
addressService = GemAddressService()` or `private val assetConfig = GemAssetConfigService()` above
a value-type extension is a hidden global: nothing can substitute it, and the extension reaches
outward instead of receiving what it needs. A mocked Core rule is a premise, not a check, so its
real mutation-checked test remains with its owning Core implementation; app tests assert only
mapping, wiring and state.

The narrow exception is a dependency-free FFI transport adapter for a type whose receiver methods
cannot cross the boundary. `GemSimulationFormatter` and `PriceAlertFormatter` may be constructed
locally; they have no state, I/O or substitutable dependency. Keep the adapter cohesive and do not
create one object per method.

### 2. Pick the store the value belongs in

| What the service needs | Trait | Shape | iOS | Android |
| --- | --- | --- | --- | --- |
| rows in the database | one `Gem<Name>Store` per service ([example](../core/gemstone/src/services/price_alert/store.rs)) | point/short-list reads may be sync; writes and asynchronous reads are `async`; every method returns `Result<_, GemServiceError>` | GRDB adapter under [`GemstoneServices/Sources/Stores/`](../ios/Packages/GemstoneServices/Sources/Stores/) | Room adapter under [`data/services/gemstone/.../stores`](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/stores/) |
| a value the user set | [`GemPreferencesStore`](../core/gemstone/src/services/preferences/store.rs) through `GemPreferencesService` | sync; `get` returns `Option<String>` and **cannot fail** | `GemstonePreferencesStore` over `UserDefaults` | `GemstonePreferencesStore` over `SharedPreferences` |
| the same, per wallet | `GemWalletPreferencesStore` through `GemWalletPreferencesService` | sync, keyed by `WalletId` | same file layout | same file layout |
| a secret | [`GemSecureStore`](../core/gemstone/src/services/preferences/store.rs) | sync; **every read can fail** | `GemstoneSecurePreferencesStore` over the Keychain | `TinkGemPreferences` over Tink |
| something only the OS can do | a foreign trait of its own (`GemNotificationPermissions`, `GemStreamConnection`) | whatever the platform needs | app class | app class |

- One owning trait per persistence boundary. A second trait over the same rows is how the two apps
  drift apart; one cohesive feature trait may span closely related rows such as contacts and their
  addresses.
- A new preference is a `const` key plus typed accessors on `GemPreferencesService` — single-word keys (`_` separates the settings hierarchy in environment variables), never a raw key string in an app.
- The preference read is infallible on purpose: getters return plain values, so neither app writes `try?`/`runCatching` around them. Secure reads are fallible and their failure must propagate — a swallowed secure read regenerates identity or loses a key.
- Store methods follow the vocabulary in [Conventions](#conventions): `get_*`, `is_*`, `set_*`, `save_*`, `add_*`, `update_<items>(items, delete_ids)`, `delete_*`, `clear*`.

### 3. Each app implements a thin store adapter

iOS, `ios/Packages/GemstoneServices/Sources/Stores/<Name>Store.swift`, class `Gemstone<Name>Store`, converting with `.json()` and `Primitives.<T>(_:)`:

```swift
public final class GemstonePriceAlertStore: GemPriceAlertStore, @unchecked Sendable {
    private let store: PriceAlertStore
    private let priceAlertFormatter = PriceAlertFormatter()

    public init(store: PriceAlertStore) {
        self.store = store
    }

    public func updatePriceAlerts(alerts: [Gemstone.PriceAlert], deleteIds: [String]) async throws {
        try store.diffPriceAlerts(
            deleteIds: deleteIds,
            alerts: alerts.map { try (id: priceAlertFormatter.alertId(alert: $0), alert: Primitives.PriceAlert($0)) },
        )
    }
}
```

Android, `android/data/services/gemstone/.../stores/<Name>Store.kt`, class
`Gemstone<Name>Store`, converting with `toJson()` and `decodeJson()`:

```kotlin
class GemstonePriceAlertStore(
    private val priceAlertsDao: PriceAlertsDao,
) : GemPriceAlertStore {

    override suspend fun updatePriceAlerts(alerts: List<String>, deleteIds: List<String>) {
        priceAlertsDao.update(alerts.map { it.decodeJson<PriceAlert>().toRecord() }, deleteIds)
    }
}
```

The two adapters are mirrors: same methods, same conflict behaviour (upsert where the other upserts), same "write only rows whose values differ" rule, same treatment of a missing row. A difference between them is a bug in one of them, not a platform choice. Types retained as JSON custom types (`Account`, `Wallet`, `SimulationResult`, …) arrive as `String` typealiases and are decoded once at the relevant FFI/app boundary. That boundary may be a store adapter, coordinator, or feature mapper; undecoded JSON must not travel deeper into the app. Types supported through `EXPOSED_TYPES`, such as `Asset`, use generated structural mappers instead.

### 4. Construct it once

iOS builds the store and the service in [`ServicesFactory.swift`](../ios/Gem/Services/ServicesFactory.swift) and publishes the service through `AppResolver` and an `@Entry` on the environment:

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

Android provides the store and the service from one Hilt module ([`PriceAlertsModule`](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/di/PriceAlertsModule.kt)):

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
public final class SupportChatSceneViewModel {
    private let service: any GemSupportServiceProtocol
    private let typing: SupportTypingState

    public init(service: any GemSupportServiceProtocol, typing: SupportTypingState) {
        self.service = service
        self.typing = typing
    }
}
```

The screen reads the service out of the environment and passes it in —
`@Environment(\.supportService)`, built once in `ServicesFactory`. **Do not write an app service
around a Core service.** The distinction is which way the dependency points: a class that
*implements* a Core trait is an adapter and is required (the `Gemstone*Store` classes,
`GemstoneNotificationPermissions`, the WalletConnect signer); a class that only calls a Core
service and re-exposes it is a wrapper, and the view model should hold the protocol instead.

**Android: the view model holds cases, never a repository.** A case is three files: the interface the screen asks for, the implementation that holds the Core service, and the view model that injects the interface.

`gemcore/.../application/pricealerts/cases/SetPriceAlertsEnabled.kt` — the case:

```kotlin
interface SetPriceAlertsEnabled {
    suspend fun setPriceAlertsEnabled(enabled: Boolean)
}
```

`data/coordinators/.../pricealerts/PriceAlertsCoordinator.kt` — the implementation. It owns the
signal shared by all five price-alert cases; this excerpt shows the enabled read/write pair:

```kotlin
class PriceAlertsCoordinator(
    private val priceAlertService: GemPriceAlertService,
    private val getCurrentCurrency: GetCurrentCurrency,
) : GetPriceAlertsEnabled, SetPriceAlertsEnabled, IncludePriceAlert, ExcludePriceAlert, SetAssetPriceAlertEnabled {

    private val changes = MutableSharedFlow<Unit>()

    override fun isPriceAlertsEnabled(): Flow<Boolean> = changes
        .onStart { emit(Unit) }
        .map { priceAlertService.isEnabled() }

    override suspend fun setPriceAlertsEnabled(enabled: Boolean) {
        priceAlertService.setEnabled(enabled)
        changes.emit(Unit)
    }
}
```

Hilt builds it once and binds both interfaces to it, so the view model asks for the case it needs and nothing else:

```kotlin
@Provides @Singleton
fun providePriceAlertsCoordinator(
    priceAlertService: GemPriceAlertService,
    getCurrentCurrency: GetCurrentCurrency,
) = PriceAlertsCoordinator(priceAlertService, getCurrentCurrency)

@Provides
fun provideGetPriceAlertsEnabled(coordinator: PriceAlertsCoordinator): GetPriceAlertsEnabled = coordinator

@Provides
fun provideSetPriceAlertsEnabled(coordinator: PriceAlertsCoordinator): SetPriceAlertsEnabled = coordinator
```

**Two shapes, and the name says which:** `*Impl` is one case with no state — it forwards to Core,
or returns the Room `Flow` the database already makes reactive (`GetAssetPriceAlertState`).
`*Coordinator` implements the read and every writer for one subject and owns the signal between
them; use it when Core answers with a point read that screens must observe.

Prefer the coordinator over refreshing state in each view model: `PriceAlertsCoordinator` owns the
enabled read plus every writer that can change it, so all successful writes emit through the same
signal. A screen holding its own copy goes stale when another path enables an alert. If Core ever
publishes preference changes as a stream, this coordinator can collapse into stateless `*Impl`s.

**A case composing other cases is always fine** — `SyncAssetPriceAlertsImpl` holds `HasAssetPriceAlerts` and `UpdatePriceAlerts` — that is how a flow is assembled; what a case must not hold is a repository.

`features/settings/price_alerts/.../PriceAlertViewModel.kt` — the screen's view model, which injects the cases and never the service or a repository:

```kotlin
val priceAlertEnabled = getPriceAlertsEnabled.isPriceAlertsEnabled()
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

fun togglePriceAlerts(enable: Boolean) = viewModelScope.launch {
    setPriceAlertsEnabled.setPriceAlertsEnabled(enable)
}
```

A case may compose other cases ([`SyncAssetPriceAlertsImpl`](../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/pricealerts/SyncAssetPriceAlertsImpl.kt) calls `HasAssetPriceAlerts` and `UpdatePriceAlerts`), and it holds the `Gemstone*Store` adapter when the screen needs an observed read — never the DAO, and never a repository for data a Core service owns. The repositories left in the graph are legacy and shrink as services land; `SessionRepository` is the one still in wide use, and Core's `GemWalletSessionService` is replacing it.

**An iOS screen asks at most one Core service; an Android screen asks one or more narrow cases.** An iOS
view model that combines multiple `Gem*Service` protocols is doing the feature's job in the view
layer. Compose the decision in Core, while keeping platform-only ports explicit.

Rules belong in `gemstone`, not in an app class wrapping several services — `GemGateway.check_node` owns the node network-id check `AddNodeSceneViewModel` used to assemble. One constraint: `GemNodeService` cannot hold the gateway, because the gateway's transport picks node URLs *through* `GemNodeService`. Check where a service sits in that graph before giving it a new collaborator.

The same rule with each platform's noun, for the cases Core genuinely cannot answer:

- **iOS** — a feature service in `Features/<Feature>/Sources/Services/`, built in the app layer and passed into the view model's initializer. A feature package cannot read the app's `@Environment` service keys, so the service reaches it through the view model it is handed, never through an ambient registry.
- **Android** — a case in `gemcore` `application/<area>/cases/` with its implementation in `data/coordinators/<area>/`, injected into the view model by Hilt.

Still open: `AddNodeViewModel` on Android is the untouched twin — `NodeStatusService`, `AddNodeCase`, `SetCurrentNodeCase` and a raw `GemChainService` for the same network-id check, which `GemGateway.check_node` now answers. It belongs to the nodes work in flight.

**An app service holds Core services, not the tables Core owns.** Reaching into a `Gem*Store`'s table from the app is a second read path the owner cannot see — the same violation as a case taking a Room DAO. The exception is a table Core has no concept of: the recent-activity list is the app's own, so `RecentActivityStore` (iOS) and `RecentAssetsService` (Android) are the platform's query layer and stay.

Android's observed store readers are cases in `data/coordinators`, the documented home for an
observed read. iOS's confirm flow was the exception — `ConfirmSimulationService`,
`FeeAssetProvider` and `TransferMetadataProvider` read `AssetStore`, `BalanceStore` and
`PriceStore` directly. All three are deleted: the confirm service now asks the owners. The one
store read left in that flow is `AssetStore.getAssetData` for the *selected* fee asset, which is a
plain row lookup for rendering, not a rule.

The precedent that made this work: `GemWalletStore.get_wallets`/`get_wallet` are **synchronous** trait methods, so a service can answer a point read without `await`. Any future point read of a single row or short list should be synchronous the same way rather than pushing the caller back to the store.

`DeveloperViewModel` holds five stores to dump and reset them; it is a developer screen and stays as it is.

**Observed reads.** Core has no observation primitive, so a screen that must update as rows change observes the app's own database — iOS with `ObservableQuery` over a GRDB request, Android with a Room `Flow` returned by the case. Everything else — writes, remote sync, point reads, every decision — goes through the service.

**Tests.** iOS mocks the protocol from [`GemstoneServices/TestKit`](../ios/Packages/GemstoneServices/TestKit/); Android fakes the case interface, or mocks `Gem*Service` with MockK, using fixtures from `gemcore` `testFixtures`. Never mock a dependency-free constructible service (`GemAssetConfigService`, `GemChainService`, …) — construct the real one, or the test asserts the mock. Never fabricate I/O to reach a rule either: an offline `AlienProvider`, in-memory preference and secure stores and empty row stores, stood up so a test can touch rules that use none of them, is always the wrong answer — pass the answer in from the caller that owns the service, or mock the service and state the premise plainly. Neither app tests a rule that lives in Core — that test stays with its owning Core implementation.

### Done means

- Core has the flow, the rules and their tests; the app code it replaced is deleted in the same commit.
- Both apps implement the same store trait the same way, and both build and pass their suites.
- No app-side copy of a Core decision, no raw preference keys, no swallowed store failure, and no app service reading a table a `Gem*Store` owns.
- Nothing was added to reach it: iOS injects the protocol, Android calls a case — no wrapper service, no repository. A feature service or case that gathers several collaborators for one screen is not a wrapper; a class that forwards one call is.
- No `private let`/`private val` holding a `Gem*Service` at file scope. A service comes from the initializer or from Hilt, so a test can substitute it.
- Its store and both adapters are documented where the migration needs them, and its line in the
  plan below is removed.

## App services

What stays on the app side, because it is a platform concern with no Core counterpart:

| Service | Notes |
| --- | --- |
| [`AppService/RateService`](../ios/Packages/FeatureServices/AppService/RateService.swift) | App Store review prompt |
| [`AppService/AppLifecycleService`](../ios/Packages/FeatureServices/AppService/AppLifecycleService.swift) | Scene phase orchestration of observers |
| [`AppService/OnstartService`](../ios/Packages/FeatureServices/AppService/OnstartService.swift) | OS security checks, URL cache and launch orchestration |
| [`ConnectionStatusService`](../ios/Packages/FeatureServices/ConnectionStatusService) | Connectivity |
| [`StreamService`](../ios/Packages/FeatureServices/StreamService) | WebSocket lifecycle and Core stream adapter |
| [`WalletConnectorService`](../ios/Packages/FeatureServices/WalletConnectorService) | Reown/WalletConnect SDK integration |
| [`SystemServices`](../ios/Packages/SystemServices) | Connectivity, image gallery, local store |

Any other app-side service must own a real platform concern. A class that merely forwards a Core
call is migration debt. A Core export needs a real consumer, but both apps do not have to consume
the identical façade when iOS protocols and Android cases expose the same Core decisions; document
intentional one-sided integration surfaces.

## Remaining

- **iOS `Packages/GemAPI`** — one endpoint, one caller: `GemPriceWidget` reads asset prices with it. It stays. Routing the widget through Core would link the Rust library into an app extension that runs under a tight memory budget and makes a single GET, so the trade is wrong; nothing else in the app or the feature packages depends on the package. Android's equivalent is already down to the alien provider itself: `data/services/native-provider` is `NativeProvider` plus its cache, named after the trait it implements the way iOS's `NativeProviderService` package is.
- **iOS `Packages/GemstonePrimitives` remains mostly load-bearing.** A prior sweep removed declarations with no reader outside the package. What remains is primarily JSON bridge conformances, chain and stake config accessors, and typed wrappers over Core's JSON-string APIs — it shrinks when primitives stop crossing as JSON strings, not by chasing a line-count target.
- iOS `Primitives` keeps hand-written views of Core types (`GasPriceType`, `FeeRate`, `Fee`, `FeeSelection`, `CustomFeeEstimate`, `TransferAmount`, `BalanceRequirement`, and ids such as `WalletId`/`TransactionId`/`AssetId`). They stay: they are typed views bridged once at the seam, not a second source of truth.

## TODO — finish Core as the single owner of logic

The target shape every item below converges on is [ARCHITECTURE.md](ARCHITECTURE.md) — read it first.

Only open work lives here. When an item lands, delete its line in the same commit — do not leave "done" notes. When the list empties, audit again and write a new one.

### 1. Decisions someone has to make

Each is one question. Nothing below is blocked on investigation.

| Item | Question | Recommendation |
|---|---|---|
| S1 keystore password scope | `GemKeystorePassword` is per-wallet; neither platform honours it (iOS returns one global Keychain secret, Android writes the same global secret under each wallet id). Make it genuinely per-wallet with a re-encrypt migration, or drop `wallet_id` from the trait? | Drop the parameter. The contract currently promises something untrue; re-adding it later is cheap, a bad migration is not. |
| S3 biometric gate | iOS gates at the Keychain ACL so every secret read prompts; Android calls a UI prompt at each call site and `PasswordStore` itself is unauthenticated, so any new caller bypasses it. | Core should mark which operations require authentication, and the adapter enforces it. |
| N1 notification permission | Core owns "granted / denied / never asked", but Android's adapter holds an application `Context` and cannot tell "never asked" from "denied", so it opens Settings for a first-time user. | Core owns the three-state decision; Android needs an activity-scoped requester. |
| Push re-enable after opt-out | Nothing records "the user said no", so the 30-day re-ask silently re-enables push after a deliberate disable (`SettingsViewModel.kt:123-128` restarts the timer on **disable**; `PushRequest.kt:25-29` then finds permission still granted). | A preference in `GemPreferencesService`, checked before the re-ask. This is a live privacy defect, not just a gap. |
| T3 swap quote debounce | iOS 250 ms, Android 500 ms. Same question as the 250/500 ms name-resolution debounce. | Pick one and put it on `GemSwapQuoteService` beside `refresh_interval_milliseconds`. **Do not** delete Android's stop-on-failed-quote break: `quote errors do not schedule automatic retries` pins it. |
| Stake push notification target | iOS `case .stake: break` selects the wallet tab behind a TODO; Android opens the asset. | Navigation decision. |
| Swap minimum-amount button copy | iOS "Use Minimum Amount" (`swap_use_minimum_amount`); Android "Minimum amount" (`stake_minimum_amount`, borrowed from staking). | Product call — nobody has written swap-specific copy. |
| S8 privacy lock | iOS has an app-lock setting with a `shouldCoverScreen` rule and an overlay window; Android has none. | Product call. The cover predicate is Core's; the overlay is platform. |
| S9 WalletConnect one-click auth (SIWE) | Android only, and its rules — including *what the user is asked to sign* — live in `WCAuthViewModel.kt` UI code. | Product call. Whoever takes it moves the rules to Core first. |
| Polling on top of a live socket | T5/T7: screens still poll while a socket is open, and each screen asks for its own price subscription rather than Core deciding when prices are subscribed (deleting `PriceUpdater.swift`). | Design change, not a missing call. |
| Frozen `assetConfig` table | `Chain.asset()` builds an immutable lookup once at first access. Threading a service through ~21 Android sites so a pure function can read a constant costs every caller a parameter for nothing at runtime, and a frozen table cannot drift into an app-side variant. | Decide whether the no-service-at-a-call-site rule carves this out before spending the change. |

### 2. Unused generated models to remove

A sweep of the `#[typeshare]` types in `core/crates/primitives` against both apps' non-generated sources found 26 with no app reference. The nine standalone ones are removed (`CosmosDenom`, `QuoteAsset`, `SlippageMode`, `SwapProviderMode`, `SwapResult`, `SwapStatus`, `WCEthereumTransaction`, `WalletImport`). What is left:

- **Sixteen are nested** inside a type the apps do use — `StreamEvent` hosts six, `Markets` two, plus `CoreListItem`, `WalletSubscription`, `PortfolioAssets`, `FiatProvider`/`FiatQuote`, `RewardRedemptionOption`, `StreamMessage`, `WalletConfigurationResult`. Their generated model is still required; they go only when the host does.
- **`TransactionInputType`** is unreferenced but stays — it is the target of the transfer-model collapse in section 4.

Three gotchas if you repeat the sweep, all met on this pass:
1. A `#[typeshare(skip)]` on a *field* stops compiling once the struct attribute is removed, so it has to go with it.
2. Removing the last attribute in a file leaves `use typeshare::typeshare;` unused — clippy fails on it.
3. **The generator does not delete a file that now emits nothing.** `WalletImport.swift`, `WalletConnect.swift` and `swap/Result.kt` survived `just generate-models` with stale contents and had to be deleted by hand. Check `git status` for generated files that *did not* change and confirm they still have a source.

### 3. Rules still written once per platform

Each already sits on top of a Core call that exists; the app-side copy is pure duplication with no divergence found. One rule each, in Core, with a test that flips:

swap slippage bounds · min-receive BPS math · swap ETA truncation · the critical-warning gate · collections availability · the custom-fee minimum check.

### 4. Core surface

- **Two device API clients, and the split is load-bearing.** `deviceRegistrationClient` has no preflight and is what `GemDeviceService`/`GemSubscriptionService` use; the general client has one and is what every other service uses. That is what stops the sync path recursing into itself. `GemDeviceApiClient.set_device_sync_preflight` must only ever be called on the general client; nothing enforces it, so this note is the only record of it.

- **Receiver transport cleanup.** Android still reimplements `SimulationResult.asset_ids()` in
  `ConfirmViewModel.kt`; add the projection to the existing `GemSimulationFormatter` or fold the
  asset synchronization into the confirm aggregate, then delete the app copy. Do not restore it
  on `GemConfirmTransferService`. `GemContactsService` is the other transport debt: it only
  forwards update/delete to `GemContactService`, so iOS should use the owning service directly.
- **Confirm error conversion.** `GemConfirmService` repeats the same
  `GemServiceError` → `GemConfirmError::Load` closure across store reads. Implement the typed
  conversion once and use `?`; keep explicit mappings where the operation changes the category,
  such as `Record` or gateway-specific `Offline`/`Network`.
- **Android confirm error collapse.**
  [`ConfirmErrorMapper.kt`](../android/features/confirm/viewmodels/src/main/kotlin/com/gemwallet/android/features/confirm/viewmodels/ConfirmErrorMapper.kt)
  translates `GemConfirmException` into parallel
  [`ConfirmError`](../android/gemcore/src/main/kotlin/com/gemwallet/android/domains/confirm/ConfirmError.kt)
  cases for scan, network, fee and broadcast failures. Render or classify the typed Core error
  directly, delete those mirrored cases and their mapper tests, and retain only genuinely
  app-owned validation or presentation state.
- **Transfer model collapse.** Generate the `TransactionInputType` enum from typeshare so the primitives tuple enum, the gemstone named-field enum and the Swift/Kotlin enums become one (685 Core, 52 Android, 5 iOS references). Transaction construction is wallet-critical — do it only after both apps carry Core records through confirm. **Not started.**
- **One-sided exports**, each waiting on the other platform: `wallet_connect::authentication_chain_ids` (iOS WalletConnect auth), `nft::report` (Android report screen), `wallet_preferences::is_initial_load_completed` (iOS wallet empty state), `reset_transactions_timestamp` (iOS developer action).
- **`GemAssetConfigService` holders**: iOS `Chain+`, `AssetScore+`, `AssetProperties+`, `AssetBasic+`; Android `ext/Chain.kt`, `AssetDefaults.kt`. Blocked on the frozen-table decision above; Android is additionally blocked by `Migration_71_72`, a Room migration `object` that calls `chain.asset()` at database open where there is no graph to inject from.
- **`AddressFormatter` (iOS)**, 23 uses / ~60 construction sites. Attempted and reverted: threading the service reaches `WalletViewModel`, then spreads into `extension Wallet: SimpleListItemViewable`, which builds a `WalletViewModel` only to read `avatarImage` and never touches the formatter. The fix is smaller than the threading — split the display-only parts (`avatarImage`, `name`) from the address-formatting parts so only the latter needs the service. Design change, wants a decision.
- **`GemSecurityService` (iOS)** is a *defaulted* parameter on `BiometryAuthenticationService`. Making it required pushes the default into `SecurityViewModel` and `LockSceneViewModel`, which also default-construct the whole service — a lock-manager pass, not a one-liner.

### 5. Tests that cannot fail

`Migration_88_89Test.kt:35` seeds a multi-sig banner with `asset_id NULL` — the pre-`46889318bc` contract — and only calls `runMigrationsAndValidate`, which checks the schema and never asserts the row survived, so it cannot fail on data loss. It is an `androidTest`, so fixing it means running it on a device.

### 6. Android

- **Earn flow.** No Earn surface exists (no `StakeProviderType.Earn` reader, no `AmountParams.Earn`, no `ConfirmParams.Earn`; `GemDelegationAction.DEPOSIT` maps to nothing). Build the scene, amount provider and confirm params on `GemStakeService.sync_earn`/`get_earn_data`, `GemAmountType::Earn` and `GemTransactionInputType::Earn`; iOS `EarnSceneViewModel` + `AmountEarnViewModel` are the reference. A feature, not a consolidation — plan it as its own batch.
- **Dead `NOT NULL` columns** with no iOS counterpart: `AssetStore.saveAsset` bumps `updatedAt`, `TransactionStateStore` writes swap amounts, `NftStore` fills two legacy image columns. minSdk 28 has no `ALTER TABLE DROP COLUMN`, so removing them means recreating tables (`asset` behind its foreign keys) and instrumented migration tests do not run in CI — batch them with a migration that has another reason to touch those tables.
- `PriceStore` still stamps `prices.currency` (now only the label `AssetPriceInfo.currency` reads) and `mapNotNull`s unparsable ids where iOS maps straight through.
- **Node screens**: `AddNodeViewModel` holds `NodeStatusService`, `AddNodeCase`, `SetCurrentNodeCase` and a raw `GemChainService`, and does the network-id check itself — `GemGateway.check_node` answers it, as it does for iOS. Belongs to the nodes work in flight, along with the seven `cases/nodes/*Case` interfaces, the last of the legacy `cases/<area>/` tree.
- `UserConfig`: delete the `ConfigStore` fallback for `auth` once enough installs have written the secure value.
- Consistency: `toChain()` (nullable) and `requireChain()` (throws) are picked arbitrarily at call sites; `*Service` classes live inside the coordinators module.
- Localization: 59 hardcoded `dp` values (worst: `SupportMessageBubble`, `ReceiveScreen`, `ImportScreen`, `WalletTypeTab`, `FiatScene`).

### 7. iOS

- Naming: `GemstoneNftStore` wraps `NFTStore` and `ConnectionStore` wraps `ConnectionsStore` (one name per store); untyped `.map()` conversions where Android has `toPrimitives()`.
- `NavigationHandler`'s `.stake` deep link is an unimplemented branch and `TransactionScene`'s corner radius is an open iOS 26 styling question — both mark real gaps, keep the TODOs until closed.
- The two "delete in 2026" `FileMigrator` calls (`LocalKeystore`, `DB.swift`) move the keystore and database from documents to application support on launch. Deleting them strands anyone who has not opened the app since the move — losing their keystore — so this needs install-base data, not a code decision.

### 8. iOS view models holding more than one Core service

Each iOS scene view model should hold at most one private Core service per
[ARCHITECTURE.md § 7](ARCHITECTURE.md#7-at-most-one-core-service-on-ios-narrow-cases-on-android).
`ManageContactViewModel`, `ContactsViewModel` and `ManageContactAddressViewModel` meet the field-count
ceiling, but `GemContactsService` is forwarding-only and `GemManageContactService.names()` /
`addresses()` / `chains()` remain reach-through debt until the shared components receive narrow
dependencies directly. `ConfirmTransferSceneViewModel` is done: it holds `service` alone, with
`signer`, `keystore` and `recentAssetsService` as outbound ports the app implements, and takes
`currency` as a value rather than reading it from preferences.

The inventory below tracks remaining multi-service view models. Re-run the audit before a bulk
migration; a non-private service means the view is reaching through the model, so fix that first by
having the parent vend the child view model.

| view model | services | non-private |
|---|---|---|
| `Gem/ViewModels/RootSceneViewModel.swift` | 11 | 4 |
| `Assets/AssetSceneViewModel.swift` | 9 | 2 |
| `Assets/SelectAssetViewModel.swift` | 6 | 5 |
| `WalletTab/AssetsResultsSceneViewModel.swift` | 5 | 2 |
| `Settings/Settings/ViewModels/DeveloperViewModel.swift` | 5 | 0 |
| `Onboarding/ImportWalletViewModel.swift` | 4 | 4 |
| `Transactions/TransactionsViewModel.swift` | 4 | 2 |
| `WalletTab/WalletSearchSceneViewModel.swift` | 4 | 2 |
| `MarketInsight/ChartSceneViewModel.swift` | 4 | 1 |
| `WalletTab/WalletSceneViewModel.swift` | 4 | 1 |
| `Settings/Settings/ViewModels/NotificationsViewModel.swift` | 4 | 0 |
| `Swap/SwapSceneViewModel.swift` | 4 | 0 |
| `Settings/ChainSettings/ViewModels/ChainSettingsSceneViewModel.swift` | 3 | 2 |
| `NFT/CollectibleViewModel.swift` | 3 | 1 |
| `Perpetuals/PerpetualSceneViewModel.swift` | 3 | 1 |
| `Transactions/TransactionSceneViewModel.swift` | 3 | 0 |
| `Transfer/ReceiveViewModel.swift` | 3 | 0 |
| `Transfer/RecipientSceneViewModel.swift` | 3 | 0 |
| `WalletConnector/WalletConnector/ViewModels/SignMessageSceneViewModel.swift` | 3 | 0 |
| `Onboarding/ImportWalletTypeViewModel.swift` | 2 | 2 |
| `Perpetuals/PerpetualsSceneViewModel.swift` | 2 | 2 |
| `PriceAlerts/AssetPriceAlertsViewModel.swift` | 2 | 2 |
| `FiatConnect/FiatSceneViewModel.swift` | 2 | 1 |
| `ManageWallets/WalletIDetailViewModel.swift` | 2 | 1 |
| `Transfer/AmountEarnViewModel.swift` | 2 | 1 |
| `WalletConnector/WalletConnector/ViewModels/ConnectionsViewModel.swift` | 2 | 1 |
| `WalletTab/NetworkAssetsSceneViewModel.swift` | 2 | 1 |
| `Assets/AddAssetSceneViewModel.swift` | 2 | 0 |
| `Assets/AssetsFilterViewModel.swift` | 2 | 0 |
| `PriceAlerts/PriceAlertsSceneViewModel.swift` | 2 | 0 |
| `PriceAlerts/SetPriceAlertViewModel.swift` | 2 | 0 |
| `Settings/ChainSettings/ViewModels/AddNodeSceneViewModel.swift` | 2 | 0 |
| `Settings/Currency/ViewModels/CurrencySceneViewModel.swift` | 2 | 0 |
| `Settings/Settings/ViewModels/RewardsViewModel.swift` | 2 | 0 |
| `Stake/DelegationViewModel.swift` | 2 | 0 |
| `Stake/EarnSceneViewModel.swift` | 2 | 0 |
| `Stake/StakeSceneViewModel.swift` | 2 | 0 |
| `Stake/ValidatorSelectSceneViewModel.swift` | 2 | 0 |
| `Transfer/ConfirmDetailsViewModel.swift` | 2 | 0 |
| `PrimitivesComponents/AddressInputViewModel.swift` | 2 | 0 |

Single-service view models that only need the property made `private`:

- `Assets/AddAssetViewModel.swift` — `explorerService`
- `Settings/ChainSettings/ViewModels/ChainListSettingsViewModel.swift` — `chainService`
- `Swap/PriceImpactViewModel.swift` — `swapQuoteService`
- `Transfer/AmountPerpetualViewModel.swift` — `amountService`
- `Transfer/AmountStakeViewModel.swift` — `amountService`
- `Transfer/AmountTransferViewModel.swift` — `amountService`
- `WalletConnector/WalletConnector/ViewModels/ConnectionSceneViewModel.swift` — `connector`
- `WalletConnector/WalletConnector/ViewModels/WalletConnectionViewModel.swift` — `applicationMetadataService`

### 9. Rules still living in app-only enums

Both apps carry Core's `GemTransferData`, `GemConfirmInput` and `GemTransactionInputType`
end to end, per [ARCHITECTURE.md](ARCHITECTURE.md) § 6 — iOS's `TransferDataType` and
`TransferData` and Android's `ConfirmParams` are all gone.

`Recipient` is the twin that remains. It restates `GemRecipient` field for field on iOS,
and every transfer boundary now converts through `.gem` / `init(_:)`. Android has no such
twin. Removing it is the next consolidation.

Two recent-activity rules are still app-side, on `SelectAssetType` and `SelectedAssetType`.
Both are Swift-only enums with no Core counterpart, so the enums move first.

Android hand-wrote `RecentType` with different case names from the generated
`RecentActivityType` — `Send` against `Transfer`, `Buy` against `FiatBuy` — and those names
are persisted through `@SerialName`, so the platforms store different strings for the same
concept. Changing them needs a Room migration. Android also records nothing on a completed
transfer, which iOS has always done.

### Things that look like work and are not

Do not re-add these from a survey; each was checked against the code and found wrong or already done: **V5, N4, N6, V7, T4, S5, S6, V6**, backlog rows **1, 2, 4, 5, 7**, **T8**, the swap-pay recents row, and the `Transaction` typeshare model (`com.wallet.core.primitives.Transaction` is used across Android — do not drop its attribute).

Two warnings worth keeping:
- **The reconnect cap was a decision.** Android's 30 s was set deliberately (`d1cdb74`) so the price stream resumes within half a minute; iOS's 60 s was the untouched import default. Core took 30 s — do not "restore" 60 s.
- **Count the copies before adding one.** The candle interval mapping looked like two app-side copies; it was three — `candle_interval` in `gem_hypercore` has always driven the REST fetch, and the subscribed interval must equal the fetched one or the chart never updates. Check `core/crates/` before writing a rule that sounds new.
- **Core's `ValueFormatter` is not locale-aware.** Formatting amounts in Core regresses non-`en` locales, so Core hands over `{ value, decimals, symbol }` and each app formats. Applies to any future amount rule.

### How to work this list

- One change at a time: implement in Core → if a UniFFI signature, TypeShare model,
  `EXPOSED_TYPES` entry or mobile boundary changed, run `just generate` from the repo root (never
  while an iOS build is running or against half-edited Core) → wire both apps → delete the app code
  it replaces → verify → commit and push to `main` directly (no PR, no
  `Co-Authored-By`/session trailers) → fix red CI before anything else.
- Every commit that finishes an item removes its line from this file in the same commit.
- Pure feature rules go in `rules.rs`; intrinsic behavior and its tests may stay beside the owning
  type. Every decision has a unit test that would fail if the rule flipped. Services are
  `services/<name>/{mod,model,rules,store,error}.rs` with only the files they need. No code
  comments. No `utils`/`helper`/`fetch`/`resolve` names, no `tx`.
- Compare the pre-change app logic (`git show <sha>^:<path>`) with the Core rule before deleting it. When the platforms disagree, check for a test pinning the difference before picking a side — twice now the "wrong" platform was right and the divergence was a deliberate decision.
- Grep counts on property names are useless for sizing. Make the change, let the compiler count, and revert if it lands somewhere a dependency cannot go.

### Verification

- Core: `cargo fmt --all && cargo clippy -p gemstone --all-targets --all-features -- -D warnings && cargo test -p gemstone --lib --all-features`. CI also compiles the workspace with `--features unit_tests` and `chain_integration_tests`.
- Android: `just test` from `android/` plus `assembleGoogleDebug` (DI failures surface at assembly, not compile) and `assembleGoogleDebugAndroidTest` — `androidTest` sources are **not** compiled by `testGoogleDebugUnitTest`.
- iOS: `just build && just test` from `ios/`. A raw `xcodebuild` invocation must pass `GEMSTONE_LINKER_FLAGS` or every test bundle fails to link against the Rust library.

## Conventions

- Rust FFI signatures use domain types such as `WalletId`, `AssetId`, `Chain`, `NFTAssetId` and
  `Currency`, but current Swift/Kotlin bindings lower several of them to `String` typealiases. Map
  them to platform domain wrappers at the boundary; store row ids remain `String`.
- Store methods: `get_*` reads, `is_*` boolean reads, `set_*` preferences and stored flags or sets (`set_buyable_assets`, `set_assets_enabled`, `search::set_assets`), `save_*` upserts, `add_*` inserts that must not overwrite existing rows, `update_<items>(…, items, delete_ids)` for reconcile writes, `delete_*` removals, and `clear*` for wiping a whole scope (`preferences::clear`, `support::clear_typing`).
- Feature rules live in `rules.rs`; intrinsic receiver behavior may live beside the defining type.
  Reuse `testkit` mocks (`NFTData::mock_with`, `Asset::mock`, …) for shared fixtures, but a concise
  one-off literal is fine. Add a missing reusable mock to the owning crate's `testkit`. A
  `primitives` type may own structural invariants and transformations intrinsic to that type;
  feature or product policy and I/O orchestration stay in Gemstone.
- Chain icons come from the chain config, never an app-side list: `icon_chain` (an Ethereum layer 2 draws the Ethereum icon, SeiEvm draws Sei's) and `badge_chain` (the layer 2's own icon as the badge), both following `EVMChain::is_ethereum_layer2`.
