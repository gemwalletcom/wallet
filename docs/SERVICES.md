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

The narrow exception is a dependency-free rule object: a constructor that takes no argument over a body with no state, I/O or substitutable dependency. `GemSimulationFormatter`, `PriceAlertFormatter`, `GemAddressService`, `GemAssetConfigService`, `GemChainService`, `GemConnectionService` and `GemApplicationMetadataService` are that shape; iOS holds each once as `.shared` and Android once as a Hilt singleton in the rules module, and a test substitutes nothing because there is nothing to substitute. Keep the object cohesive and do not create one per method.

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

**Identifiers cross as their stored string, and stay that way.** `Currency` is a `remote` enum and crosses as a real enum; `Chain` is a `code` and crosses as its string with a generated mapper; `AssetId`, `WalletId`, `TransactionId` and the rest of `identifiers` cross as `String` custom types, so a call site writes `wallet.id.id` or `asset.id.identifier` and a return is rebuilt with the generated `toPrimitives()` / `toGem()`. That is the same footing for all of them: there is no identifier that crosses as the app's own struct.

Reviewed on 2026-09-15 and kept. Two things hold it in place. The string *is* the stored form — Room columns and GRDB rows persist a wallet id and an asset id as text — so a typed crossing would convert to and from that string at every store call rather than removing the conversion. And the typed crossing cannot be made symmetric: uniffi's custom-type `type_name` can name the app's own type (that is how `GemBigInt` becomes `BigInt` and `DateTimeUtc` becomes `Date`), and Swift could point it at `Primitives.WalletId` today, but Kotlin cannot — `com.wallet.core.primitives` lives in `:gemcore`, which depends on `:gemstone`, and `:gemstone` is published standalone as `com.gemwallet.gemstone:gemstone`. Pointing the generated binding at the app's types would either invert that dependency or move app-side serializers into the published Core artifact. Revisit only if the Kotlin primitives stop living behind `:gemstone`; until then the generated mappers are the one place the conversion is written, and a new Core parameter takes the typed `WalletId`/`Chain`, never a bare `String`.

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

fun togglePriceAlerts(enable: Boolean) = viewModelScope.launch(ioDispatcher) {
    service.setEnabled(enable)
    alertsEnabled.value = service.isEnabled()
}
```

Android keeps a narrow case only when the screen needs a reactive Room read or app-side aggregation that the synchronous Core service does not provide. [`GetPriceAlertsImpl`](../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/pricealerts/GetPriceAlertsImpl.kt) and [`GetAssetPriceAlertStateImpl`](../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/pricealerts/GetAssetPriceAlertStateImpl.kt) observe the `GemstonePriceAlertStore`; they do not wrap Core commands. Such cases hold the `Gemstone*Store` adapter, never the DAO and never a repository for data a Core service owns.

**A screen asks at most one Core service.** Android may additionally inject narrow observed-read cases. A view model that combines multiple `Gem*Service` protocols is doing the feature's job in the view layer. Compose the decision in Core, while keeping platform-only ports explicit.

Rules belong in `gemstone`, not in an app class wrapping several services — `GemChainSettingsService.check_node` owns the URL rule, the network-id check and the node status `AddNodeSceneViewModel` used to assemble. `GemGateway`, `GemSwapper` and `GemSimulationService` take `GemNodeService` and read the selected node from it; the gateway's own preferences store holds gateway state such as HyperCore agent data, never the node selection. Check where a service sits in that graph before giving it a new collaborator.

The same rule with each platform's noun, for the cases Core genuinely cannot answer:

- **iOS** — a feature service in `Features/<Feature>/Sources/Services/`, built in the app layer and passed into the view model's initializer. A feature package cannot read the app's `@Environment` service keys, so the service reaches it through the view model it is handed, never through an ambient registry.
- **Android** — a case in `gemcore` `application/<area>/cases/` with its implementation in `data/coordinators/<area>/`, injected into the view model by Hilt.

**An app service holds Core services, not the tables Core owns.** Reaching into a `Gem*Store`'s table from the app is a second read path the owner cannot see — the same violation as a case taking a Room DAO. The exception is a table Core has no concept of: the recent-activity list is the app's own, so `RecentActivityStore` (iOS) and `RecentAssetsService` (Android) are the platform's query layer and stay.

Android's observed store readers are cases in `data/coordinators`, the documented home for an observed read. iOS's confirm flow was the exception — `ConfirmSimulationService`, `FeeAssetProvider` and `TransferMetadataProvider` read `AssetStore`, `BalanceStore` and `PriceStore` directly. All three are deleted: the confirm service now asks the owners. 

The precedent that made this work: `GemWalletStore.get_wallets`/`get_wallet` are **synchronous** trait methods, so a service can answer a point read without `await`. Any future point read of a single row or short list should be synchronous the same way rather than pushing the caller back to the store.

`DeveloperViewModel` holds five stores to dump and reset them; it is a developer screen and stays as it is.

**Observed reads.** Core has no observation primitive, so a screen that must update as rows change observes the app's own database — iOS with `ObservableQuery` over a GRDB request, Android with a Room `Flow` returned by the case. Everything else — writes, remote sync, point reads, every decision — goes through the service.

**Tests.** iOS mocks the protocol from [`GemstoneServices/TestKit`](../ios/Packages/GemstoneServices/TestKit/); Android fakes the case interface, or mocks `Gem*Service` with MockK, using fixtures from `gemcore` `testFixtures`. Never mock a dependency-free constructible service (`GemAssetConfigService`, `GemChainService`, …) — construct the real one, or the test asserts the mock. Never fabricate I/O to reach a rule either: an offline `AlienProvider`, in-memory preference and secure stores and empty row stores, stood up so a test can touch rules that use none of them, is always the wrong answer — pass the answer in from the caller that owns the service, or mock the service and state the premise plainly. Neither app tests a rule that lives in Core — that test stays with its owning Core implementation.

### 6. Publish a multi-source refresh as one batch

A refresh that asks several sources at once — [`GemBalanceService.update`](../core/gemstone/src/services/balance/mod.rs) asks every chain of a wallet concurrently — publishes **one** store write carrying every source that answered. The contract each app's observers rely on:

- **One write per refresh, and it is atomic.** Both adapters write the batch inside a single database transaction, so an observed query never sees a wallet half updated and a portfolio total never mixes rows from two different refreshes of the same call.
- **A source that fails holds nothing back.** The sources that answered are written; the first failure in request order is returned after the write, so the caller can report it without discarding good data. `published_balances` owns that split and is tested on its own.
- **A source that fails leaves its rows as they were.** There is no "unknown" state: the previous values stay and stay visible, so a total computed while one chain is offline is a total of older values for that chain, not a total missing it.
- **The wallet is named, not implied.** Every write is keyed by the `WalletId` the refresh was asked for, so a response that lands after the user switched wallets writes the wallet it belongs to and never the one on screen.
- **Only rows whose values differ are written.** The refresh reads the stored rows, folds its updates onto them by kind — a stake answer does not clear a coin's available balance — and drops the rows that come back equal.

What the contract does **not** give you, and what any move to per-source publication has to add first: two refreshes of the same wallet in flight together each read, fold and write independently, so the slower one can publish over the newer values; nothing fences a write by the age of what it read. Splitting one batch into several also multiplies observer notifications and makes mixed-age totals the normal case rather than the exception, so the policy for both has to be decided before the split, not after.

### Done means

- Core has the flow, the rules and their tests; the app code it replaced is deleted in the same commit.
- Both apps implement the same store trait the same way, and both build and pass their suites.
- No app-side copy of a Core decision, no raw preference keys, no swallowed store failure, and no app service reading a table a `Gem*Store` owns.
- Nothing was added to reach it: both apps inject the generated Core service directly, with narrow Android cases only for observed reads or app-side aggregation — no forwarding wrapper and no repository.
- No `private let`/`private val` holding a `Gem*Service` at file scope. A service comes from the initializer or from Hilt, so a test can substitute it.
- Its store and both adapters are documented where the migration needs them, and its line in [TODO.md](TODO.md) is deleted.

## Screen services

One Core service per screen, held by the screen's view model on both apps. The session column names the screen's [state record](ARCHITECTURE.md#a-screen-whose-state-changes-is-a-session) where it has one; a dash means the screen reads without driving state, or has not been migrated. Re-run the holder sweep (`rg -l "Gem<Name>ServiceProtocol"` under `ios/Features`, `"Gem<Name>ServiceInterface"` under `android/features`) before adding a service: a screen service that only one app holds is the next consolidation, and a second Core service in a view model is the one to remove.

| Core service | Session | iOS | Android |
| --- | --- | --- | --- |
| `GemAddAssetService` | — | `AddAssetSceneViewModel` | `AddAssetViewModel` |
| `GemAmountService` | — | `AmountSceneViewModel` and its providers | `AmountViewModel`, `AmountPerpetualProvider` |
| `GemAppUpdateService` | — | `AboutUsViewModel` | `AppUpdateCoordinator` (adds the Play vs universal-APK delivery channel) |
| `GemAssetDetailsService` | — | `AssetSceneViewModel` | `AssetDetailsViewModel` |
| `GemAssetSelectionService` | — | `SelectAssetViewModel`, `WalletSearchSceneViewModel`, `AssetsResultsSceneViewModel` | `BaseAssetSelectViewModel` and its subclasses |
| `GemAvatarService` | — | `WalletImageViewModel`, vended by `CreateWalletModel` and `ImportWalletViewModel` | `WalletImageViewModel` (through `WalletAvatarService`) |
| `GemBannerService` | — | — (a collaborator inside `GemAssetDetailsService` and `GemWalletHomeService`) | — (the same two services carry `bannerContent` and `closeBanner`) |
| `GemChainService` | — | `ChainListSettingsViewModel` (chain picker) | `ContactChainSelectViewModel`, `SelectImportTypeViewModel` |
| `GemChainSettingsService` | — | `ChainSettingsSceneViewModel`, `AddNodeSceneViewModel` | `NetworksViewModel`, `AddNodeViewModel` |
| `GemChartService` | — | `ChartSceneViewModel` | `ChartViewModel` |
| `GemCollectibleService` | — | `CollectibleViewModel`, `ReportNftViewModel` | `NftDetailsViewModel` (+ `GetNftAssetDetails` observed read) |
| `GemConfirmTransferService` | `GemConfirmation` (one confirmation in flight; it loads and executes, so it is not a session) | `ConfirmTransferSceneViewModel` | `ConfirmViewModel` |
| `GemContactService` | — | `ContactsViewModel` | `ContactsViewModel` |
| `GemCurrencyService` | — | `CurrencySceneViewModel` | `CurrenciesViewModel` (+ session currency cases) |
| `GemDeviceService` | — | `RootSceneViewModel`, `AppLifecycleService` | `MainViewModel` |
| `GemDeveloperService` | — | `DeveloperViewModel` (+ the iOS stores it wipes) | `DevelopViewModel` |
| `GemFiatQuoteService` | `GemFiatSession` | `FiatSceneViewModel` | `FiatViewModel` |
| `GemManageContactService` | — | `ManageContactViewModel` (+ `nameService`) | `ManageContactViewModel` (+ `GemNameServiceInterface`) |
| `GemNftService` | — | `CollectionsViewModel`, `CollectionViewModel`, `UnverifiedCollectionsViewModel` | `NftListViewModels`, `ReceiveNftChainsViewModel` |
| `GemNotificationService` | — | `InAppNotificationsViewModel` | `InAppNotificationsViewModel` |
| `GemNotificationsService` | — | `NotificationsViewModel` | `DevicePushSettings` (the push cases `SettingsViewModel` calls) |
| `GemPerpetualDetailsService` | — | `PerpetualSceneViewModel` | `PerpetualDetailsViewModel` |
| `GemPerpetualService` | — | `PerpetualsSceneViewModel` (+ recent activity) | `PerpetualMarketViewModel` (+ recent activity) |
| `GemPortfolioService` | — | `PortfolioSceneViewModel` | `PortfolioChartViewModel` |
| `GemPriceAlertService` | — | `PriceAlertsSceneViewModel`, `SetPriceAlertViewModel` | `PriceAlertViewModel`, `PriceAlertTargetViewModel` |
| `GemReceiveService` | — | `ReceiveViewModel` | `ReceiveViewModel` |
| `GemRecentActivityService` | — | `RecentsSceneViewModel`, `SelectAssetViewModel`, `PerpetualsSceneViewModel` | `RecentsSheetViewModel`, `PerpetualMarketViewModel` |
| `GemRecipientService` | — | `RecipientSceneViewModel` (+ `nameService`) | `RecipientViewModel` (+ `GemNameServiceInterface`) |
| `GemRewardsService` | — | `RewardsViewModel`, `CreateRewardsCodeViewModel`, `RedeemRewardsCodeViewModel` | `ReferralViewModel` |
| `GemSettingsService` | — | `SettingsViewModel`, `PreferencesViewModel`, `SecurityViewModel` | `SettingsViewModel`, `PreferencesViewModel`, `SecurityViewModel` |
| `GemSignMessageService` | — | `SignMessageSceneViewModel` | `WCRequestViewModel`, `WCAuthViewModel` |
| `GemStakeService` | — | `StakeSceneViewModel`, `DelegationSceneViewModel`, `EarnSceneViewModel` | `StakeViewModel`, `DelegationViewModel`, `EarnViewModel` |
| `GemSupportService` | — | `SupportChatSceneViewModel` | `SupportChatSceneViewModel` |
| `GemSwapQuoteService` | `GemSwapSession` | `SwapSceneViewModel` | `SwapViewModel` |
| `GemTransactionDetailsService` | — | `TransactionSceneViewModel` | `GetTransactionDetailsImpl` (observed read + links) |
| `GemTransactionsService` | — | `TransactionsViewModel` | `TransactionsViewModel` |
| `GemWalletConnectService` | — | `WalletConnectorService`, `ConnectionsViewModel` | `WCRequestViewModel`, `ProposalSceneViewModel`, `WCAuthViewModel`, `ConnectionsViewModel`, `ConnectionViewModel` |
| `GemWalletHomeService` | — | `WalletSceneViewModel`, `NetworkAssetsSceneViewModel` | `AssetsViewModel`, `NetworkAssetsViewModel` |
| `GemWalletService` | — | onboarding and manage-wallet view models (`WalletsSceneViewModel` gates on `can_add_wallet`, `WalletDetailViewModel` exports the secret through `export_secret`) | `CreateWalletViewModel`, `ImportViewModel`, `WalletsViewModel` (`can_add_wallet`), `WalletViewModel` / `SetupWalletViewModel` (`rename`), `WalletSecretDataViewModel` (`export_secret`), wallet cases |
| `GemWalletSessionService` | — | `RootSceneViewModel`, `NavigationHandler` | `SessionCoordinator` (+ the services it composes) |
| `GemWidgetService` | — | `WidgetPriceService` (the price widget) | `PricesWidget` and `WidgetPriceSyncWorker` through `WidgetEntryPoint` |

### Services no screen holds

These are not screen services and must not be added to the table above. Each is either a collaborator another Core service composes, or an app-lifecycle service the platform holds outside any screen. A screen view model that starts holding one of these has taken on a second service, which [§ 7](ARCHITECTURE.md#7-at-most-one-core-service-on-ios-narrow-cases-on-android) forbids.

| Core service | Held by |
| --- | --- |
| `GemAssetsService` | composed by `app_start`, `assets`, `balance`, `confirm`, `fiat`, `receive`, `search`, `transaction_state`, `transactions`, `wallet_connect` |
| `GemExplorerService` | composed by `assets`, `chart`, `confirm`, `nft`, `node`, `stake`, `transactions`, `wallet`, `wallet_connect` |
| `GemPriceService` | composed by `assets`, `chart`, `confirm`, `currency`, `perpetual`, `portfolio`, `search`, `stream` |
| `GemStreamSubscriptionService` | composed by `assets`, `balance`, `stream`, `swap` |
| `GemSwapService` | composed by `assets` and `swap` |
| `GemSimulationService` | composed by `confirm` and `wallet_connect` |
| `GemScanService` | composed by `confirm` |
| `GemSearchService` | composed by `assets` |
| `GemFiatService` | composed by `fiat` and `stream` |
| `GemAssetDiscoveryService` | composed by `wallet_home` |
| `GemAuthService` | composed by `rewards` |
| `GemConfigService` | composed by `app_start` and `app_update` |
| `GemWalletConfigurationService` | composed by `app_start` |
| `GemDeviceKeyService` | composed by `auth` and the device signer |
| `GemSubscriptionService` | composed by `device` |
| `GemAppStartService` | iOS `OnstartService`, Android `MainViewModel` — launch orchestration, not a screen |
| `GemConnectionService` | iOS `ConnectionStatusObserver`, Android `RefreshInterval` |
| `GemPerpetualStreamService` | `HyperliquidObserverService` on both apps |
| `GemPushNotificationService` | iOS `NavigationHandler`, Android notification routing |
| `GemSecurityService` | iOS `LockSceneViewModel` and `BiometryAuthenticationService`, Android `LockTimer` |

`GemNameService` is not a row in this table: it is the [shared-component dependency](ARCHITECTURE.md#a-service-never-hands-out-another-service) that `AddressInputViewModel` and `NameRecordViewModel` take, and a parent passes it down beside its own service.

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

Open work lives in [TODO.md](TODO.md): the architecture migration to row records and screen sessions, the decisions someone has to make, and the per-platform items. This file keeps only the contract for how a service is built.

### Deliberate divergences — do not "fix" these

- The stream reconnect cap is 30 seconds because the price stream must resume within half a minute. The 60 seconds one app used to carry was an untouched import default, not a decision.
- The confirm button on the autoclose screen differs by design: one app enables it on any pending change and reveals validation after a tap, the other keeps it disabled until the change can build. Both read the same Core outcome; only the moment errors appear differs.
- `debugLog` compiles out in release on purpose. Stream connection errors and dropped events are logged only in debug builds.
- Wallet auth signs and verifies an Ethereum-scheme signature on every chain: `GemAuthService` signs with `AUTH_CHAIN`, which is `Chain::Ethereum`, so the verifier's `_ => false` is fail-closed by construction rather than a missing chain. The TON verified-collection allowlist stays hardcoded until there is an authoritative source to read it from; that is a backend gap, not a Core decision.
- A contact address can carry a memo and neither app shows it. Both apps agree, so this is a product gap rather than a duplicated decision: the title, the address format and the chain icon already come from Core, and the memo is waiting for a screen that wants it.
- Three Core deprecations are dated or gated, not forgotten: the singular transaction route in the devices API is due after 2026-11-15; the legacy locale compat answers installed clients that still send a raw platform code; and `CetusAggregator` is a wire value a stored swap can still carry, so it goes when no stored swap carries it. A new build reading old data must keep working — that is the opposite direction from the wire tolerance above, and it has no expiry a release can decide.
- Two iOS markers are open questions, not debt: the swap-again button is styled only on iOS 26, and the `Gemstone` package pins Swift 5 language mode until `GemstoneFFI` is Swift 6 clean. The lint tasks the Android `gemstone` module disables are deliberate — every Kotlin file in it is written by uniffi-bindgen, so a finding there has no author to fix it.
- Two changes are blocked on install-base data, not on a decision: Android's config-store auth fallback in `TinkGemPreferences`, and the two dated `FileMigrator` moves that put the keystore and the database under application support. They come out when the install base that needs them is gone.
- Two Core exports are app-facing for tests alone and stay: `transaction_type` is how the iOS amount view model tests name the transfer they built, and `decode_url` is what the Android instrumentation test decodes the documented QR cases with. Both are projections of a value the app already holds, not a service reach-through. About 67 exported records and enums are named by neither app; review before deleting one, since a nested field or a test may reach it.
- The NFT list, the fiat transaction row and the curated asset list read the same on both apps, and the shape is the iOS one: title on the left, the secondary number trailing, chevron last. An iOS `ListItemModel.subtitle` is the trailing slot, not a second title line — the field name is what made these look divergent on paper when the rendered rows already matched. The verified badge belongs to the NFT grid, not the list, and the grid takes it from Core's `GemNftRow.is_verified` on both apps; the details screen derives it from the collection status it already holds.
- A template that composes Core values is Core's, and the app supplies only the formatted pieces and the localized label: `GemAssetRate::text`, `GemPerpetual::margin_text` and `GemPerpetual::trigger_order_text` are the shape. The placeholder dash a missing trigger price reads as lives there too, so neither app spells it.
- The select-asset and amount screens now take every decision from Core — the flow record, the sections, the search step, the list state, the search limits, the amount entry, `allows_confirm`, the max entry, the error display and the input-type toggle. What differs is the reactive binding each platform uses to feed them: SwiftUI drives an `ObservableQuery` and refreshes an entry off a text binding, Compose debounces a `TextFieldState` and combines flows. That is transport, not a decision, and it stays per platform.
- `GemAmountError::display` decides whether an amount error reads at all — a zero amount is silent on both apps now, where iOS used to call it an invalid amount — and names the asset as `Name (SYMBOL)` unless the name is the symbol. Android used to print the bare symbol.
- A currency symbol is locale data, not a Core answer: it is what the platform prints for a currency code in a locale, and it changes with neither. `FiatSceneViewModel` keeps a `CurrencyFormatter` for that one lookup and a `ValueFormatter` for the balance, which is a `BigInt` that must not round through an `f64`. Every number the fiat session hands the screen crosses as `GemFormattedNumber`.
- Both apps take the value ladder from `GemValueStyle` — the precision for a magnitude, whether the value abbreviates, and whether it reads as dust — and render it with their own locale formatter. Core owns the decision and neither app owns a threshold; `ValueFormatter` on iOS moved to `GemstonePrimitives` for that reason.
- The iOS currency and numeric formatters live in `GemstonePrimitives` and take their precision from Core (`GemCurrencyStyle::precision`, `adaptive_precision`, `abbreviation_threshold`). `Formatters` holds only locale formatting with no rule of its own; it can depend on Gemstone — the price widget links `libgemstone.a` like the app target does — so a rule that Core owns has no reason to keep an iOS copy there. The widget's remaining copies are open work: F49 and F50 in [TODO.md](TODO.md).
- Reading a typed number into a plain one is Core's rule on both apps: Android calls it directly and iOS through `NumberInput`, which reads `GemNumberFormat`. `Formatters` is not a boundary either; the "cannot link" that once said otherwise was a missing linker flag on the widget target, fixed on 2026-09-16. The iOS `Validators` package was the last consumer that parsed an amount itself, and it is gone: Core owns every amount rule, and the text-validation seam now lives in `PrimitivesComponents/Sources/Validation/` beside the `InputValidationViewModel` that stores it.
- A view model may hold more than one Core service when it is a launch host or a flow parent vending child models, and the extra services are private. A non-private service is the real defect: the view is reaching through the model, so have the parent vend the child model instead.
- The privacy lock is iOS-only and WalletConnect one-click auth is Android-only. Both were reviewed on 2026-09-14 and kept one-sided; neither is a Core decision waiting to be shared.
- The biometric gate is per call site on Android and per secret read on iOS. Core does not mark which operations need authentication, so an Android caller that reaches the config store directly is not prompted. Reviewed on 2026-09-14 and left as is; a new Android secret read must request auth at its call site.
- The wallet home takes prices from the socket and refreshes on pull only. It does not carry the `refresh_interval` timer the asset, transactions and perpetuals screens use; reviewed on 2026-09-14 and left that way.
- Six Core answers are read by iOS alone — `notification_type`, `show_collections`, `GemConfirmation::authentication`, `application_short_name`, `image_file` and `amount_check` — and two by Android alone: `should_ask_notifications` with `set_notifications_asked`, and `sync_assets` after a token search. Reviewed on 2026-09-14 and all kept one-sided. Each is a feature one app has and the other does not; build the feature when that app wants it, never for parity, and the Core answer is already waiting.
- A bare number that crosses is not always a rendered one. Reviewed on 2026-09-14: `GemBalanceValue.amount` and `GemPriceUpdate` are written straight to the balance and price tables by both stores, `GemAssetDetailsInput.price` and `GemPriceAlertSession`'s `input` and `current_price` are inputs Core's own rules consume, `GemNftRow.count` is widened to an integer a grid poster lays out, and `invite_reward_points` is bolded as a bare integer inside the same shared sentence on both apps. None of them picks a separator, a precision or a style, so none of them belongs in the value-and-style contract. A bare-number sweep has to split rendered numbers from stored, input and layout ones before it lists anything.
- The NFT receive chain picker is Android-only, built on Core's `receive_accounts`. Reviewed on 2026-09-14 and kept one-sided; iOS receives an NFT without choosing a chain first.
- Android highlights the invalid words of a mnemonic during import and iOS does not. Reviewed on 2026-09-14 and kept one-sided. Core's `preview_import` is not the unused export it looks like: `GemWalletService` calls it on the way into `import_wallet`, and the keystore one backs an Android test fixture.
- A payment link's amount prefills the transfer amount on both apps, reached differently: iOS asks Core `GemAmountTransfer::prefilled_amount`, Android carries it on its `AmountParams.Transfer` navigation value, which Core filled in `GemRecipientNext::Amount`. Navigation values are app types, so the Android route is not a second decision.
- `SwapUiState.action` carries the Core session action for the Android swap view model tests alone; no composable reads it. It is the only handle those tests have on the session phase, so it is kept until the screen itself renders a phase — at which point the tests read that instead and the field goes.
- `with_validator`, `fee_asset`, `current_wallet_id`, `setup_chains`, `swap_quote` and `newest_release` were listed as iOS-only decisions Android remade. Re-checked on 2026-09-14: Android calls `with_validator`; it takes the fee asset and the quote from the Core records its confirm and swap services already return, reads the current wallet id as a flow off its session store because its screens observe it, and reaches the release rule through `check` rather than `newest_release`. One app calling an export the other does not need is not a second decision.
- `connection_status`, `user_rejected_error`, `chain_from_caip2` and `default_asset_basic` were listed as Android-only decisions iOS remade. Re-checked on 2026-09-14 and none of them is: iOS calls `connectionStatus` through its own component-list extension, Core emits the rejection error inside `process_request` so iOS never builds one, and iOS projects an `AssetBasic` from an `AssetFull` it already holds rather than re-deriving the defaults. `chain_from_caip2` answers a question only Android asks; iOS builds CAIP-2 ids from Core and hands the parsed chain back to Core.
- Each app reads the half of `GemAuthPromptOutcome` its platform needs — iOS `is_cancelled` to swallow a dismissal, Android `retry_delay_milliseconds` to pace the next prompt. One enum, two questions, not a duplicated decision.
- The developer screens are internal tools, not a product surface, and they diverge on purpose: iOS calls `resetTransactionsTimestamp`, `delete_wallet_preferences`, `clear_preferences`, `clear_perpetual_markets` and `deeplink_url`, Android calls `platform_store`. Reviewed on 2026-09-14: add a tool to either side when that side needs it, never for parity.
- The perpetuals banner reaches its screen differently by design: iOS opens the `gem://perpetuals` deep link, Android navigates in-app. Both turn the perpetuals preference on when the banner is tapped, and neither routes that write through Core — each app's preference observable has to see it.
- The `Delegation` mappers on both apps are not a twin violation. Both apps persist delegations, which is where a twin is correct, and the app's `Delegation` joins a price the Core record does not carry. `StakeType` and `RedelegateData` are not persisted and could cross as the remote types, but every payload they carry is one of the persisted twins, so they move only once the delegation store reads the Core record directly.
- Earn is fully built on both apps and stays hidden behind `EARN_OFFERED` in [`config/stake.rs`](../core/gemstone/src/config/stake.rs). Reviewed on 2026-09-14 and kept that way: the flag is the only switch, so the screens, view models and services behind it are live code, not dead code to delete.
- The Android notification adapter holding an application context is correct, not a leak: reading whether notifications are granted and opening the system settings both work from one, and the settings intent carries `FLAG_ACTIVITY_NEW_TASK` because of it. The one operation that needs an activity — the permission request itself — already goes through the activity collector.

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
