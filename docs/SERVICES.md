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

The service may own its feature store and holds `Arc`s of other Core services, never another domain's store or an app type. Narrow foreign traits are allowed for platform capabilities:

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

Everything that decides belongs in a pure function or receiver method with a test that fails if the rule flips. A failure is either impossible through a built-in default, surfaced as a typed error, or recorded through `services::failures::record` — never swallowed.

**There is no `*RulesService`.** A pure answer with one honest receiver belongs on that receiver, even when it takes additional plain values: `inputType.transactionAsset()`, `stakeBalance.showsStakeBalance()` and `contactAddressInput.addAddress(addresses)`. A pure rule with no honest receiver stays private in `rules.rs` and is called by the service. I/O, stores and platform ports belong on the service that owns the flow. See the complete decision matrix in [`ARCHITECTURE.md` § 6](ARCHITECTURE.md#6-where-derived-domain-answers-live).

A constructible service is **not** a licence to hold one at file scope. `private let addressService = GemAddressService()` or `private val assetConfig = GemAssetConfigService()` above a value-type extension is a hidden global: nothing can substitute it, and the extension reaches outward instead of receiving what it needs. A mocked Core rule is a premise, not a check, so its real mutation-checked test remains with its owning Core implementation; app tests assert only mapping, wiring and state.

The narrow exception is a dependency-free FFI transport adapter for a type whose receiver methods cannot cross the boundary. `GemSimulationFormatter`, `PriceAlertFormatter` and the config lookup `GemChainService` inside `NetworkSelectorViewModel` may be constructed locally; they have no state, I/O or substitutable dependency. Keep the adapter cohesive and do not create one object per method.

### 2. Pick the store the value belongs in

| What the service needs | Trait | Shape | iOS | Android |
| --- | --- | --- | --- | --- |
| rows in the database | one `Gem<Name>Store` per service ([example](../core/gemstone/src/services/price_alert/store.rs)) | point/short-list reads may be sync; writes and asynchronous reads are `async`; every method returns `Result<_, GemServiceError>` | GRDB adapter under [`GemstoneServices/Sources/Stores/`](../ios/Packages/GemstoneServices/Sources/Stores/) | Room adapter under [`data/services/gemstone/.../stores`](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/stores/) |
| a value the user set | [`GemPreferencesStore`](../core/gemstone/src/services/preferences/store.rs) through `GemPreferencesService` | sync; `get` returns `Option<String>` and **cannot fail** | `GemstonePreferencesStore` over `UserDefaults` | `GemstonePreferencesStore` over `SharedPreferences` |
| the same, per wallet | `GemWalletPreferencesStore` through `GemWalletPreferencesService` | sync, keyed by `WalletId` | same file layout | same file layout |
| a secret | [`GemSecureStore`](../core/gemstone/src/services/preferences/store.rs) | sync; **every read can fail** | `GemstoneSecurePreferencesStore` over the Keychain | `TinkGemPreferences` over Tink |
| something only the OS can do | a foreign trait of its own (`GemNotificationPermissions`, `GemStreamConnection`) | whatever the platform needs | app class | app class |

- One owning trait per persistence boundary. A second trait over the same rows is how the two apps drift apart; one cohesive feature trait may span closely related rows such as contacts and their addresses.
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

Android, `android/data/services/gemstone/.../stores/<Name>Store.kt`, class `Gemstone<Name>Store`, converting with the generated mapper:

```kotlin
class GemstonePriceAlertStore(
    private val priceAlertsDao: PriceAlertsDao,
) : GemPriceAlertStore {

    override suspend fun updatePriceAlerts(alerts: List<uniffi.gemstone.PriceAlert>, deleteIds: List<String>) {
        priceAlertsDao.update(alerts.map { it.toPrimitives().toRecord() }, deleteIds)
    }
}
```

The two adapters are mirrors: same methods, same conflict behaviour (upsert where the other upserts), same "write only rows whose values differ" rule, same treatment of a missing row. A difference between them is a bug in one of them, not a platform choice. Types retained as JSON custom types (`Account`, `Wallet`, `SimulationResult`, …) arrive as `String` typealiases and are decoded once at the relevant FFI/app boundary. That boundary may be a store adapter, coordinator, or feature mapper; undecoded JSON must not travel deeper into the app. Types listed in `core/bin/generate/remote_types.yml`, such as `Asset`, use generated structural mappers instead; enums listed there as codes (`Currency`) cross as their string code with a generated `Currency(core:)` / `toCurrency()`.

### 4. Construct it once

iOS builds the store and service once in [`ServicesFactory.swift`](../ios/Gem/Services/ServicesFactory.swift), then passes the service through [`ViewModelFactory.swift`](../ios/Gem/Services/ViewModelFactory.swift) to each scene that needs it:

```swift
let gemstonePriceAlertStore = GemstonePriceAlertStore(store: storeManager.priceAlertStore)
let priceAlertService = Gemstone.GemPriceAlertService(
    api: deviceApiClient,
    preferences: preferencesService,
    store: gemstonePriceAlertStore,
    device: deviceService,
    permissions: notificationPermissions,
)
```

Android provides the store and the service from one Hilt module ([`PriceAlertsModule`](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/di/PriceAlertsModule.kt)):

```kotlin
@Singleton @Provides
fun provideGemstonePriceAlertStore(priceAlertsDao: PriceAlertsDao): GemstonePriceAlertStore = GemstonePriceAlertStore(priceAlertsDao)

@Provides
fun provideGemPriceAlertStore(store: GemstonePriceAlertStore): GemPriceAlertStore = store

@Singleton @Provides
fun provideGemPriceAlertService(...): GemPriceAlertService = GemPriceAlertService(api, preferences, store, device, permissions)
```

### 5. Call the service directly; keep observed reads narrow

**iOS: the view model holds the Core protocol.** Nothing sits in between.

```swift
@Observable
@MainActor
public final class SupportChatSceneViewModel {
    private let service: any GemSupportServiceProtocol
    private let typing: ObservableSupportTyping

    public init(service: any GemSupportServiceProtocol, typing: ObservableSupportTyping) {
        self.service = service
        self.typing = typing
    }
}
```

`ViewModelFactory.supportChatScene()` passes the service into the view model. Feature packages do not read app-level environment service keys. **Do not write an app service around a Core service.** The distinction is which way the dependency points: a class that *implements* a Core trait is an adapter and is required (the `Gemstone*Store` classes, `GemstoneNotificationPermissions`, the WalletConnect signer); a class that only calls a Core service and re-exposes it is a wrapper, and the view model should hold the protocol instead.

**Android follows the same direct-service rule.** Hilt constructs one `GemPriceAlertService`, and [`PriceAlertViewModel`](../android/features/settings/price_alerts/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/price_alerts/viewmodels/PriceAlertViewModel.kt) injects it for Core-owned commands and point reads:

```kotlin
private val alertsEnabled = MutableStateFlow(service.isEnabled())

fun togglePriceAlerts(enable: Boolean) = viewModelScope.launch(Dispatchers.IO) {
    service.setEnabled(enable)
    alertsEnabled.value = service.isEnabled()
}
```

Android keeps a narrow case only when the screen needs a reactive Room read or app-side aggregation that the synchronous Core service does not provide. [`GetPriceAlertsImpl`](../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/pricealerts/GetPriceAlertsImpl.kt) and [`GetAssetPriceAlertStateImpl`](../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/pricealerts/GetAssetPriceAlertStateImpl.kt) observe the `GemstonePriceAlertStore`; they do not wrap Core commands. Such cases hold the `Gemstone*Store` adapter, never the DAO and never a repository for data a Core service owns.

**A screen asks at most one Core service.** Android may additionally inject narrow observed-read cases. A view model that combines multiple `Gem*Service` protocols is doing the feature's job in the view layer. Compose the decision in Core, while keeping platform-only ports explicit.

Rules belong in `gemstone`, not in an app class wrapping several services — `GemChainSettingsService.check_node` owns the URL rule, the network-id check and the node status `AddNodeSceneViewModel` used to assemble. The gateway's transport and `GemNodeService` read the selected node through the same `node_url` rule over the preferences store, so neither holds the other. Check where a service sits in that graph before giving it a new collaborator.

The same rule with each platform's noun, for the cases Core genuinely cannot answer:

- **iOS** — a feature service in `Features/<Feature>/Sources/Services/`, built in the app layer and passed into the view model's initializer. A feature package cannot read the app's `@Environment` service keys, so the service reaches it through the view model it is handed, never through an ambient registry.
- **Android** — a case in `gemcore` `application/<area>/cases/` with its implementation in `data/coordinators/<area>/`, injected into the view model by Hilt.

**An app service holds Core services, not the tables Core owns.** Reaching into a `Gem*Store`'s table from the app is a second read path the owner cannot see — the same violation as a case taking a Room DAO. The exception is a table Core has no concept of: the recent-activity list is the app's own, so `RecentActivityStore` (iOS) and `RecentAssetsService` (Android) are the platform's query layer and stay.

Android's observed store readers are cases in `data/coordinators`, the documented home for an observed read. iOS's confirm flow was the exception — `ConfirmSimulationService`, `FeeAssetProvider` and `TransferMetadataProvider` read `AssetStore`, `BalanceStore` and `PriceStore` directly. All three are deleted: the confirm service now asks the owners. The one store read left in that flow is `AssetStore.getAssetData` for the *selected* fee asset, which is a plain row lookup for rendering, not a rule.

The precedent that made this work: `GemWalletStore.get_wallets`/`get_wallet` are **synchronous** trait methods, so a service can answer a point read without `await`. Any future point read of a single row or short list should be synchronous the same way rather than pushing the caller back to the store.

`DeveloperViewModel` holds five stores to dump and reset them; it is a developer screen and stays as it is.

**Observed reads.** Core has no observation primitive, so a screen that must update as rows change observes the app's own database — iOS with `ObservableQuery` over a GRDB request, Android with a Room `Flow` returned by the case. Everything else — writes, remote sync, point reads, every decision — goes through the service.

**Tests.** iOS mocks the protocol from [`GemstoneServices/TestKit`](../ios/Packages/GemstoneServices/TestKit/); Android fakes the case interface, or mocks `Gem*Service` with MockK, using fixtures from `gemcore` `testFixtures`. Never mock a dependency-free constructible service (`GemAssetConfigService`, `GemChainService`, …) — construct the real one, or the test asserts the mock. Never fabricate I/O to reach a rule either: an offline `AlienProvider`, in-memory preference and secure stores and empty row stores, stood up so a test can touch rules that use none of them, is always the wrong answer — pass the answer in from the caller that owns the service, or mock the service and state the premise plainly. Neither app tests a rule that lives in Core — that test stays with its owning Core implementation.

### Done means

- Core has the flow, the rules and their tests; the app code it replaced is deleted in the same commit.
- Both apps implement the same store trait the same way, and both build and pass their suites.
- No app-side copy of a Core decision, no raw preference keys, no swallowed store failure, and no app service reading a table a `Gem*Store` owns.
- Nothing was added to reach it: both apps inject the generated Core service directly, with narrow Android cases only for observed reads or app-side aggregation — no forwarding wrapper and no repository.
- No `private let`/`private val` holding a `Gem*Service` at file scope. A service comes from the initializer or from Hilt, so a test can substitute it.
- Its store and both adapters are documented where the migration needs them, and its line in the plan below is removed.

## Screen services

One Core service per screen, held by the screen's view model on both apps. Re-run the holder sweep (`rg -l "Gem<Name>ServiceProtocol"` under `ios/Features`, `"Gem<Name>ServiceInterface"` under `android/features`) before adding a service: a screen service that only one app holds is the next consolidation, and a second Core service in a view model is the one to remove.

| Core service | iOS | Android |
| --- | --- | --- |
| `GemAddAssetService` | `AddAssetSceneViewModel` | `AddAssetViewModel` |
| `GemAmountService` | `AmountSceneViewModel` and its providers | `AmountViewModel`, `AmountPerpetualProvider` |
| `GemAssetDetailsService` | `AssetSceneViewModel` | `AssetDetailsViewModel` |
| `GemAssetSelectionService` | `SelectAssetViewModel`, `WalletSearchSceneViewModel`, `AssetsResultsSceneViewModel` | `BaseAssetSelectViewModel` and its subclasses |
| `GemChainSettingsService` | `ChainSettingsSceneViewModel`, `AddNodeSceneViewModel` | `NetworksViewModel`, `AddNodeViewModel` |
| `GemChartService` | `ChartSceneViewModel` | `ChartViewModel` |
| `GemCollectibleService` | `CollectibleViewModel`, `ReportNftViewModel` | `NftDetailsViewModel` (+ `GetNftAssetDetails` observed read) |
| `GemConfirmSession` (from `GemConfirmTransferService::session`) | `ConfirmTransferSceneViewModel` | `ConfirmViewModel` |
| `GemContactService` | `ContactsViewModel` | `ContactsViewModel` |
| `GemCurrencyService` | `CurrencySceneViewModel` | `CurrenciesViewModel` (+ session currency cases) |
| `GemDeveloperService` | `DeveloperViewModel` (+ the iOS stores it wipes) | `DevelopViewModel` |
| `GemFiatQuoteService` | `FiatSceneViewModel` | `FiatViewModel` |
| `GemManageContactService` | `ManageContactViewModel` (+ `nameService`) | `ManageContactViewModel` (+ `GemNameServiceInterface`) |
| `GemNotificationService` | `InAppNotificationsViewModel` | `InAppNotificationsViewModel` |
| `GemNotificationsService` | `NotificationsViewModel` | — (`SettingsViewModel` uses push cases) |
| `GemPerpetualDetailsService` | `PerpetualSceneViewModel` | `PerpetualDetailsViewModel` |
| `GemPerpetualService` | `PerpetualsSceneViewModel` (+ recent activity) | `PerpetualMarketViewModel` (+ recent activity) |
| `GemPortfolioService` | `PortfolioSceneViewModel` | `PortfolioChartViewModel` |
| `GemPriceAlertService` | `PriceAlertsSceneViewModel`, `SetPriceAlertViewModel` | `PriceAlertViewModel`, `PriceAlertTargetViewModel` |
| `GemReceiveService` | `ReceiveViewModel` | `ReceiveViewModel` |
| `GemRecipientService` | `RecipientSceneViewModel` (+ `nameService`) | `RecipientViewModel` (+ `GemNameServiceInterface`) |
| `GemRewardsService` | `RewardsViewModel`, `CreateRewardsCodeViewModel`, `RedeemRewardsCodeViewModel` | `ReferralViewModel` |
| `GemSignMessageService` | `SignMessageSceneViewModel` | `WCRequestViewModel`, `WCAuthViewModel` |
| `GemStakeService` | `StakeSceneViewModel`, `DelegationSceneViewModel`, `EarnSceneViewModel` | `StakeViewModel`, `DelegationViewModel` (earn flow missing, § 6) |
| `GemSupportService` | `SupportChatSceneViewModel` | `SupportChatSceneViewModel` |
| `GemSwapQuoteService` | `SwapSceneViewModel` | `SwapViewModel` |
| `GemTransactionDetailsService` | `TransactionSceneViewModel` | `GetTransactionDetailsImpl` (observed read + links) |
| `GemTransactionsService` | `TransactionsViewModel` | `TransactionsViewModel` |
| `GemWalletConnectService` | `WalletConnectorService` | `WCRequestViewModel`, `ProposalSceneViewModel`, `WCAuthViewModel` |
| `GemWalletHomeService` | `WalletSceneViewModel`, `NetworkAssetsSceneViewModel` | `AssetsViewModel`, `NetworkAssetsViewModel` |
| `GemWalletService` | onboarding and manage-wallet view models (`WalletsSceneViewModel` gates on `can_add_wallet`, `WalletDetailViewModel` exports the secret through `export_secret`) | `CreateWalletViewModel`, `ImportViewModel`, `WalletsViewModel` (`can_add_wallet`), `WalletViewModel` / `SetupWalletViewModel` (`rename`), `WalletSecretDataViewModel` (`export_secret`), wallet cases |

Android holds an observed Room read beside the service where the screen lists rows (a `Get*` case); that is the platform's reactive read, not a second service.

A view model depends on the generated abstraction, never the concrete Core class, so a test can substitute it. Re-run both sweeps before touching a screen: a hit means a second service crept in, or a Hilt module binds only the class.

```
rg -o "(let|var) \w+: (any )?Gem\w+Service(Protocol)?" ios/Features --glob '*ViewModel.swift'   # >1 per file, or no Protocol
rg -o "val \w+: Gem\w+Service\b" android/features --glob '*ViewModel*.kt'                       # concrete class
```

## App services

What stays on the app side, because it is a platform concern with no Core counterpart:

| Service | Notes |
| --- | --- |
| [`AppService/RateService`](../ios/Packages/FeatureServices/AppService/RateService.swift) | App Store review prompt |
| [`AppService/AppLifecycleService`](../ios/Packages/FeatureServices/AppService/AppLifecycleService.swift) | Scene phase orchestration of observers |
| [`AppService/OnstartService`](../ios/Packages/FeatureServices/AppService/OnstartService.swift) | OS security checks, URL cache and launch orchestration |
| [`ConnectionStatusService`](../ios/Packages/FeatureServices/ConnectionStatusService) | Connectivity |
| [`StreamService`](../ios/Packages/FeatureServices/StreamService) | Native socket observation and cancellation; `GemStreamService` owns session preparation, subscriptions, and currency selection on both apps |
| [`WalletConnectorService`](../ios/Packages/FeatureServices/WalletConnectorService) | Reown/WalletConnect SDK integration |
| [`SystemServices`](../ios/Packages/SystemServices) | Connectivity, image gallery, local store |

Any other app-side service must own a real platform concern. A class that merely forwards a Core call is migration debt. A Core export needs a real consumer, but both apps do not have to consume the identical façade when their generated bindings expose the same Core decisions; document intentional one-sided integration surfaces.

## Open work

Only open work lives here. When an item lands, delete its line in the same commit; a finished item leaves no note behind. When the list empties, audit again and write a new one. The shape everything converges on is [ARCHITECTURE.md](ARCHITECTURE.md).

### Decisions someone has to make

Each is one question, and none is blocked on investigation.

| Item | Question | Recommendation |
|---|---|---|
| Biometric gate | iOS gates at the Keychain ACL, so every secret read prompts. Android prompts at each call site and its password store is unauthenticated, so a new caller bypasses the gate. | Core marks which operations require authentication; the adapter enforces it. |
| Notification permission | Core owns "granted / denied / never asked", but Android's adapter holds an application context and cannot tell "never asked" from "denied", so it sends a first-time user to Settings. | Core keeps the three-state decision; Android needs an activity-scoped requester. |
| Privacy lock | iOS has an app-lock setting with a cover-screen rule and an overlay window; Android has none. | Product call. The cover predicate is Core's, the overlay is platform. |
| WalletConnect one-click auth | Android only, and its rules — including what the user is asked to sign — live in view-model code. | Product call. Whoever takes it moves the rules to Core first. |
| Earn visibility | Both apps hide the earn balance row outside a debug build, each with its own build check, so releasing the feature means flipping two switches. | Core should answer whether earn is offered, the way it answers the rest of the row. |
| Polling beside a live socket | Android polls nowhere, so adopting Core's refresh interval means adding timers to the activity, asset and perpetual screens: new background work, not a consolidation. | Product call on whether Android wants the safety net iOS has. |

### Android

- **Earn flow.** No Earn surface exists: no earn provider reader, amount params or confirm params, and Core's deposit delegation action maps to nothing. Build the scene, amount provider and confirm params on the earn methods Core already exports.
- **Dead `NOT NULL` columns** with no iOS counterpart: an asset `updatedAt` stamp, swap amounts on transaction state, two legacy NFT image columns, and the price currency column. minSdk 28 has no `ALTER TABLE DROP COLUMN`, so removing them means recreating the tables behind their foreign keys in one migration.
- **Secure auth fallback.** The config-store fallback for the auth value can go once enough installs have written the secure one; that is an install-base call, not a code call.

### iOS

- **Naming.** Untyped `.map()` conversions remain where Android names the direction (`toPrimitives()`).
- **Transaction scene corner radius** is an open iOS 26 styling question; it marks a real gap.
- **The two dated file migrations** move the keystore and database from documents to application support at launch. Deleting them strands anyone who has not opened the app since the move, losing their keystore, so this needs install-base data.

### Deliberate divergences — do not "fix" these

- The stream reconnect cap is 30 seconds because the price stream must resume within half a minute. The 60 seconds one app used to carry was an untouched import default, not a decision.
- The confirm button on the autoclose screen differs by design: one app enables it on any pending change and reveals validation after a tap, the other keeps it disabled until the change can build. Both read the same Core outcome; only the moment errors appear differs.
- A view model may hold more than one Core service when it is a launch host or a flow parent vending child models, and the extra services are private. A non-private service is the real defect: the view is reaching through the model, so have the parent vend the child model instead.

## Verification

- Core, from `core/`: `just lint` and `cargo test -p gemstone --lib --all-features`. CI also compiles the workspace with `--features unit_tests` and `chain_integration_tests`.
- Android, from `android/`: `just test`, `./gradlew assembleGoogleDebug`, and `./gradlew assembleGoogleDebugAndroidTest`. DI failures surface at assembly, not compile, and `androidTest` sources are not compiled by the unit-test task.
- iOS, from `ios/`: `just build && just test`, after `just generate-stone` when a Core FFI signature changed. A raw `xcodebuild` invocation must pass `GEMSTONE_LINKER_FLAGS` or every test bundle fails to link.

## Conventions

- Rust FFI signatures use domain types such as `WalletId`, `AssetId`, `Chain`, `NFTAssetId` and `Currency`, but current Swift/Kotlin bindings lower several of them to `String` typealiases. Map them to platform domain wrappers at the boundary; store row ids remain `String`.
- Store methods: `get_*` reads, `is_*` boolean reads, `set_*` preferences and stored flags or sets (`set_buyable_assets`, `set_assets_enabled`, `search::set_assets`), `save_*` upserts, `add_*` inserts that must not overwrite existing rows, `update_<items>(…, items, delete_ids)` for reconcile writes, `delete_*` removals, and `clear*` for wiping a whole scope (`preferences::clear`, `support::clear_typing`).
- Feature rules live in `rules.rs`; intrinsic receiver behavior may live beside the defining type. Reuse `testkit` mocks (`NFTData::mock_with`, `Asset::mock`, …) for shared fixtures, but a concise one-off literal is fine. Add a missing reusable mock to the owning crate's `testkit`. A `primitives` type may own structural invariants and transformations intrinsic to that type; feature or product policy and I/O orchestration stay in Gemstone.
- Core never makes a caller wait on background work. `GemConfirmService::execute` broadcasts, stores the transaction as pending and returns the hashes; keeping that transaction's status current runs through `GemTransactionStatusService`, a foreign port both apps implement by scheduling `GemTransactionStateService::track` off-thread, because Gemstone has no async runtime of its own. Awaiting the poll instead leaves the confirm screen spinning until the transaction is final.
- Every dependency a service takes is a `Gem*Service`, whether Core owns it or the app implements it as a foreign port. Name a port for the domain it serves, not for the mechanism.
- Chain and asset icons come from Core, never an app-side list: `ChainConfig.icon_chain` is the chain's own logo and `GemAssetConfigService::asset_icon` decides an asset's image and badge (an Ethereum layer 2's native coin draws as Ethereum only when that coin is ETH; `EVMChain::is_ethereum_layer2` alone also covers layer 2s with their own gas coin).
