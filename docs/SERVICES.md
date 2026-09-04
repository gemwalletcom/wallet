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
cannot cross the boundary. `GemSimulationFormatter`, `PriceAlertFormatter` and the config lookup
`GemChainService` inside `NetworkSelectorViewModel` may be constructed locally; they have no state,
I/O or substitutable dependency. Keep the adapter cohesive and do not create one object per method.

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

`ViewModelFactory.supportChatScene()` passes the service into the view model. Feature packages do not read app-level environment service keys. **Do not write an app service around a Core service.** The distinction is which way the dependency points: a class that
*implements* a Core trait is an adapter and is required (the `Gemstone*Store` classes,
`GemstoneNotificationPermissions`, the WalletConnect signer); a class that only calls a Core
service and re-exposes it is a wrapper, and the view model should hold the protocol instead.

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

Rules belong in `gemstone`, not in an app class wrapping several services — `GemChainSettingsService.check_node` owns the URL rule, the network-id check and the node status `AddNodeSceneViewModel` used to assemble. One constraint: `GemNodeService` cannot hold the gateway, because the gateway's transport picks node URLs *through* `GemNodeService`. Check where a service sits in that graph before giving it a new collaborator.

The same rule with each platform's noun, for the cases Core genuinely cannot answer:

- **iOS** — a feature service in `Features/<Feature>/Sources/Services/`, built in the app layer and passed into the view model's initializer. A feature package cannot read the app's `@Environment` service keys, so the service reaches it through the view model it is handed, never through an ambient registry.
- **Android** — a case in `gemcore` `application/<area>/cases/` with its implementation in `data/coordinators/<area>/`, injected into the view model by Hilt.

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
- Nothing was added to reach it: both apps inject the generated Core service directly, with narrow Android cases only for observed reads or app-side aggregation — no forwarding wrapper and no repository.
- No `private let`/`private val` holding a `Gem*Service` at file scope. A service comes from the initializer or from Hilt, so a test can substitute it.
- Its store and both adapters are documented where the migration needs them, and its line in the
  plan below is removed.

## Screen services

One Core service per screen, held by the screen's view model on both apps. Re-run the holder sweep
(`rg -l "Gem<Name>ServiceProtocol"` under `ios/Features`, `"Gem<Name>ServiceInterface"` under
`android/features`) before adding a service: a screen service that only one app holds is the next
consolidation, and a second Core service in a view model is the one to remove.

| Core service | iOS | Android |
| --- | --- | --- |
| `GemAddAssetService` | `AddAssetSceneViewModel` | `AddAssetViewModel` |
| `GemAmountService` | `AmountSceneViewModel` and its providers | `AmountViewModel`, `AmountPerpetualProvider` |
| `GemAssetDetailsService` | `AssetSceneViewModel` | `AssetDetailsViewModel` |
| `GemAssetSelectionService` | `SelectAssetViewModel`, `WalletSearchSceneViewModel`, `AssetsResultsSceneViewModel` | `BaseAssetSelectViewModel` and its subclasses |
| `GemChainSettingsService` | `ChainSettingsSceneViewModel`, `AddNodeSceneViewModel` | `NetworksViewModel`, `AddNodeViewModel` |
| `GemChartService` | `ChartSceneViewModel` | `ChartViewModel` |
| `GemCollectibleService` | `CollectibleViewModel`, `ReportNftViewModel` | `NftDetailsViewModel` (+ `GetNftAssetDetails` observed read) |
| `GemConfirmTransferService` | `ConfirmTransferSceneViewModel` | `ConfirmViewModel` |
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

Android holds an observed Room read beside the service where the screen lists rows (a `Get*` case);
that is the platform's reactive read, not a second service.

Audit of September 2026 (re-run the two sweeps below before touching a screen): every iOS screen
view model holds one `any Gem*ServiceProtocol` — the only multi-service holders are the three
in section 8 — and no view model on either app names a concrete Core class any more (iOS
`DeveloperViewModel` did; eighteen Android view models and their `pack`/`unpack`,
`toSupportedNamespaces`, `activityDefaults` helpers took `Gem*Service` where the Hilt module only
bound the class, so each such module now also binds the `*Interface`).

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
| [`StreamService`](../ios/Packages/FeatureServices/StreamService) | WebSocket lifecycle and Core stream adapter |
| [`WalletConnectorService`](../ios/Packages/FeatureServices/WalletConnectorService) | Reown/WalletConnect SDK integration |
| [`SystemServices`](../ios/Packages/SystemServices) | Connectivity, image gallery, local store |

Any other app-side service must own a real platform concern. A class that merely forwards a Core
call is migration debt. A Core export needs a real consumer, but both apps do not have to consume
the identical façade when their generated bindings expose the same Core decisions; document
intentional one-sided integration surfaces.

## Remaining

- **Hosts live in Core.** `GemApiClient`, `GemDeviceApiClient` and `GemStaticApiClient` construct on the production hosts, `GemDeviceRequestSigner::device_stream_request` returns the socket URL with its `Authorization` header, and `WalletConnectConfig` carries the project id and the app metadata Reown needs, so neither app keeps an assets or stream URL constant (Android's `Constants` object is gone; iOS keeps the app-group identifier and `apiURL`, which only the widget's `GemAPI` reads, for the reason below).
- **iOS `Packages/GemAPI`** — one endpoint, one caller: `GemPriceWidget` reads asset prices with it. It stays. Routing the widget through Core would link the Rust library into an app extension that runs under a tight memory budget and makes a single GET, so the trade is wrong; nothing else in the app or the feature packages depends on the package. Android's equivalent is already down to the alien provider itself: `data/services/native-provider` is `NativeProvider` plus its cache, named after the trait it implements the way iOS's `NativeProviderService` package is.
- **iOS `Packages/GemstonePrimitives` remains mostly load-bearing.** A prior sweep removed declarations with no reader outside the package. What remains is primarily JSON bridge conformances, chain and stake config accessors, and typed wrappers over Core's JSON-string APIs — it shrinks when primitives stop crossing as JSON strings, not by chasing a line-count target.
- iOS `Primitives` keeps hand-written views of Core types (`FeeRate`, `FeeSelection`, `TransferAmount`, `BalanceRequirement`, and ids such as `WalletId`/`TransactionId`/`AssetId`). They stay: they are typed views bridged once at the seam, not a second source of truth — a view carries Core's answer as data (`BalanceRequirement.shortfall` is stored, never recomputed in Swift). Navigation inputs that carry a Core record (`RecipientData`, `AmountInput`, `SelectAssetType`, `SelectedAssetInput`, `ChainRecipient`) live in `GemstonePrimitives`, because `Primitives` cannot import `Gemstone`.

## TODO — finish Core as the single owner of logic

The target shape every item below converges on is [ARCHITECTURE.md](ARCHITECTURE.md) — read it first.

Only open work lives here. When an item lands, delete its line in the same commit — do not leave "done" notes. When the list empties, audit again and write a new one.

### 1. Decisions someone has to make

Each is one question. Nothing below is blocked on investigation.

| Item | Question | Recommendation |
|---|---|---|
| S3 biometric gate | iOS gates at the Keychain ACL so every secret read prompts; Android calls a UI prompt at each call site and `PasswordStore` itself is unauthenticated, so any new caller bypasses it. | Core should mark which operations require authentication, and the adapter enforces it. |
| N1 notification permission | Core owns "granted / denied / never asked", but Android's adapter holds an application `Context` and cannot tell "never asked" from "denied", so it opens Settings for a first-time user. | Core owns the three-state decision; Android needs an activity-scoped requester. |
| Stake push notification target | iOS `case .stake: break` selects the wallet tab behind a TODO; Android opens the asset. | Navigation decision. |
| Swap minimum-amount button copy | iOS "Use Minimum Amount" (`swap_use_minimum_amount`); Android "Minimum amount" (`stake_minimum_amount`, borrowed from staking). | Product call — nobody has written swap-specific copy. |
| Swap asset selection enables the asset | Android's `SwapViewModel.updateBalance` enables a selected pay/receive asset on the current wallet (`EnableAsset`, one of the last `getSession` reads in a view model) and subscribes its price; iOS only subscribes prices when a quote loads. | Product call. If enabling stays, it becomes `GemSwapQuoteService::select_asset(asset_id)` reading the session, on both apps. |
| Wallet header swap button | Android's wallet header shows Swap when `GetWalletSummaryImpl.isSwapAvailable` says so (multicoin, or a single-chain wallet whose chain swaps; never a view wallet); iOS's `WalletHeaderViewModel` has no swap button. | Product call. If iOS gets the button, the rule moves to `GemWalletHomeService` first so both read it. |
| S8 privacy lock | iOS has an app-lock setting with a `shouldCoverScreen` rule and an overlay window; Android has none. | Product call. The cover predicate is Core's; the overlay is platform. |
| S9 WalletConnect one-click auth (SIWE) | Android only, and its rules — including *what the user is asked to sign* — live in `WCAuthViewModel.kt` UI code. | Product call. Whoever takes it moves the rules to Core first. |
| Polling on top of a live socket | T5/T7: screens still poll while a socket is open, and each screen asks for its own price subscription rather than Core deciding when prices are subscribed (deleting `PriceUpdater.swift`). | Design change, not a missing call. |
| Frozen `assetConfig` table | `Chain.asset()` builds an immutable lookup once at first access. Threading a service through ~21 Android sites so a pure function can read a constant costs every caller a parameter for nothing at runtime, and a frozen table cannot drift into an app-side variant. | Decide whether the no-service-at-a-call-site rule carves this out before spending the change. |

### 2. Unused generated models to remove

A sweep of the `#[typeshare]` types in `core/crates/primitives` against both apps' non-generated sources found 26 with no app reference. The nine standalone ones are removed (`CosmosDenom`, `QuoteAsset`, `SlippageMode`, `SwapProviderMode`, `SwapResult`, `SwapStatus`, `WCEthereumTransaction`, `WalletImport`). What is left:

- **Sixteen are nested** inside a type the apps do use — `StreamEvent` hosts six, `Markets` two, plus `CoreListItem`, `WalletSubscription`, `PortfolioAssets`, `FiatProvider`/`FiatQuote`, `RewardRedemptionOption`, `StreamMessage`, `WalletConfigurationResult`. Their generated model is still required; they go only when the host does.
- **`TransactionInputType`** is unreferenced but stays — it is the target of the transfer-model collapse in section 4.
- A later pass (September 2026) found `BalanceType` — a file that was not even in `lib.rs` — and
  `AssetRank`, which only `AssetScore::rank_type` (a skipped field) uses in Core; the first is
  deleted with its two generated files, the second is no longer shared. `AssetScoreType` went
  with the verification-status move. A third pass un-shared `WalletConnectionEvents` (Core's
  `rules::` enumerates it; neither app named it). The other unreferenced names the sweep still
  prints — `AddressChains`, `TransactionWalletConnectMetadata`, the `Stream*`/`Support*` event
  payloads, `RewardLevel`/`RewardRedemptionType`, `WebSocketPricePayload` — are nested in a
  `json_bridge!` type (`WalletSubscription`, `Transaction` metadata, `StreamEvent`, `Rewards`)
  the apps decode, so they stay.

Three gotchas if you repeat the sweep, all met on this pass:
1. A `#[typeshare(skip)]` on a *field* stops compiling once the struct attribute is removed, so it has to go with it.
2. Removing the last attribute in a file leaves `use typeshare::typeshare;` unused — clippy fails on it.
3. **The generator does not delete a file that now emits nothing.** `WalletImport.swift`, `WalletConnect.swift` and `swap/Result.kt` survived `just generate-models` with stale contents and had to be deleted by hand. Check `git status` for generated files that *did not* change and confirm they still have a source.

### 3. Rules still written once per platform

- **The amount providers still mirror each other.** `GemAmountType::validate` now checks a value
  against the type's own available value and minimum, so neither app hands those in, and
  `GemAmountService { stake, preferences }` answers the currency, the earn data and the perpetual
  defaults on both apps: `perpetual_leverage(max_leverage)` picks the preferred option and
  `perpetual_autoclose(price, direction, leverage)` turns the preference percents into target
  prices, so neither app reads the perpetual preferences or runs the estimator itself (Android's
  `AmountPerpetualProvider` took `UserConfig` for that, and `AmountViewModel` read the currency
  off the price with a `USD` fallback). The stake and earn confirm transfers are Core's too:
  `stake_transfer_data(asset, stake_type, value, use_max_amount)` (on `GemAmountService` and
  `GemStakeService`, for the amount screen and the stake/delegation screens) names the validator
  as the recipient, carries the resource for a freeze, and keeps max only for a new stake or a
  freeze; `GemAmountService::earn_transfer_data(asset, earn_type, value, use_max_amount)` finds
  the account on the session wallet, asks the gateway for the contract call and addresses it to
  the contract under the provider name, so `get_earn_data` is no longer exported. Android's
  `GemTransferData.stake` (with its `StakeType.validatorId` switch) and iOS's inline
  `TransferData(...)` builders are gone. What is left is the provider layer itself: the iOS providers (`AmountTransferViewModel`,
  `AmountStakeViewModel`, `AmountPerpetualViewModel`, `AmountEarnViewModel`) and Android's
  `providers/*` each build the type-specific confirm input; the screen state is one Core answer,
  `GemAmountType::input(asset, balance) -> GemAmountInput { available_value, max_value,
  reserved_fee: Option, can_change_value, shows_asset_balance }` (the former `rules` + `limits`
  pair — `reserved_fee` is `Some` exactly when the max keeps a fee back, which is the note both
  screens show at max; the minimum is Core's to validate, not the app's to read), so iOS's
  `AmountDataProvidable` extension and Android's five derived provider flows are one `input`
  each, and the perpetual order is Core's too:
  `GemPerpetualDetailsService::position_action` builds the action the position screen hands over
  and `GemAmountService::perpetual_transfer_data` turns it into the transfer, so the per-app
  `PerpetualTransferData`, `PerpetualPositionAction`, `PerpetualOrderFactory` and
  `PerpetualOrder+GemstonePrimitives` are gone; the chart takes the `Perpetual` itself —
  `GemPerpetualDetailsService::{candlesticks, candle_subscription, market_subscription}` derive
  the Hyperliquid symbol and `apply_candle_update(candles, update, perpetual, period)` answers
  whether a socket candle belongs to this chart and merges it, so the per-app `coin` derivation
  and the interval/coin filters are gone (`limits` is infallible — the available value exists for
  every type — so neither app carries a catch that handed back zero limits; the balance they hand
  in is the one `GemAssetBalance` record, bridged once per app). What is left per app is the
  input text ↔ value conversion (both through Core converters) and the type-specific
  `makeTransferData` / `buildTransfer` dispatch.
- **The generator is the way a duplicated type stops being duplicated.** Adding a fieldless enum or
  a scalar-field record to `core/bin/generate/remote_types.yml` replaces a hand-written mapper on
  both apps at once: `Account`, `Chain`, `ChainType`, `ConnectionStatus`, `ConnectionComponent`,
  `LinkType`, `PriceAlertDirection`, `PriceAlertNotificationType`, `AssetFiatValue`,
  `TotalFiatValue` and `SwapProvider` moved that way, each deleting an app-side copy and, in four
  cases, a rule the two platforms disagreed on. What it cannot represent yet is a data-carrying
  enum: `remote_mappers.rs` reads variant names only, so `GasPriceType` would generate an empty
  mapping. That blocks the last hand-written fee mappers (`GemGasPriceType` ↔ `GasPriceType`,
  `GemTransactionLoadFee` ↔ `Fee`, `GemFeeOptions` ↔ `FeeOptionMap`), where the same conversion is
  written twice. The fee chain no longer needs it: `Fee`, `GasPriceType`, `FeeOption`/`FeeOptionMap`
  and `TransferDataExtra` are deleted from iOS `Primitives`, because nothing read them — the confirm
  screen takes `GemTransactionLoadFee` and `GemTransferDataExtra` straight from Core, as Android
  always has. Teaching the generator about associated values is still what a data-carrying enum
  needs before it can cross typed.
- **Screens read their rows from one Core answer.** `GemTransactionDetailsService::details`
  (swap progress steps, swap-again, provider name, confirmation ETA, pnl, price) and
  `GemAssetBalance::detail_rows` (available, staked, earn, pending, reserved) replaced the
  per-row predicates both apps kept — Android had mapped the transaction state to swap-progress
  step statuses a third time inside a Compose item. Left on the apps: formatting, titles, and the
  DEBUG gate iOS keeps on the earn row.
- **The WalletConnect request lifecycle is one Core call.**
  `GemWalletConnectService::process_request` takes the SDK's session request as it arrives
  (topic, request id, method, params, chain id, origin, verification) and returns
  `GemWalletConnectOutcome { response, failure }`: it ignores a redelivered request id
  (`response: None`, nothing is sent — the first delivery is still being decided or was already
  answered), looks the connection up in its own store, rejects an unverified or malicious origin,
  rejects an EVM transaction whose `from` is not the session account before it is simulated
  (#1012), runs the request through the signer port and maps a cancel to a silent user-rejected reply and
  any other error to `Failed { message }`. Both apps send `response` back to the SDK when there is
  one and show `failure` if there is one; iOS's `handleRequest`/`rejectRequest` and Android's `WalletConnectRequestHandler`, the
  connection lookup and the per-request origin check are gone, and `WCRequestViewModel` holds
  the Core service plus the SDK responder, the pending-request port, `GemSignMessageService` and the
  active-request tracker. `GemSignMessageService { names, explorer }` answers the preview (text,
  payload fields and `has_critical_warning`, which is what disables the sign button — the
  per-app "any warning is critical" helpers are gone; the confirm screen reads the same flag off
  `GemConfirmSimulation`), the payload address names and the explorer links for the
  sign-message screen on both apps.

### 4. Core surface

- **Current-wallet screen services read the session.** Every Android view model that calls a
  wallet-scoped Core method starts with `session.value?.wallet ?: return@launch`, and every iOS one
  threads `wallet.id.id`, because the method takes the wallet as an argument. Core owns the current
  wallet (`GemWalletSessionService`), so a service whose screen only ever acts on the current
  wallet takes the session and drops the parameter — `GemRecentActivityService::add_recent`,
  `GemNotificationService::open` and `GemWalletHomeService` are converted (`current_wallet_id()`
  / `current_wallet()` on the session are the Core-internal accessors that fail with `NotFound`).
  `GemAssetDetailsService`, `GemPerpetualDetailsService`, `GemAssetSelectionService`,
  `GemSwapQuoteService`, `GemFiatQuoteService` and `GemTransactionsService::sync` too (Android's
  `RequestSwapQuotesImpl` now asks the quote service, not `GemSwapService`, for quotes; the
  Core-internal wallet-scoped sync is `sync_wallet`), and `GemPerpetualService::sync_enablement`
  / `should_connect_perpetuals` (the `Option<Wallet>` argument is gone), and
  `GemStakeService::{sync, sync_earn}`, which take the chain or asset and look the account up on
  the session wallet (`sync_wallet` / `sync_earn_wallet` stay for the transaction-state
  post-processing), and `GemNftService::sync` (`sync_wallet` underneath), and
  `GemConfirmTransferService` (`confirm_input(transfer)` picks the signing account; `load` and
  `execute` read the wallet), so every screen hands the confirm flow a `GemTransferData` and no
  Android call site looks an account up any more, and `GemRecentActivityService::clear(types)`
  reads the wallet too (Android's `ClearRecentAssets` use case is gone; `RecentsSheetViewModel`
  holds the Core service), and the perpetual screens refresh positions through
  `GemPerpetualService::sync_current_positions` / `GemPerpetualDetailsService::sync_positions`,
  which find the Hyperliquid account on the session wallet (the app-side `hyperliquidAccount`
  rule and the observer's `update(wallet)` are gone from both apps; `sync_positions(wallet_id,
  chain, address)` and `account_mode` are Core-internal, used by the socket connection). The
  batch is done; what remains on Android is the welcome-banner key, reading the session for a
  wallet id Core could hand out. `GemBalanceService`,
  `GemSwapService` and the socket-driven `GemPerpetualService::{connection, sync_positions,
  apply_socket_message}` stay explicit underneath: the app-start and observer flows call them
  for the wallet whose socket they hold. Keep an explicit
  wallet only where a screen acts on a wallet that is not current (`GemWalletService` rename,
  delete, pin, `export_secret`; `GemAppStartService::setup_wallet`).

- **Big integers are typed on both sides of the boundary.** `uniffi.toml` maps `GemBigInt` /
  `GemBigUint` to `java.math.BigInteger` and `BigInt` / `BigUInt`, so no app code parses a Core
  amount or renders one to a string to hand it back; `GemAmountError`, `GemAmountType::validate`
  and `GemTransferDataExtra.gas_limit` carry big integers too. What still parses is the
  typeshare model (`Balance`, `SwapQuote`, `Delegation.base`, `TransactionSwapMetadata`) and the
  database columns both apps store as text — a typeshare-level mapping would finish it.

- **The network fee sheet is one Core answer.** `GemConfirmData::fee_rate_rows(selection, fee_asset) -> GemFeeRateRows { rows: [{ priority, unit_value, fee }], unit_type, unit_decimals, supports_custom_fee, selected_total, normal_total }` scales the loaded fee to each rate against the selected base, picks the unit (`FeeUnitType` is a remote enum now) and its decimals, and says whether a custom rate can be entered (bitcoin chains with more than one rate). Both apps' copies of that — iOS `estimatedFee`/`feeRateDecimals`/`Chain.customFeeEnabled`/`FeePriority.rank` and Android `FeeDetailsModel.from`/`FeeRateUIModel.feeAmount`/`CustomFee.baseTotal`/`feeRateDecimals`/`Chain.feeUnitType()` — are gone; the app formats a row and drives the custom-fee input from `selected_total`/`normal_total`. `fee_rates` themselves arrive sorted (normal, then fast). `Primitives.FeeRate` and the `GemFeeRate` mapping on iOS, and `SignerParams.feeRates` on Android, had no reader left and are deleted.

- **The asset status row is one Core answer.** `VerificationStatus::from_rank(rank)` holds the
  score thresholds (suspicious up to 5, unverified up to 15) that the banner rule and the asset
  scene both read; `GemAssetDetailsService::verification_status(asset, rank)` says which status
  the asset scene shows as a row (an unverified token; a suspicious one is already the banner,
  and a native asset never shows one). iOS `AssetScoreTypeViewModel(score:)` and Android
  `StatusItem.assetVerification` carried those thresholds, and Android also rowed suspicious
  tokens next to the banner — that is aligned to the iOS behaviour. The typeshare-only
  `AssetScoreType` enum (a copy of `VerificationStatus`) and the iOS bridge between the two are
  deleted; `VerificationStatus` is a remote enum now instead of a dead JSON bridge.

- **The asset scene's network row is one Core answer.** `GemAssetDetailsService::network_destination(asset_id) -> Option<GemAssetNetworkDestination { Asset { asset }, Assets { chain } }>`
  opens the chain's native asset for a token on a chain that has one, the chain's asset list
  otherwise, and nothing on a chain without tokens. iOS `AssetNetworkDestination` and
  `AssetSceneViewModel.networkDestination`, and Android `Asset.networkNavigationAction`, carried
  that rule (with `Chain.hasNativeAsset` / `isTokenSupported` config reads); iOS
  `Chain.hasNativeAsset` had no reader left and is deleted.

- **Opening an asset for the current wallet is one Core call.** `GemAssetsService` holds the
  session now and `open_asset(asset_id)` reads the current wallet itself before
  `open_wallet_asset` (chain in the wallet, native asset exists, asset and balance rows ensured).
  Android's `AssetNavigationPolicy` re-derived the "can open" half of that rule from
  `Chain.hasNativeAsset()` and gated every in-app asset open with it while notification and
  deep-link opens skipped Core; the navigator, the notification routes and the deep-link route
  all go through `open_asset` now and the policy is deleted. iOS `NavigationHandler` /
  `NavigationPresenter` stop reading `currentWallet` to hand it back to Core for the same call.

- **A banner's link is part of its content.** `GemBannerContent.link: Option<GemBannerLink { Docs { item: DocsUrl }, External { url } }>`
  names what a tap opens (the chain's reserve documentation for account activation, the
  multi-signature and token-verification docs); the apps only turn a `Docs` item into their
  UTM-tagged docs URL. iOS `BannerViewModel.url` and the two Android banner click handlers
  (`BannerItem`, `AssetsScreen`) held that per-event switch, and Android never opened the
  suspicious-asset docs; `BannersScene` opens the link for every caller now.

- **The reserved balance row carries its documentation link.** `GemBalanceRow::Reserved { value, url }`
  holds the chain's reserve documentation (the account activation url), so neither app reads
  `ChainConfig` for it: iOS `AssetSceneViewModel.reservedBalanceUrl` and
  `Chain.accountActivationFeeUrl`, Android `Chain.getReserveBalanceUrl()` are gone, along with
  the unread `Chain.isDefiSupported` / `blockTime` (iOS) and `Chain.hasNativeAsset()` (Android).
  iOS also shows the row without a link now instead of dropping it.

- **Whether a collectible can be sent is Core's answer.** `GemCollectibleService::can_send(chain)`
  reads the session wallet and needs a signing wallet on a chain with NFT transfers. iOS
  `CollectibleViewModel.isSendEnabled` held `wallet.canSign && chain.supportsNftTransfer`;
  Android's `NftDetailsScene` only checked the chain, so a view-only wallet got a send button.
  `Chain.supportsNftTransfer` is gone from both apps.

- **Whether the wallet can add a token is Core's answer.** `GemAssetSelectionService::supports_tokens()`
  reads the session wallet and asks `rules::token_chains` for a chain with a token type. iOS
  `Wallet.hasTokenSupport` and Android `accounts.any { chain.isTokenSupported() }` are gone, with
  the iOS `Chain.isTokenSupported` / `isSwapSupported` (test-only) config readers.

- **The receive-collection chains are Core's list.** `GemNftService::receive_accounts(query)`
  returns the session wallet's accounts on NFT chains matching a search, and
  `Config::get_nft_chains()` the static NFT chain set the iOS asset-select filter needs. Android
  `ReceiveNftChainsViewModel` held the wallet read, the `isNftSupported()` filter and a
  `GemChainService` for matching; iOS `SelectAssetFlow` filtered `Chain.allCases` itself. Both
  `Chain.isNftSupported` readers are gone.

- **Which icon an asset borrows is Core's answer.** `GemAssetConfigService::icon_asset_id(asset_id)`
  maps a HyperCore perpetual (`hypercore_perpetual::BTC`) to the chain whose native symbol it
  names and returns any other id unchanged; iOS `AssetIdViewModel` and Android `Asset.getIconUrl`
  each scanned every chain for that symbol, and their `twoSubTokenIds` / `twoSubtokenIds`
  helpers are gone.

- **The swap pay/receive lists start from Core's filters on both apps.** With no opposite asset
  chosen, Android's `SearchSwapAssetsImpl` rebuilt the swappable universe itself — every wallet
  chain with `isSwapSupport()`, one `supported_assets` call per chain, unioned — while iOS used
  `GemAssetAction::SwapPay/SwapReceive` filters on the store (`enabled`, `swappable`, `has
  available balance`). Android now searches with the same `query_filters()` and only asks the
  swapper for the supported list once an opposite asset constrains it, which is what iOS does.

- **Exports with no app caller are gone.** A sweep of every `#[uniffi::export]` method against
  both apps un-exported `GemPerpetualService::{get_portfolio, apply_socket_message}` (the
  portfolio and stream services call them in Rust), `GemDeeplinkService::build_gem_url`,
  `GemSimulationFormatter::balance_changes`, `MessageSigner::plain_preview`,
  `GemWalletPreferencesService::{get_assets_timestamp, is_initial_load_completed}` and
  `GemNodeService::get_nodes`, `GemConfirmService::simulation` and
  `GemSimulationFormatter::{payload_fields, shows_header}`, and deleted `GemAddressService::short`
  (with `short_address`) and `Config::get_fee_config`; the iOS wrappers and mock methods that only
  existed for them went too. The iOS confirm mock used to recompute a `GemConfirmSimulation` from
  a `SimulationResult` through those formatter exports — a mocked Core rule (the critical-warning
  test passed because the mock re-derived the flag); tests hand it a `GemConfirmSimulation` now.
  A second pass caught `GemTransferService::{approval, metadata}` (only unread iOS wrappers called
  them; Core calls the input type directly), `GemConfigService::update_config` (its Android case
  had no reader), `GemGasPriceType::total_fee` plus the unread iOS `Date.isOutdated`,
  `TransactionState.isCompleted`, `GemFeeAsset.feeBalance` and `getCurrentWallet()` helpers and
  Android's `getWalletConnectOutputAction`. Still exported for the test kits alone:
  `GemWalletService::setup_chains` (three iOS store-adapter tests), `GemKeystore::preview_import`
  and `GemKeystore::create_store` (Android instrumented fixtures and the keystore concurrency /
  benchmark tests) and `GemDeeplinkService::build_url` (the iOS asset-details mock builds deep links with it).

- **The chain filter lists are Core's.** `GemAssetSelectionService::filter_chains()` and
  `GemTransactionsService::filter_chains()` return the session wallet's chains by rank
  (`chain::rules::wallet_chains_by_rank`). iOS `Wallet.chains` + `[Chain].sortByRank` are gone;
  Android's asset select read the accounts off the session unsorted and its activity filter
  offered `Chain.entries` — every chain, not the wallet's. `TransactionsViewModel` holds the Core
  service directly now, so the one-line `SyncTransactions` case and coordinator are deleted.

- **Android pass-through cases are gone where a Core service already sits next to them.**
  `GetBannerContent`, `ApplyBannerAction` (with the app-only `BannerAction` enum),
  `SyncMissingAssets`, `SetWalletPinned` and the unread `GetRemoteConfig` each wrapped one
  `Gem*Service` call; `BannersViewModel`, `WalletsViewModel`, `NotificationNavigation` and
  `SyncAssetInfoImpl` call the service. `Banner.toGemKey()` / `Wallet.onboardingBannerKey()`
  in `ext/Banner.kt` are the one place Android builds a banner key. `SettingsViewModelTest`
  now joins the view model scope before `resetMain()` — its `withContext(IO)` hop used to resume
  on a reset Main and fail the next test.

- **Android classifies Core's import error where the screen branches.** `ImportError` mirrored
  `GemWalletImportException` case for case (plus a `DuplicatedWallet` nothing threw), with a
  `validatedOrImportError` re-wrap between the two; the import screen switches on
  `GemWalletImportException` now, exactly as iOS's `GemWalletImportError: LocalizedError` does,
  and any other failure keeps its message. The swap screen went the same way: `SwapError` with
  its `toError(SwapperException)` mapping and `SwapErrorTest` are gone, and the error item
  classifies `SwapperException` the way iOS's `SwapperError.message(asset:)` does. iOS's
  `ChainCoreError` was the same shape in the other direction: two cases mirrored
  `GemSignerError` and three matched strings Core stopped producing; `GemConfirmError.Sign` now
  localizes the dust-threshold and insufficient-funds signer errors itself (which is what
  Android's confirm screen does) and `ConfirmTransferError` is `confirm | other`.

- **Android's amount errors match iOS on zero.** `AmountError` carried four cases nothing raised
  (`Unavailable`, `InsufficientFeeBalance`, `IncorrectAddress`, `ZeroAmount`), two of them with
  hard-coded English; a zero amount now stays silent (`AmountError.None`, the button just does
  not proceed), which is what iOS's `SilentValidationError` does. The remaining cases are the
  app-side parse errors and Core's minimum / insufficient-balance answers.

- **The asset scene's swap pair comes from the same service on both apps.** iOS asked
  `GemAssetDetailsService::swap_pair`; Android's `AssetInfoUIModelFactory` held a
  `GemSwapServiceInterface` for `pair_for_asset`. The factory takes the pair the view model reads
  from the details service now, and `GemSwapService::pair_for_asset` is no longer exported.

- **The acquire-asset flow is read from the confirm service on both apps.** iOS asked
  `GemConfirmTransferService::acquire_asset_flow` to route the button but hard-coded `chain == .tron`
  for the button's *title* in `InfoSheetModelFactory`; Android read `GemAssetConfigService::acquire_flow`
  from a `CompositionLocal` inside the error composable. Both now take the flow from the confirm
  view model (`InfoSheetType.balanceRequired / insufficientNetworkFee` carry the finished
  `InfoSheetButton`; `ConfirmErrorInfo` takes an `acquireFlow` lambda). `GemAssetConfigService::acquire_flow`
  stays exported only because the iOS confirm mock answers through it.

- **The slippage sheet's default is the pay chain's on both apps.** Android asked
  `GemSwapQuoteService::default_slippage(chain)` (Solana gets three times the default); iOS read
  `SwapConfig.default_slippage`, the chain-agnostic value, so a Solana swap's "auto" showed the
  wrong number. `SwapSlippageViewModel` takes the pay chain and asks the service. Android's unread
  `PerpetualFormatter.minimumOrderUsdAmount` and the `GemPerpetual::minimum_order_usd_amount` export
  behind it are gone (the amount rule uses the crate function).

- **The asset share link comes from the details service on both apps.** Android's asset menu
  pulled `GemDeeplinkService` out of a `CompositionLocal` to build it; the view model asks
  `GemAssetDetailsService::deeplink_url` now, as iOS does, and the UI model carries `shareUrl`.

- **Explorer links come from the screen's own service on Android too.** The chart screen's
  `AssetChartViewModel` held `GemExplorerService` for the token link (plus an `explorerName` no
  view read) and the confirm properties builder held it for the sender link; both use
  `GemChartService::token_url` and `GemConfirmTransferService::address_url` now, which is what the
  iOS chart and confirm view models call. With no app caller left, `GemExplorerService::{get_explorer_name,
  set_explorer_name, get_address_url, get_token_url}` are no longer exported — the explorer
  service is composition (the screen services hold it), not a screen service. The unused iOS
  `GemExplorerServiceMock` went with it.

- **Two device API clients, and the split is load-bearing.** `deviceRegistrationClient` has no preflight and is what `GemDeviceService`/`GemSubscriptionService` use; the general client has one and is what every other service uses. That is what stops the sync path recursing into itself. `GemDeviceApiClient.set_device_sync_preflight` must only ever be called on the general client; nothing enforces it, so this note is the only record of it.

- **Transfer model collapse.** Generate the `TransactionInputType` enum from typeshare so the primitives tuple enum, the gemstone named-field enum and the Swift/Kotlin enums become one (685 Core, 52 Android, 5 iOS references). Transaction construction is wallet-critical — do it only after both apps carry Core records through confirm. **Not started.**
- **One-sided calls that are structure, not drift** (from the September 2026 sweep of every
  generated protocol method against both apps — re-run it with the two `rg` lines in Screen
  services): Android carries transfers and position actions through routes, so only it calls
  `GemTransferService::{encode, decode}_*`; iOS builds the confirm screen's first state with
  `initial_state` while Android reads `metadata` + `preload`; iOS's `MessageSigner.hash /
  sign_with_keystore` and Android's `payload_preview` are the two halves of the WalletConnect
  sign-message path each platform drives from its SDK; `GemPreferencesService::{set_notifications_asked, should_ask_notifications,
  set_price_alerts_enabled, *_swap_slippage_bps}` are Android's push/N1 adapters and
  `UserConfig`. Anything else that shows up as one-sided is a candidate.
- **The receive screen prefetches what iOS prefetches.** Android's `ReceiveViewModel` ran
  `SyncAssetInfo` on open — a price subscription, a metadata `sync_asset`, a balance update and
  the association prefetch — where iOS only enables the asset and prefetches the other network
  assets through `GemReceiveService::sync_missing_assets`. Android now does the same
  (`syncMissingAssets` over `networkAssetIds` minus the source once the list is known); the
  asset screen's `GemAssetDetailsService::refresh` is where the metadata/balance/price sync
  belongs and already runs on both apps. `SyncAssetInfo`, its impl, test and provider are
  deleted, `GemAssetsService::sync_asset` is un-exported (only `refresh` calls it), and
  `GemAssetDetailsService::{sync_asset, sync_missing_assets}` — never called by either app —
  are gone. `SettingsViewModelTest`'s rewards tests waited on a `withContext(IO)` hop with
  `advanceUntilIdle()` alone and flaked; one test now waits for Core's answer with `first {}`
  and covers both directions.
- **The onboarding banner's "wallet empty" premise is the same fact on both apps.** Core
  decides visibility (`is_visible_event(Onboarding)` / `shows_onboarding`) from an
  `is_wallet_empty` flag the caller supplies; iOS supplied `totalFiatValue == 0` and Android
  "every enabled balance is zero", so a wallet holding only unpriced tokens welcomed the user on
  iOS. iOS now derives the flag from its observed asset rows (`assets.allSatisfy { balance.total.isZero }`),
  covered by `WalletSceneViewModelTests.onboardingBannerShowsOnlyWhileEveryBalanceIsZero`
  (funded-but-unpriced hides it, all-zero shows it). Android's banner list is reactive now
  (`GetActiveBanners` is a `Flow` over the observed banner rows, the asset info and the
  all-zero-balances fact; `BannersViewModel` no longer reloads after an action), so the
  onboarding banner comes out of the same `visibleBanners` call as every other banner and
  `BannersScene` renders that event with the `WelcomeBanner` composable. `GetShowWelcomeBanner`,
  `Wallet.onboardingBannerKey`, the home's `showWelcomeBanner`/`onHideWelcomeBanner` and
  `GemBannerService::shows_onboarding` are gone, and the DAO no longer pre-filters banner
  states in SQL — Core's `is_visible` decides.
- **The network-assets screen refreshes balances through its screen service on Android.**
  `NetworkAssetsViewModel` called `GetChainAssets.updateBalances(chain)`, which re-read the
  chain's assets from the store and ran `SyncBalances` (`GemBalanceService::update` per
  wallet id read in Kotlin) — including the native coin the screen never lists. It now does what
  iOS's `NetworkAssetsSceneViewModel.updateBalances` does: hand the ids of the listed active and
  hidden tokens to `GemWalletHomeService::update_balances`, the session-scoped screen call.
  `SyncBalances`, its impl and provider, and `GetChainAssets.updateBalances` are deleted.
- **NFT search results are Core's.** The wallet search matched collections and their assets the
  same way on both apps — iOS inside the `WalletSearchRequest` GRDB fetch, Android in
  `WalletSearchViewModel.searchNfts` with its own copy of the collection ordering — and
  `nft::rules::search_collections(data, query)` (`GemAssetSelectionService::search_collections`,
  returning `GemNftSearchItem::{Collection, Asset}`) now answers it, ordering collections with
  `sorted_collections` and assets by name. The iOS request only returns the collections; the
  view model asks Core and maps the items (`GemNftSearchItem.map()`), and the search
  view-model tests stub the mock's answer instead of the request's.
- **Which quote the swap screen shows is Core's pick.** Both apps kept "the preferred provider's
  quote, else the first" — iOS inline in `performFetch`, Android as `SwapQuotesResult.getQuote`.
  `GemSwapQuoteService::selected_quote(quotes, preferred)` (`rules::selected_quote`, tested)
  answers it; iOS calls it when quotes arrive and Android's `SwapQuoteSession` stores
  `selectedQuote` from it on `onQuoteResults` / `onProviderSelected`, the two transitions that
  can change it. The mocks state the same premise so the provider-selection view-model tests
  keep their meaning.
- **The recommended validators section is one Core answer.** Both validator-select screens
  asked `GemStakeService::recommended_validator_ids(chain)` and then ran the same
  `validators.filter { ids.contains(id) }` themselves; `recommended_validators(chain, validators)`
  returns the section (and `recommended_validator` picks from it), so neither app keeps the
  membership filter and the id accessor is gone.
- **The wallets limit is Core's rule on both apps.** iOS's `WalletsSceneViewModel` kept
  `walletsLimit = 100` and refused to open the create/import sheets past it (gem-ios #1067);
  Android had no limit at all. `rules::can_add_wallet` / `WALLETS_LIMIT` back
  `GemWalletService::{can_add_wallet, wallets_limit}`; iOS's `validate()` reads them and
  Android's `WalletsViewModel.onAddWallet` gates Create/Import the same way, showing the shared
  `errors_wallets_limit_*` strings in a dialog (`WalletsViewModelTest`).
- **Android refreshed the home twice on a wallet switch.** `StreamObserverService` called
  `SyncAssets` (`GemWalletHomeService::refresh` over the wallet's assets) on every wallet change
  in addition to `setupAssets`; once `AssetsViewModel.loadOnce` took over that refresh (above),
  the observer's copy was a second full balance + discovery pass per switch. iOS's
  `AppLifecycleService` only re-subscribes prices on a wallet change and leaves the refresh to
  the home scene, so Android does the same; `SyncAssets` and its impl are deleted.
- **Two more Android pass-through cases are gone.** `SyncFiatTransactions` read the session in
  Kotlin and called `GemFiatService::sync_transactions(wallet_id)` where iOS's
  `FiatTransactionsViewModel` calls the screen service's session-scoped
  `GemFiatQuoteService::sync_transactions()`; Android's view model does the same now, and with
  no app calling `GemFiatService` directly its four methods are plain `impl` (the constructor
  stays exported for the quote service). `SetupWallet` wrapped `GemAppStartService::setup_wallet`
  for `AppViewModel`, which now holds `GemAppStartServiceInterface` and logs the step failures
  itself (`WalletImportModule` had nothing else left and is deleted).
- **A wallet's own accounts are named after the wallet on both apps, by Core.** Android saved
  its accounts as `InternalWallet` address names when a wallet was added (`WalletStore.addWallet`
  → `saveWalletAddresses`), renamed them through `SetWalletName` → `RenameWalletAddresses`, and
  never deleted them; iOS never wrote them, so a transfer to your own other wallet showed a bare
  address on iOS and the wallet name on Android. `GemWalletService` now takes the
  `GemAddressStore` port and owns the lifecycle: `rules::wallet_address_names(wallet)` is saved
  when a wallet is stored (import, and `setup_chains` adding accounts), re-saved by `rename`,
  and deleted by `delete_wallet` (`test_wallet_accounts_are_named_after_the_wallet_until_it_is_deleted`).
  Android's `SetWalletName`, `RenameWalletAddresses`, `SaveWalletAddresses`, their impls and
  module, `Wallet.toAddressRecords`, `AddressesDao.updateName` and the address-store hooks in
  `GemstoneWalletStore` are gone; `WalletViewModel` and `SetupWalletViewModel` call
  `GemWalletServiceInterface.rename`. Both stores already keep a local name from being
  overwritten by another local type (`reservedTypes` / `isLocal`), so contacts win over wallet
  names on both apps.
- **Dead per-app helpers found by a member sweep** (declare-then-grep over `ext/`, `domains/`
  and the iOS `Extensions/`/`GemstonePrimitives` sources, excluding tests): Android `Chain.withdraw`,
  `TransactionState.isCompleted`, `Payment.request` + `decodePayment` (the instrumented payment
  test now decodes through `decodeUrl` itself) and `String.toTransactionData` — a copy of Core's
  payment `transaction_data` rule that only an instrumented codec test used to build fixtures, so
  it lives in that test now; iOS `Int.asBigInt`, `URL.toWebSocketURL`, `Locale.appstoreLanguageIdentifier`
  (each kept alive only by its own unit test), `AssetId.subTokenId`, `PerpetualConfig.{defaultLeverage,
  depositAddress, minDeposit, minWithdraw}`, `PerpetualFormatter.formatSize` and `Wallet.makeView`,
  which only the keystore test kit's address-import branch used and which now builds there.
- **Android's import screen asks `GemMnemonic` directly.** `GemValidatePhraseOperator`
  (`findInvalidWords` + `isValid` behind a `Result` with `InvalidWords`/`InvalidPhrase`
  exceptions) and `GemFindPhraseWord` were two wrappers around one Core object, and only the
  invalid-word half was read (import itself validates in `GemWalletService::import_wallet`).
  `ImportViewModel` holds `GemMnemonicInterface`; the wrappers, their exceptions and
  `GemMnemonic::is_valid` (no app caller left) are gone.
- **`display_account` was unobservable.** Core preferred a multicoin wallet's Ethereum account,
  but both apps label a multicoin row "Multicoin" and only show an address for single-chain
  wallets, where it is always the first account; iOS's `WalletViewModel` and Android's
  `WalletItem(wallet:)` already read `accounts.first`. `GetAllWalletsImpl` now does the same and
  the Core rule, test and export are deleted.
- **A rejected WalletConnect origin is Core's error, not a pre-check.** `ProposalSceneViewModel`
  asked `is_origin_rejected` before `prepare_session_proposal`, which already refuses with
  `GemWalletConnectError::InvalidOrigin`; the view model now classifies that error into the
  malicious-session notice (iOS surfaces the same error through `handleRejectSession`).
  `WalletConnectOriginVerifier` stays only for the Android-only SIWE auth flow (S9).
- **The perpetual socket asks one accessor on both apps.** Android's `ObservePerpetualWallet`
  passed the wallet into `GemPreferencesService::show_perpetuals` where iOS's lifecycle asks
  `GemPerpetualService::should_connect_perpetuals()` (the session read of the same rule); the
  Android observer now asks the perpetual service too, so the "connect" decision has one
  accessor and `show_perpetuals(wallet)` stays for the home and portfolio screens on both apps.
- **Small trims from the Android-only sweep**: `GemSwapper::get_quote` is un-exported (only `GemSwapService::get_quotes` calls it; the apps just construct the swapper), `GemPerpetual::format_size` is un-exported (Core's perpetual rules format the size; Android's `PerpetualFormatter.formatSize` wrapper had no caller and iOS never had one) and `GemPerpetualService::sync_markets` is un-exported
  (both apps call `sync_markets_if_needed`; the sweep matched Android's private `syncMarkets`
  helper); iOS's add-node scene debounces with `GemChainSettingsService::node_check_debounce_milliseconds`
  like Android instead of the component default (both 250 ms today, one owner now).
- **`GemAddressService` keeps only `format`.** `validate` and `checksum` had no iOS caller and
  Android's only callers were `Chain.isValidAddress` (unused) and `Chain.checksumAddress`, which
  `ContactAddressInput.resolvedAddress` used to re-derive the recipient address rule Core
  already applies in `GemNameService::validate_recipient` (name-record address else input,
  checksummed) — the contact view model reads `AddressInputModel.resolvedAddress`, which is
  that Core answer. The helper, its test and the stale `ChecksumAddressTest` instrumented
  test (it no longer even compiled) are deleted; Core's `address.rs` tests own the checksum rule.
- **Android reports collectibles through `GemCollectibleService::report`.** The export was
  iOS-only because Android had no report action. `NftDetailsViewModel` now holds
  `GemCollectibleServiceInterface` (the `RefreshNftAsset` case that wrapped `refresh_asset` is
  gone) and `report(reason)` sends `ReportNft { collection, asset, reason }`; the details menu
  gets a destructive Report item opening a reason sheet over `ReportReason.entries`, the same
  five reasons iOS's `ReportSelectReasonScene` lists. `refresh_asset` and `set_wallet_avatar`
  read the current wallet from the session the service already holds, so neither app passes a
  wallet id any more.
- **`GemGateway` exports only its constructor.** Neither app called a gateway method: the nine
  iOS `GatewayService` wrappers (`utxos`, `chainId`, `latestBlock`, `validators`,
  `delegationValidators`, `delegations`, `getPerpetual{AccountMode, Candlesticks, Portfolio}`)
  had no caller — the developer screen note here was stale — and Android only injects the
  gateway into Core services. The six Core uses are plain `impl` methods now;
  `get_chain_id`, `get_block_number` and `get_utxos` had no Core caller either and are deleted
  (`check_node` reads them from the provider's node status). `GemWalletPreferencesService::get_perpetual_account_mode`
  lost its dead iOS wrapper the same way; `includes_perpetual_collateral(wallet_id)` stays
  exported because Android's `PerpetualBalanceCoordinator` reads it (iOS reads the session
  variant on `GemWalletHomeService`).
- **The keystore password is created only while the keystore is empty — decided in Core.**
  `GemWalletService::import_wallet` already passed `create_if_missing = !has_stored_wallets()`
  to the password port, and `migrate_to_shared_password` deliberately passes `true`; iOS's
  `LocalKeystore.keystorePassword(createIfMissing:)` re-checked `hasStoredWallets()` itself,
  a second copy of the import rule that would have broken the migration path had iOS used it.
  The adapter now creates when asked; Core's test records the port's `create_if_missing`
  flags across two imports (`[true, false]`), `GemKeystore::has_stored_wallets` is un-exported,
  and the iOS test only keeps the adapter's own boundary (no creation unless asked).
- **Android's secret export is `GemWalletService::export_secret`.** `GetWalletSecretDataImpl`
  re-derived Core's `rules::secret_export` (private key for `WalletType::PrivateKey`, words
  otherwise) through its own `LoadPrivateDataOperator` + `PasswordStore` read, and wrapped it
  in a `WalletSecretDataValue` with `isError`. `WalletSecretDataViewModel` now holds
  `GemWalletServiceInterface` and exposes `Result<GemWalletSecret>?`; the screen renders the
  `Words` / `PrivateKey` cases. `GetWalletSecretData`, `WalletSecretDataValue`,
  `LoadPrivateDataOperator`, `GemLoadPrivateDataOperator` and their providers are deleted.
  Core's password port (`get_password(false)`) is the same keystore password the operator read,
  so the S3 note about a UI prompt was stale — neither path prompted.
- **Payment links ask Core for the token asset.** Android's `PaymentNavigation.linkRoutes`
  picked the request's asset from the *enabled* list and fell back to the chain's native coin
  when it was missing — a Solana Pay link for a token the wallet had not enabled would have
  built the transfer against SOL. It now calls `GemAssetsService::ensure_token_asset`, as iOS's
  `NavigationHandler` always did, and drops the account re-check Core's `PaymentService::load`
  already performs. `PaymentNavigationTest` covers a request for an asset outside the enabled
  list. `GemAssetsServiceInterface` is bound for it.
- **Asset screen price alerts go through the screen service on Android.** `AssetPriceAlertsViewModel`
  called `GemPriceAlertService::set_auto_alert` directly; it now calls
  `GemAssetDetailsService::set_price_alert` like iOS's `AssetSceneViewModel`, so the composition
  service is no longer reached from the asset feature.
- **Un-exported after the sweep**: `GemDeviceService::synchronize` (both apps call
  `synchronize_if_needed`; Core's app-start and price-alert flows call it), `GemAssetDiscoveryService::discover`
  (only `GemWalletHomeService::refresh` calls it; the apps only construct the service),
  `GemMnemonic::generate` (deleted — wallets are created by `GemWalletService`, neither app
  generated a phrase). Sweep gotcha: an iOS `+GemstonePrimitives.swift` wrapper calls the export
  with implicit `self` (`newest(` not `.newest(`), so verify by the bare name — `GemAppUpdateService::newest`
  looked dead and is not.
- **The home "importing" row is Core's answer on both apps.** Android's `AssetsViewModel` took
  `GetImportInProgress`, which read an `ImportWalletState` that `SyncWalletImport` /
  `ImportWalletService` wrote around the import call — an app-side copy of what
  `GemWalletHomeService::shows_initial_loading` already decides from the wallet's discovery
  step and assets timestamp. Android now does what iOS `WalletSceneViewModel.loadOnce` does:
  on every wallet change ask `showsInitialLoading()`, show the row around that wallet's
  `refresh(assetIds)`, hide it in `finally`; the import screen no longer calls anything after
  `importWallet` + `setCurrentWalletId`. The five import-state files and their DI providers are
  deleted; `AssetsViewModelTest` covers the first-load and already-loaded answers.
- **One-sided exports**, each waiting on the other platform: `wallet_connect::authentication_chain_ids` (iOS WalletConnect auth), `GemDeveloperService::{reset_transactions_timestamp, delete_wallet_preferences, clear_preferences, clear_perpetual_markets, deeplink_url}` (iOS developer actions Android's develop screen does not offer), `GemAppUpdateService::newest` (iOS's About screen shows the newest release; Android's shows the installed version and updates through Play), `GemAssetDetailsService::deeplink_gem_url` (iOS opens perpetuals through its deep-link router; Android navigates in-app), `GemCollectibleService::set_wallet_avatar` (iOS sets the avatar from the collectible screen; Android from the wallet-image screen through `GemAvatarService`), `GemWalletHomeService::apply_banner_action` and `GemAssetDetailsService::{apply_banner_action, banner_content}` (iOS's home and asset scenes forward banner actions through their screen service; Android renders banners with one host-independent `BannersScene` whose view model holds `GemBannerService`, so the forwarding pair is iOS structure).
- **`GemAssetConfigService` holders**: iOS `Chain+`, `AssetScore+`, `AssetProperties+`, `AssetBasic+`; Android `ext/Chain.kt`, `AssetDefaults.kt`. Blocked on the frozen-table decision above; Android is additionally blocked by `Migration_71_72`, a Room migration `object` that calls `chain.asset()` at database open where there is no graph to inject from.
- **`AddressFormatter` (iOS)**, 23 uses / ~60 construction sites. Attempted and reverted: threading the service reaches `WalletViewModel`, then spreads into `extension Wallet: SimpleListItemViewable`, which builds a `WalletViewModel` only to read `avatarImage` and never touches the formatter. The fix is smaller than the threading — split the display-only parts (`avatarImage`, `name`) from the address-formatting parts so only the latter needs the service. Design change, wants a decision.
- **`GemSecurityService` (iOS)** is a *defaulted* parameter on `BiometryAuthenticationService`. Making it required pushes the default into `SecurityViewModel` and `LockSceneViewModel`, which also default-construct the whole service — a lock-manager pass, not a one-liner.

### 5. Tests that cannot fail

`SettingsViewModelTest` (Android) fails intermittently across unrelated changes — seen on both
`single wallet hides rewards` and `rewards stay available while no wallets are loaded`, each passing
on an immediate rerun with an `IllegalStateException` from `TestMainDispatcher`. A flake in a
wallet-settings suite is a false signal every contributor has to re-run past; fix the dispatcher
setup rather than retrying it.

`services::wallet::tests::test_every_wallet_change_bumps_the_subscriptions_version` (Core) failed
once in a full `cargo test -p gemstone --lib` run with the subscriptions version at 9 instead of 1
after the last wallet was deleted, and passed on rerun and in isolation — the counter it asserts
on is shared preferences state, so another test's bumps can leak in under parallel execution.

`Migration_88_89Test.kt:35` seeds a multi-sig banner with `asset_id NULL` — the pre-`46889318bc` contract — and only calls `runMigrationsAndValidate`, which checks the schema and never asserts the row survived, so it cannot fail on data loss. It is an `androidTest`, so fixing it means running it on a device.

### 6. Android

- **Earn flow.** No Earn surface exists (no `StakeProviderType.Earn` reader, no `AmountParams.Earn`, no `ConfirmParams.Earn`; `GemDelegationAction.DEPOSIT` maps to nothing). Build the scene, amount provider and confirm params on `GemStakeService.sync_earn`, `GemAmountService::earn_transfer_data`, `GemAmountType::Earn` and `GemTransactionInputType::Earn`; iOS `EarnSceneViewModel` + `AmountEarnViewModel` are the reference. A feature, not a consolidation — plan it as its own batch, and not before iOS ungates it: `AssetSceneViewModel.showEarnButton` and the earn balance row are `#if DEBUG` only, so release iOS has no Earn entry either.
- **Dead `NOT NULL` columns** with no iOS counterpart: `AssetStore.saveAsset` bumps `updatedAt`, `TransactionStateStore` writes swap amounts, `NftStore` fills two legacy image columns. minSdk 28 has no `ALTER TABLE DROP COLUMN`, so removing them means recreating tables (`asset` behind its foreign keys) and instrumented migration tests do not run in CI — batch them with a migration that has another reason to touch those tables.
- `PriceStore` still stamps `prices.currency` (now only the label `AssetPriceInfo.currency` reads; the column goes with the dead-column migration). The `USD` fallbacks are gone: `AssetInfoDataAggregate` only formats fiat inside the price it has, `HeadDelegationInfo` takes `GemStakeService::currency`, and `GetCurrentCurrency::getCurrency()` is a `StateFlow` so `SettingsViewModel` and `CurrenciesViewModel` start from the real value. The perpetual screens' `Currency.USD` is deliberate (Hyperliquid is USD-denominated). (`AddAssetViewModel` now keeps the chain optional until the wallet's chains load, like iOS.)
- **Store adapters diff in SQL, not in Kotlin.** `GemAssetStore::set_buyable_assets` / `set_sellable_assets` ("exactly these ids") are two guarded `UPDATE`s in `AssetsDao` (enable the listed ids that are off, disable the unlisted ids that are on), the way iOS's `AssetStore.updateColumn` always was; the `AssetsAvailabilityService` + `calculateAvailabilityChanges` pair that computed the diff in Kotlin is gone.
- **Node screens**: `AddNodeViewModel` and `NetworksViewModel` hold `GemChainSettingsService` alone and keep no node rule of their own; the legacy `cases/<area>/` tree is gone — `NativeProvider` and the Hyperliquid socket read `GemNodeServiceInterface` directly.
- `UserConfig`: delete the `ConfigStore` fallback for `auth` once enough installs have written the secure value.
- Consistency: `*Service` classes live inside the coordinators module. (`toChain()` is gone — every chain string the app converts comes from Core, whose `Chain` and the typeshare enum are generated from one source, so `requireChain()` is the only conversion and a mismatch fails loudly instead of dropping the row.)
- Localization: 59 hardcoded `dp` values (worst: `SupportMessageBubble`, `ReceiveScreen`, `ImportScreen`, `WalletTypeTab`, `FiatScene`).

### 7. iOS

- Image URLs come from `GemImage { Asset, Validator, NftAsset, AssetList }::url()` on both apps; `Validator` answers the chain's own logo for the system ("unstaking") validator, so neither app keeps a system-validator id (iOS `DelegationValidator.systemId`, Android `SYSTEM_VALIDATOR_ID` are gone) (iOS `AssetImageFormatter`, Android's remote-URL half of `IconUrlGeneration.kt` and both apps' `assets.gemwallet.com` constants are gone). The one exception is `GemPriceWidget`, which does not link Gemstone — a widget extension cannot afford the Rust binary — so `WidgetPriceService.tokenImageURL` spells the token logo URL itself; bundled chain/provider icons stay platform paths.
- Naming: untyped `.map()` conversions where Android has `toPrimitives()`.
- `NavigationHandler`'s `.stake` deep link is an unimplemented branch and `TransactionScene`'s corner radius is an open iOS 26 styling question — both mark real gaps, keep the TODOs until closed.
- The two "delete in 2026" `FileMigrator` calls (`LocalKeystore`, `DB.swift`) move the keystore and database from documents to application support on launch. Deleting them strands anyone who has not opened the app since the move — losing their keystore — so this needs install-base data, not a code decision.

### 8. iOS view models holding more than one Core service

Each iOS scene view model should hold at most one private Core service per
[ARCHITECTURE.md § 7](ARCHITECTURE.md#7-at-most-one-core-service-on-ios-narrow-cases-on-android).
`ManageContactViewModel`, `ContactsViewModel` and `ManageContactAddressViewModel` meet the field-count
ceiling; the forwarding-only `GemContactsService` is deleted and `ContactsViewModel` holds the
owning `GemContactService`. The shared `AddressInputViewModel` takes `any GemNameServiceProtocol`,
which the parent receives as a plain `nameService` dependency beside its `service`, so no service
forwards name methods and no client declares a protocol intersection. The forwarding-only
`GemOnboardingService` is deleted: create and import wallet screens hold `GemWalletService`, which
already owns `createWallet`, `importWallet`, `nextWalletIndex`, `wallets` and `setCurrentWalletId`;
`avatarService` is injected beside it where the wallet image is shown, and
`ImportWalletTypeViewModel` takes the shared `GemChainService.shared`, which is where every stateless
Core object the apps reach for lives (`Config.shared`, `GemAssetConfigService.shared`, Android's
`assetConfig`) rather than a fresh instance per caller. `ConfirmTransferSceneViewModel` is done: it holds `service` alone, with `signer`, the keystore
password and the recent-activity store as outbound ports the app implements and
`GemConfirmTransferService` owns, and reads the currency from `service.currency()`. It hands out no
other service: `GemFeeService` is deleted (the custom-fee estimate is a `GemCustomFee.estimate`
constructor, so `NetworkFeeSceneViewModel` needs no dependency at all), and `swapQuote()` is
replaced by `swap_price_impact`, which takes `GemSwapValue` on each side and does the fiat
conversion Core-side, so `SwapDetailsViewModel` takes the computed impact and a ready
`[SwapProviderItem]` list instead of a service.

The inventory below tracks remaining multi-service view models. Re-run the audit before a bulk
migration; a non-private service means the view is reaching through the model, so fix that first by
having the parent vend the child view model.

| view model | services | non-private |
|---|---|---|
| `Gem/ViewModels/RootSceneViewModel.swift` | 4 | 0 — the launch host: app start, app update, device, session |
| `Onboarding/ViewModels/ImportWalletViewModel.swift` | 3 | 0 — a flow parent vending `ImportWalletSceneViewModel` (+ `nameService` for the shared address input) and `WalletImageViewModel` (`avatarService`); the Onboarding package cannot see `ViewModelFactory` |
| `Contacts/ViewModels/ManageContactViewModel.swift` | 2 | 0 — `nameService` for the shared address input, above |


### 9. Rules still living in app-only enums

Both apps carry Core's `GemTransferData`, `GemConfirmInput` and `GemTransactionInputType`
end to end, per [ARCHITECTURE.md](ARCHITECTURE.md) § 6 — iOS's `TransferDataType` and
`TransferData` and Android's `ConfirmParams` are all gone. The perpetual provider's recipient
(the `"Hyperliquid"` name and the deposit address) is `GemPerpetual::recipient` /
`deposit_recipient`, and `GemPerpetual::transfer_data` builds the close/modify transfer, so
neither app spells the provider name or the address (iOS `GemRecipient.hyperliquidProvider`
and Android `HyperliquidRecipient` are gone).

Which recent activity a selection records, and which types a select screen lists, are
`GemAssetAction::recent_activity_type(asset)` / `recent_activity_types` on both apps (iOS maps
`SelectAssetType` and `SelectedAssetType` to the action; Android's select and search actions carry
the asset and the action), recorded through one `add_recent(action, asset)` that reads the current
wallet from Core's session. A completed transfer is recorded by `GemConfirmTransferService` on both.

Android hand-wrote `RecentType` with different case names from the generated
`RecentActivityType` — `Send` against `Transfer`, `Buy` against `FiatBuy` — and those names
are persisted through `@SerialName`, so the platforms store different strings for the same
concept. Changing them needs a Room migration.

### Things that look like work and are not

Do not re-add these from a survey; each was checked against the code and found wrong or already done: **V5, N4, N6, V7, T4, S5, S6, V6**, backlog rows **1, 2, 4, 5, 7**, **T8**, the swap-pay recents row, and the `Transaction` typeshare model (`com.wallet.core.primitives.Transaction` is used across Android — do not drop its attribute).

Two warnings worth keeping:
- **The reconnect cap was a decision.** Android's 30 s was set deliberately (`d1cdb74`) so the price stream resumes within half a minute; iOS's 60 s was the untouched import default. Core took 30 s — do not "restore" 60 s.
- **Count the copies before adding one.** The candle interval mapping looked like two app-side copies; it was three — `candle_interval` in `gem_hypercore` has always driven the REST fetch, and the subscribed interval must equal the fetched one or the chart never updates. Check `core/crates/` before writing a rule that sounds new.
- **Core's `ValueFormatter` is not locale-aware.** Formatting amounts in Core regresses non-`en` locales, so Core hands over `{ value, decimals, symbol }` and each app formats. Applies to any future amount rule.

### How to work this list

- One change at a time: implement in Core → if a UniFFI signature, TypeShare model,
  `remote_types.yml` entry or mobile boundary changed, run `just generate` from the repo root (never
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

- Core: from `core/`, run `cargo fmt --all && cargo clippy -p gemstone --all-targets --all-features -- -D warnings && cargo test -p gemstone --lib --all-features`. CI also compiles the workspace with `--features unit_tests` and `chain_integration_tests`.
- Android: from `android/`, run `just test`, `./gradlew assembleGoogleDebug`, and `./gradlew assembleGoogleDebugAndroidTest` (DI failures surface at assembly, not compile; `androidTest` sources are **not** compiled by `testGoogleDebugUnitTest`).
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
- Core never makes a caller wait on background work. `GemConfirmService::execute` broadcasts, stores
  the transaction as pending and returns the hashes; keeping that transaction's status current runs
  through `GemTransactionStatusService`, a foreign port both apps implement by scheduling
  `GemTransactionStateService::track` off-thread, because Gemstone has no async runtime of its own.
  Awaiting the poll instead leaves the confirm screen spinning until the transaction is final.
- Every dependency a service takes is a `Gem*Service`, whether Core owns it or the app implements
  it as a foreign port. Name a port for the domain it serves, not for the mechanism.
- Chain icons come from the chain config, never an app-side list: `icon_chain` (an Ethereum layer 2 draws the Ethereum icon, SeiEvm draws Sei's) and `badge_chain` (the layer 2's own icon as the badge), both following `EVMChain::is_ethereum_layer2`.
