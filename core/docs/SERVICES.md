# Gemstone Services

Core-owned services live in [`core/gemstone/src/services/`](../gemstone/src/services/) as `<name>/{mod,model,rules,store}.rs` (only the files a service needs); every service and store returns the shared [`GemServiceError`](../gemstone/src/services/error.rs). A service owns the flow (API + rules); each app implements the `Gem*Store` trait over its database or preferences and constructs the service in DI ([`ServicesFactory.swift`](../../ios/Gem/Services/ServicesFactory.swift), Hilt modules under [`android/data/repositories/.../di`](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/di/) and [`android/data/coordinators/.../di`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/di/)). Read [How a service is built](#how-a-service-is-built) before adding or changing one.

Status: **Done** = flow in Core, both apps use it · **In progress** = being migrated · **Review** = app service not yet reviewed for Core-movable logic · **App-only** = platform concern, stays in the app · **Planned** = queued.

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

Everything that decides belongs in `rules.rs` with a test that fails if the rule flips. A rule that needs no service state is exported as a free function (`price_alerts_sorted`, `price_alert_id`) so a screen can call it without holding the service. A failure is either impossible (a rule with a built-in default), surfaced (returned as `GemServiceError`), or recorded through `services::failures::record` — never swallowed.

### 2. Pick the store the value belongs in

| What the service needs | Trait | Shape | iOS | Android |
| --- | --- | --- | --- | --- |
| rows in the database | one `Gem<Name>Store` per service ([example](../gemstone/src/services/price_alert/store.rs)) | `async`, every method returns `Result<_, GemServiceError>` | GRDB store under [`GemstoneServices/Sources/Stores/`](../../ios/Packages/GemstoneServices/Sources/Stores/) | Room DAO under [`data/repositories/.../gemstone/`](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/) |
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

Android provides the store and the service from one Hilt module ([`PriceAlertsModule`](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/di/PriceAlertsModule.kt)):

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

`gemcore/.../application/pricealerts/coordinators/SetPriceAlertsEnabled.kt` — the case:

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

A case may compose other cases ([`SyncAssetPriceAlertsImpl`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/pricealerts/SyncAssetPriceAlertsImpl.kt) calls `HasAssetPriceAlerts` and `UpdatePriceAlerts`), and it holds the Room DAO when the screen needs an observed read. It must not take a repository for data a Core service owns. The repositories left in the graph are legacy and shrink as services land; `SessionRepository` is the one still in wide use, and Core's `GemWalletSessionService` is replacing it.

**Observed reads.** Core has no observation primitive, so a screen that must update as rows change observes the app's own database — iOS with `ObservableQuery` over a GRDB request, Android with a Room `Flow` returned by the case. Everything else — writes, remote sync, point reads, every decision — goes through the service.

**Tests.** iOS mocks the protocol from [`GemstoneServices/TestKit`](../../ios/Packages/GemstoneServices/TestKit/); Android fakes the case interface, or mocks `Gem*Service` with MockK, using fixtures from `gemcore` `testFixtures`. Neither app tests a rule that lives in Core — that test belongs in `rules.rs`.

### Done means

- Core has the flow, the rules and their tests; the app code it replaced is deleted in the same commit.
- Both apps implement the same store trait the same way, and both build and pass their suites.
- No app-side copy of a Core decision, no raw preference keys, no swallowed store failure.
- Nothing was added to reach it: iOS injects the protocol, Android calls a case — no wrapper service, no repository.
- The service's row in the table above says **Done**, and its line in the plan below is removed.

## Core services

| Service | Store | iOS adapter | Android adapter | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| [`GemAssetDiscoveryService`](../gemstone/src/services/asset_discovery/mod.rs) | `GemWalletPreferencesService` | — | — | Done | Discovers wallet assets, enables them, prefetches metadata; timestamps and initial-load steps live in wallet preferences |
| [`GemAssetsService`](../gemstone/src/services/assets/mod.rs) | [`GemAssetStore`](../gemstone/src/services/assets/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/AssetStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/AssetStore.kt) | Done | Asset details, search, prefetch, missing balances, buy/sell/swap availability from config versions, wallet default balances, default asset seeding (`sync_default_assets`, stakeable flags) |
| [`GemBalanceService`](../gemstone/src/services/balance/mod.rs) | [`GemBalanceStore`](../gemstone/src/services/balance/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/BalanceStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/BalanceStore.kt) | Done | Coin/token/stake/earn balance updates via gateway |
| [`GemBannerService`](../gemstone/src/services/banner/mod.rs) | [`GemBannerStore`](../gemstone/src/services/banner/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/BannerStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/BannerStore.kt) | Done | Banner state rules |
| [`GemContactService`](../gemstone/src/services/contact/mod.rs) | [`GemContactStore`](../gemstone/src/services/contact/store.rs), [`GemFileStore`](../gemstone/src/services/file/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/ContactStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/ContactStore.kt) | Done | Contact writes, address-name sync and avatar files (`save_avatar`, `remove_avatar`, removed on delete) |
| [`GemDeviceService`](../gemstone/src/services/device/mod.rs) | [`GemDeviceStore`](../gemstone/src/services/device/store.rs), [`GemDevicePlatform`](../gemstone/src/services/device/platform.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/DeviceStore.swift), [Swift platform](../../ios/Packages/GemstoneServices/Sources/Device/DevicePlatform.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/DeviceStore.kt), [Kotlin platform](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/device/DeviceRepository.kt) | Done | Device registration and subscription sync; Core builds the `Device` from the foreign platform facts (id, push token/enabled, os/model/version/locale/store, currency) plus its own price-alert preference, and owns `synchronize`/`synchronize_if_needed` with in-flight sync coordination — `GemPriceAlertService` calls it directly (the foreign `GemDeviceSync` and both apps' sync coordinators are gone) |
| [`GemNameService`](../gemstone/src/services/name/mod.rs) | [`GemAddressStore`](../gemstone/src/services/name/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/AddressStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/AddressStore.kt) | Done | Name resolution (incl. the `can_resolve_name` rule), address names; iOS `NameServiceable`/`GemstoneNameService` adapter removed, view models use the Core protocol |
| [`GemNftService`](../gemstone/src/services/nft/mod.rs) | [`GemNftStore`](../gemstone/src/services/nft/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/NftStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/NftStore.kt) | Done | NFT sync, asset refresh, reporting |
| [`GemNodeService`](../gemstone/src/services/node/mod.rs) | [`GemNodeStore`](../gemstone/src/services/node/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/NodeStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/NodeStore.kt) | Done | Node selection and custom nodes |
| [`GemNotificationService`](../gemstone/src/services/notification/mod.rs) | [`GemNotificationStore`](../gemstone/src/services/notification/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/NotificationStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/NotificationStore.kt) | Done | In-app notifications sync; sync timestamp in wallet preferences |
| [`GemPerpetualService`](../gemstone/src/services/perpetual/mod.rs) | [`GemPerpetualStore`](../gemstone/src/services/perpetual/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PerpetualStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PerpetualStore.kt) | Done | Perpetual markets and positions |
| [`GemPreferencesService`](../gemstone/src/services/preferences/mod.rs) | [`GemPreferencesStore`](../gemstone/src/services/preferences/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PreferencesStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PreferencesStore.kt) | Done | Typed app preferences over a key-value store; also backs the gateway; `default_currency(locale)` rule used by both apps when no currency is set |
| [`GemPriceService`](../gemstone/src/services/price/mod.rs) | [`GemPriceStore`](../gemstone/src/services/price/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PriceStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PriceStore.kt) | Done | Prices, rates, currency change, market data |
| [`GemPriceAlertService`](../gemstone/src/services/price_alert/mod.rs) | [`GemPriceAlertStore`](../gemstone/src/services/price_alert/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PriceAlertStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PriceAlertStore.kt) | Done | Alerts sync; `set_enabled`/`enable_price_alert` request notification permissions (`GemNotificationPermissions`) and sync the device (`GemDeviceSync`) |
| [`GemStakeService`](../gemstone/src/services/stake/mod.rs) | [`GemStakeStore`](../gemstone/src/services/stake/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/StakeStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/StakeStore.kt) | Done | Validators, delegations, and earn positions sync |
| [`GemStreamService`](../gemstone/src/services/stream/mod.rs) | — | — | — | Done | Dispatches WebSocket events to the Core services; support typing goes through `GemSupportStore.update_typing`/`clear_typing` |
| [`GemStreamSubscriptionService`](../gemstone/src/services/stream/subscription.rs) | [`GemStreamConnection`](../gemstone/src/services/stream/connection.rs) | [Swift](../../ios/Packages/FeatureServices/StreamService/StreamConnection.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/StreamConnection.kt) | Done | Price subscription bookkeeping (`setup_assets`, `resubscribe`, `add_prices`, `reset`) over the app WebSocket, which stays app-side |
| [`GemSubscriptionService`](../gemstone/src/services/subscription/mod.rs) | [`GemWalletStore`](../gemstone/src/services/wallet/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletStore.kt) | Done | Wallet subscription changes |
| [`GemSupportService`](../gemstone/src/services/support/mod.rs) | [`GemSupportStore`](../gemstone/src/services/support/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/SupportStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/SupportStore.kt) | Done | Message sync, pending/failed delivery of text and images, image preview files cached through `GemFileStore` (`image_file`, downloaded via `AlienProvider`) |
| [`GemAmountService`](../gemstone/src/services/amount/mod.rs) | — | [Swift](../../ios/Features/Transfer/Sources/Protocols/AmountDataProvidable.swift) | [Kotlin](../../android/features/transfer_amount/viewmodels/src/main/kotlin/com/gemwallet/android/features/transfer_amount/viewmodels/providers/AmountDataProvider.kt) | Done | Amount screen rules for every flow (`GemAmountType`: transfer, deposit, withdraw, stake types, earn, perpetual): minimum value (stake/redelegate/USDC deposit-withdraw/perpetual order), reserve-for-fee, can-change-value, shows-asset-balance (`rules`), available/max value and whether the fee reserve applies (`limits` over `GemAmountBalance` incl. Tron votes, delegation balances, rewards, resources, perpetual reduce), and `validate` (zero, below minimum, insufficient balance) ([rules](../gemstone/src/services/amount/rules.rs)); iOS providers only supply `gemAmountType`, Android providers only `amountType` (+ perpetual balance) |
| [`GemRecipientService`](../gemstone/src/services/recipient/mod.rs) | — | [Swift](../../ios/Packages/PrimitivesComponents/Sources/ViewModels/AddressInputViewModel.swift) | [Kotlin](../../android/ui-models/src/main/kotlin/com/gemwallet/android/ui/models/name/AddressInputModel.kt) | Done | Recipient input rules shared by the address input and recipient screens: validity of typed address vs a resolved name record (name, chain and address must match), checksummed recipient address, when to show the address error (never for name-shaped input), `recipient(chain, input, name_record, memo, references)` → `GemRecipient` ([rules](../gemstone/src/services/recipient/rules.rs)); name lookups stay on `GemNameService` |
| [`GemTransferService`](../gemstone/src/services/transfer/mod.rs) | — | [Swift](../../ios/Packages/GemstonePrimitives/Sources/Extensions/TransferData+GemstonePrimitives.swift) | [Kotlin](../../android/gemcore/src/main/kotlin/com/gemwallet/android/domains/confirm/ConfirmInputMapper.kt) | Done | Shared transfer model `GemTransferData { input_type, recipient: GemRecipient, value, use_max_amount, minimum_value }` (`GemConfirmInput = from + transfer`) and the rules both apps duplicated: transaction type, transaction metadata, asset ids, fee asset, output type/action, approval data, spends-balance, available balance source (delegation/rewards/frozen/locked/withdrawable) and the pending-transaction factory incl. which HyperCore legs are tracked ([rules](../gemstone/src/services/transfer/rules.rs)); iOS `TransferData`/`TransferDataType` and Android `ConfirmParams` keep their typed shape but forward these rules to Core; Android `ConfirmParams` is packed/unpacked for navigation only through Core `confirm_input_encode/decode` (every variant), `SwapParams` holds the Core `SwapData`, `Generic` carries the Core output type/action, `TokenApprovalParams` and the kotlinx fallback are gone |
| [`GemTransactionStateService`](../gemstone/src/services/transaction_state/mod.rs) | [`GemTransactionStateStore`](../gemstone/src/services/transaction_state/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/TransactionStateStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/TransactionStateStore.kt) | Done | Owns the pending-transaction poll loop ([tracker](../gemstone/src/services/transaction_state/tracker.rs)): per-chain intervals, hash replacement, cancellation and post-processing (balances on in-transit/completion; stake, earn and NFT syncs on completion, [rules](../gemstone/src/services/transaction_state/rules.rs)); the apps only call `track_pending`/`track`/`stop_tracking` and observe their database |
| [`GemTransactionsService`](../gemstone/src/services/transactions/mod.rs) | [`GemTransactionStore`](../gemstone/src/services/transactions/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/TransactionStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/TransactionStore.kt) | Done | Transaction history sync; per-wallet and per-asset sync timestamps in wallet preferences |
| [`GemWalletConfigurationService`](../gemstone/src/services/wallet_configuration/mod.rs) | `GemWalletPreferencesService` | — | — | Done | Initial wallet configuration sync, multi-signature banners; completion flag in wallet preferences |
| [`GemWalletPreferencesService`](../gemstone/src/services/wallet_preferences/mod.rs) | [`GemWalletPreferencesStore`](../gemstone/src/services/wallet_preferences/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletPreferencesStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletPreferencesStore.kt) | Done | Per-wallet key-value preferences (sync timestamps, initial-load steps, wallet configuration flag, perpetual account mode; keys are a strum enum); cleared on wallet delete, completed for created wallets; Android keeps its reactive `UserConfig` copy of the perpetual mode for `GetWalletSummary` |
| [`GemWalletSessionService`](../gemstone/src/services/wallet_session/mod.rs) | [`GemWalletSessionStore`](../gemstone/src/services/wallet_session/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletSessionStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletSessionStore.kt) | Done | Current wallet session |
| [`GemWalletService`](../gemstone/src/services/wallet/mod.rs) | [`GemWalletStore`](../gemstone/src/services/wallet/store.rs), [`GemKeystorePassword`](../gemstone/src/services/wallet/password.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletStore.kt) | Done | Wallet import, creation, deletion (removes the avatar file through `GemFileStore`), chain setup, pin, rename and image url; wallet reads on `GemWalletStore` are synchronous so session lookups need no `await` |
| [`GemAvatarService`](../gemstone/src/services/avatar/mod.rs) | [`GemFileStore`](../gemstone/src/services/file/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Wallet/FileStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/FileStore.kt) | Done | Wallet avatar: stores the image file through the app file store (removing the previous one), downloads image urls through `AlienProvider`, writes the wallet image url |
| [`GemDeviceRequestSigner`](../gemstone/src/services/device/signer.rs) | — | [Swift](../../ios/Packages/GemstonePrimitives/Sources/Extensions/GemDeviceRequestSigner+GemstonePrimitives.swift) | — | Done | Device auth header for the stream WebSocket request; Android builds it in `AssetsModule` |
| [`GemSwapService`](../gemstone/src/services/swap/mod.rs) | [`GemKeystorePassword`](../gemstone/src/services/wallet/password.rs), [`GemSwapStore`](../gemstone/src/services/swap/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/SwapStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/SwapStore.kt) | Done | Quotes (request from the wallet accounts, route preload, sorted by output), `swap_quote` mapping, `get_transfer` (quote data with permit2 signing through the Core keystore, recipient account, amount) and `suggest_pair` (most swapped receive asset, then recent swap assets, then the receive picker head; pay asset when the screen opens empty) ([rules](../gemstone/src/services/swap/rules.rs)) |
| [`GemAppUpdateService`](../gemstone/src/services/app_update/mod.rs) | — | — | — | Done | Release for the store, version compare, skipped version via `GemPreferencesService` |
| [`GemFiatService`](../gemstone/src/services/fiat/mod.rs) | [`GemFiatStore`](../gemstone/src/services/fiat/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/FiatStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/FiatStore.kt) | Done | Fiat quotes, transaction sync with asset prefetch |
| [`GemAuthService`](../gemstone/src/services/auth/mod.rs) | [`GemKeystorePassword`](../gemstone/src/services/wallet/password.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/KeystorePasswordStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/KeystorePasswordStore.kt) | Done | Auth payload: nonce → auth message → Core keystore signature, device id from the device key ([rules](../gemstone/src/services/auth/rules.rs)) |
| [`GemChartService`](../gemstone/src/services/chart/mod.rs) | — | — | — | Done | Price charts |
| [`GemExplorerService`](../gemstone/src/services/explorer/mod.rs) | — | — | — | Done | Block explorer selection (preference) and transaction/address/token/NFT/validator links |
| [`GemAppStartService`](../gemstone/src/services/app_start/mod.rs) | — | — | — | Done | App start (`run`), wallet start (`setup_wallet`) and `setup_wallets` (default asset seeding + chain setup for every wallet, used by iOS `OnstartService` and Android `CheckAccountsService`); each `run` step reports failures without stopping the rest |
| [`GemConfigService`](../gemstone/src/services/config/mod.rs) | — | — | — | Done | Remote config, cached via `GemPreferencesService`; concurrent updates share one request |
| [`GemPortfolioService`](../gemstone/src/services/portfolio/mod.rs) | [`GemPortfolioStore`](../gemstone/src/services/portfolio/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PortfolioStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PortfolioStore.kt) | Done | Portfolio chart for the wallet's held assets |
| [`GemRewardsService`](../gemstone/src/services/rewards/mod.rs) | — | — | — | Done | Rewards and referrals; authenticated calls take a wallet and build the auth payload through `GemAuthService`; `redeem` enables the redeemed asset through `GemBalanceService` |
| [`GemScanService`](../gemstone/src/services/scan/mod.rs) | — | — | — | Done | Transaction scanning |
| [`GemSearchService`](../gemstone/src/services/search/mod.rs) | [`GemSearchStore`](../gemstone/src/services/search/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/SearchStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/SearchStore.kt) | Done | Wallet search: API + token lookup merge, asset/price/balance persistence, perpetuals, lists and search history keys ([rules](../gemstone/src/services/search/rules.rs)) |
| [`GemWalletConnectService`](../gemstone/src/services/wallet_connect/mod.rs) | [`GemConnectionStore`](../gemstone/src/services/wallet_connect/store.rs), [`GemWalletConnectSigner`](../gemstone/src/services/wallet_connect/signer.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/ConnectionStore.swift), [Swift signer](../../ios/Features/WalletConnector/Sources/WalletConnector/Services/WalletConnectorSigner.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/ConnectionStore.kt), [Kotlin signer](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletConnectSigner.kt) | Done | Session proposal preparation (CAIP-2 chain parsing, wallet selection, origin verification), app metadata rule, session approval data (chains, accounts, methods, events), session model from namespace accounts, connection persistence (`add_connection`, `update_sessions` sync rule, `delete_session`), request parse → connection/account lookup and session chain validation → simulate → decode → app signer (`GemWalletConnectSignRequest` with wallet, account, session, simulation and a message or transaction payload) → encode ([rules](../gemstone/src/services/wallet_connect/rules.rs)); Android `WalletConnectRequestHandler` runs `handle_request` and the signer publishes a `WalletConnectPendingRequest` the request screen approves or rejects |

## App services (iOS is the reference)

Every app service is listed so nothing is missed; "Review" rows are the remaining migration work.

| App service | Core service | Status | Notes |
| --- | --- | --- | --- |
| `ActivityService` | — | App-only | Wrapper removed; callers use `RecentActivityStore` directly (Android: `RecentAssetsService`) |
| `AddAssetService` | `GemGateway` | Done | Wrapper removed; the add-asset view model reads token data through `GatewayService` |
| `AddressNameService` | `GemNameService` | Done | Wrapper removed; callers use the Core service (`addressNames` typed helper) and `AddressStore` |
| [`AppService/OnstartService`](../../ios/Packages/FeatureServices/AppService/OnstartService.swift) | `GemAssetsService` | Done | `setupWallets` runs the v3 keystore migration then `GemAppStartService.setup_wallets`; default currency comes from `GemPreferencesService.default_currency`; what remains is device security, URL cache, backup exclusion and screenshots config |
| `AppService/OnstartAsyncService` | `GemAppStartService` | Done | Replaced by `GemAppStartService.run()` (config update, banner setup, swappable chains, availability sync); Android [`SyncService`](../../android/app/src/main/kotlin/com/gemwallet/android/services/SyncService.kt) calls the same |
| `AppService/OnstartWalletService` | `GemAppStartService` | Done | Replaced by `GemAppStartService.setup_wallet()` (default balances, wallet banners, configuration sync) on both apps; the push permission prompt is `PushNotificationEnablerService.requestPermissionsIfNotDetermined` |
| `AppService/ConfigService` | `GemConfigService` | Done | Wrapper removed; concurrent `update_config` calls are coalesced in Core |
| `AppService/ReleaseAlertService` | `GemAppUpdateService` | Done | Wrapper removed; view models call the Core service (typed helpers in `GemAppUpdateService+GemstonePrimitives.swift`) and open the store themselves; Android: [`AppUpdateCoordinator`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/update/AppUpdateCoordinator.kt) |
| [`AppService/RateService`](../../ios/Packages/FeatureServices/AppService/RateService.swift) | — | App-only | App Store review prompt |
| [`AppService/AppLifecycleService`](../../ios/Packages/FeatureServices/AppService/AppLifecycleService.swift) | — | App-only | Scene phase orchestration of observers |
| `AssetsService` / `ImportAssetsService` | `GemAssetsService` | Done | Wrappers removed; callers use the Core service (typed helpers in `GemAssetsService+GemstonePrimitives.swift`) and `AssetStore` directly; default assets are seeded by `sync_default_assets` (Android `updateNativeAssetRanks` removed) |
| `AuthService` | `GemAuthService` | Done | Wrapper removed; auth payload built in Core on both apps (Android `GetAuthPayloadImpl` removed) |
| `AvatarService` | `GemAvatarService` | Done | Wrapper removed on both apps; the app provides `GemFileStore` (iOS `LocalStore`, Android `LocalStore`), Android keeps only the emoji rendering |
| `BalanceService` | `GemBalanceService` | Done | Wrapper removed; view models call `update` on the Core service and read `BalanceStore` directly |
| `BannerService` | `GemBannerService` | Done | Wrapper removed; banner seeding (`setup`, `setup_wallet`), action handling, closing and visibility/ordering (`visible_banners`) live in Core, the app provides `GemNotificationPermissions` and the `GemBannerContext` inputs |
| [`ConnectionsService`](../../ios/Packages/FeatureServices/ConnectionsService) | — | App-only | WalletConnect activation flag and SDK setup; session persistence goes through Core (`GemstoneConnectionStore`) |
| [`ConnectionStatusService`](../../ios/Packages/FeatureServices/ConnectionStatusService) | — | App-only | Connectivity |
| `ContactService` | `GemContactService` | Done | Wrapper removed; avatar files go through Core over `GemFileStore` (typed helpers in `GemContactService+GemstonePrimitives.swift`); Android: `ContactsRepository` |
| [`DeviceService`](../../ios/Packages/GemstoneServices/Sources) | `GemDeviceService` | Done | `DeviceRequestSigner` replaced by Core `GemDeviceRequestSigner` on both apps; `GemstoneDeviceSync` adapts the app device sync for Core |
| `DiscoverAssetsService` | `GemAssetDiscoveryService` | Done | Wrapper removed; view models call the Core service directly |
| `EarnService` | `GemStakeService` | Done | Wrapper removed; `sync_earn` and `get_earn_data` on the Core service, APR read from `GemStakeStore`; Android has no earn UI yet |
| `ExplorerService` | `GemExplorerService` | Done | Moved to Core with the selected-explorer preference; both apps call the Core service directly (legacy selection migrated once) |
| `FiatService` | `GemFiatService` | Done | Wrapper removed; view models and buy/sell operations call the Core service directly; Android: `SyncFiatTransactionsImpl`, `GetBuyQuotesImpl` |
| `NFTService` | `GemNftService` | Done | Wrapper removed; view models call the Core service directly |
| `InAppNotificationService` | `GemNotificationService` | Done | Wrapper removed; view models call the Core service directly |
| `PerpetualService` (iOS), `blockchain/PerpetualService` (Android) | `GemPerpetualService` | Removed | Callers use `GemPerpetualService` directly (candlesticks, portfolio, market sync with the Core currency); iOS typed shim in `GemPerpetualService+GemstonePrimitives.swift` |
| `PortfolioService` | `GemPortfolioService` | Done | Wrapper removed; held assets come from `GemPortfolioStore` on both apps |
| `PriceAlertService` | `GemPriceAlertService` | Done | Wrapper removed; enable/disable, permissions and device sync run in Core (typed helpers in `GemPriceAlertService+GemstonePrimitives.swift`); Android `PriceAlertsEnabledCoordinator` only republishes the flag |
| [`PriceService`](../../ios/Packages/GemstoneServices/Sources) | `GemPriceService` | Done | |
| `RewardsService` | `GemRewardsService` | Done | Wrapper removed; view models call the Core service (typed helpers in `GemRewardsService+GemstonePrimitives.swift`) |
| `ServiceStatusService` | `GemServiceStatus` | Done | Wrapper removed on both apps; view models use the Core client directly |
| [`StreamService`](../../ios/Packages/FeatureServices/StreamService) | `GemStreamService` | Done | Event handling and price subscriptions in Core (`GemStreamService`, `GemStreamSubscriptionService`); only the socket connection stays app-side (`StreamObserverService`, `GemStreamConnection` adapter), see [DEVICE_WEBSOCKETS.md](DEVICE_WEBSOCKETS.md); Android: [`StreamObserverService`](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/stream/StreamObserverService.kt) |
| `SupportChatService` | `GemSupportService` | Done | Wrapper removed; the view model calls the Core service (typed helpers in `GemSupportService+GemstonePrimitives.swift`), `SupportTypingState` is an app observable fed by Core through `GemSupportStore`, image previews stay in the view model |
| `SwapService` / `SwapQuotesProvider` / `SwapQuoteDataProvider` / `Permit2DataProvider` | `GemSwapService` | Done | Wrappers removed on both apps (Android `GetSwapQuotes*`/`GetSwapSupported` too); view models call the Core service (typed helpers in `GemSwapService+GemstonePrimitives.swift`), see [SWAPPER.md](../../docs/SWAPPER.md) |
| `TransactionsService` | `GemTransactionsService` | Done | Wrapper removed; view models call `sync` on the Core service and use `TransactionStore` directly |
| [`TransactionStateTracker`](../../ios/Packages/GemstoneServices/Sources/TransactionState/TransactionStateTracker.swift) | `GemTransactionStateService` | Done | Trigger-only adapter: the poll loop, its intervals and post-processing live in Core; Android mirrors it with [`TransactionStateTracker`](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/transactions/TransactionStateTracker.kt) |
| [`WalletService`](../../ios/Packages/GemstoneServices/Sources/WalletService.swift) | `GemWalletService` | Done | Import/create/delete/setup-chains/pin/rename orchestration in Core over `GemKeystore`, `GemWalletStore`, `GemWalletSessionService` and `GemDeviceStore` (subscriptions version); the app provides the keystore password lazily via `GemKeystorePassword` and keeps v3 migration, avatar files and app preferences; Android uses it for import (`PhraseAddressImportWalletService`), deletion (`DeleteWalletImpl`), account backfill (`CheckAccountsService`), pin and rename |
| `WalletSetupService` | `GemAssetsService` | Done | Wrapper removed; default asset balances for a new wallet come from `setup_wallet` (`GemAssetStore.add_balances`) on both apps |
| [`WalletSessionService`](../../ios/Packages/GemstoneServices/Sources/Wallet/WalletSessionService.swift) | `GemWalletSessionService` | Done | Current wallet id and wallet lookups in Core over `GemWalletSessionStore` and the synchronous `GemWalletStore`; the iOS wrapper only maps types for `WalletSessionManageable`, Android `SessionRepositoryImpl` calls Core |
| [`GatewayService`](../../ios/Packages/GemstoneServices/Sources/Gateway) | `GemGateway` | Done | Typed gateway wrapper and chain service factory, moved from `Blockchain` and the former `ChainServices` package into `GemstoneServices` |
| `WalletSearchService` / `AssetSearchService` | `GemSearchService` | Done | Wrappers removed; both apps call the Core search service (Android `WalletSearchTokens` delegates to it) |
| [`Signer`](../../ios/Packages/GemstoneServices/Sources/Signer) | `GemSigner` | Done | Transaction and message signing over the Core signer, moved into `GemstoneServices` |
| [`NodeService`](../../ios/Packages/GemstoneServices/Sources) | `GemNodeService` | Done | |
| `StakeService` | `GemStakeService` | Done | Wrapper removed; view models call `sync` on the Core service, APR read from `GemStakeStore` |
| [`FeatureServices/WalletConnectorService`](../../ios/Packages/FeatureServices/WalletConnectorService) | `GemWalletConnectService` | Done | Package `ChainServices` folded into `FeatureServices`; only the reown SDK bridge remains (pairing, `AutoNamespaces`, streams, rejection reasons); proposals, session models, metadata and requests go through Core; Android `ProposalSceneViewModel` uses the same `prepare_session_proposal` and `BridgesRepository` syncs sessions through `update_sessions` (its `Session.kt` settled-session record mapping still applies the name/icon rule locally) |
| [`SystemServices`](../../ios/Packages/SystemServices) | — | App-only | Connectivity, image gallery, local store |

## App packaging goal

Store adapters and the thin app services are being consolidated so both apps look the same:

- **iOS `Packages/GemstoneServices/Sources/Stores`** — every `Gem*Store` implementation, one file per store (`FiatStore.swift`, `BalanceStore.swift`, …) over the `Store`/`Preferences` packages. Feature packages depend on `GemstoneServices` instead of owning adapters; the app injects the gateway's preference stores into `GatewayService`.
- **iOS `Packages/GemstoneServices`** — the thin app-side services that wrap the Core services, one folder per service that needs more than one file (`Device/`, `Perpetual/`, `Support/`, `TransactionState/`, `Wallet/`, …), tests under `Tests/`. App services must not read or write stores directly: persistence goes through Core (`Gem*Store` adapters); the only sync store reads left are `NodeService.node(for:)` (RPC provider hot path) and `WalletSessionService.currentWallet` (session façade). The migrated `FeatureServices` move here as they are wired to `GemstoneStore`; wrappers that only forward calls (for example `AssetDiscoveryService`, `InAppNotificationService`, `MarketService`, `ChartService`) are removed and callers use the Core service directly; a wrapper stays only when it combines Core calls with app-side stores or files, or maps bridged JSON to typed models for several call sites.
- **Android** — the same split in Android's patterns: store adapters in one package (`data/repositories/.../gemstone`, one file per store) and the coordinators/repositories stay thin wrappers over the Core services.

Status: stores — done on both apps (iOS `GemstoneServices/Sources/Stores`, Android package `data.repositories.gemstone`); GemstoneServices — in progress: all migrated feature services moved (Fiat, NFT, Transactions, SupportChat, AddressName, Contact, Price, Device, Auth, Rewards, Notification, PriceAlert, Banner, DiscoverAssets, Assets, Balance, Earn, TransactionState, Perpetual); Stake, Node, Swap and Activity moved too; `Blockchain`, `ChainService`, `Signer` and `Keystore` packages folded in (`Sources/Gateway`, `Sources/Signer`, `Sources/Keystore`); pure-forwarding wrappers removed (AssetDiscovery, InAppNotification, Market, Chart, Stake, Earn, AddAsset, ServiceStatus, WalletSetup, NFT, Fiat, Balance, Transactions, AddressName, Portfolio, Price, Assets); `DeviceRequestSigner` lives in `GemstoneServices` (the `GemAPIDevice` target is gone).

## Remaining

- **iOS `Packages/GemAPI`** — only the widget's one-endpoint client (`GemPriceWidget`); the app and feature packages no longer depend on it.

- Shared transfer model: both apps keep a typed transfer type (`TransferDataType` / `ConfirmParams`) for UI pattern matching and forward every rule to `GemTransferService`; a possible next step is generating that enum from `TransactionInputType` (typeshare) so the three copies (primitives tuple enum, gemstone named-field enum, Swift/Kotlin enums) collapse. Android has no Earn flow yet (`GemTransactionInputType::Earn` is iOS-only there).

- iOS `Primitives` hand-written mirrors of Core types (`GasPriceType`, `FeeRate`, `Fee`, `FeeSelection`, `CustomFeeEstimate`, `TransferAmount`, `BalanceRequirement`, ids such as `WalletId`/`TransactionId`/`AssetId`) stay as typed views bridged once in `GemstonePrimitives`; dead ones were removed 2026-08-27.

- Periodic review of app wrappers (`NodeService.node(for:)`, `WalletService`, `HyperliquidObserverService`) and of typeshare models unused by both apps (18 exports dropped 2026-08-27; the Rust types stay where Core uses them).

## TODO — finish Core as the single owner of logic

State on 2026-08-28: every app service forwards to a Core service or is platform glue; the rules that used to live in view models now live in `gemstone` with unit tests, both apps map Core errors to localized text, and the Android Room chain is a single 87→88 migration. A full code-smell audit of Core, iOS and Android (2026-08-28) produced the plan below. Work it top to bottom inside each section; each item lands with the app-side code it replaces deleted, a Core test that would flip if the rule flipped, and its line removed from this file in the same commit. When the list is empty, audit again and write a new one.

### Design decisions to apply everywhere

- **One preferences owner.** `GemPreferencesService` (Core, over the key-value `GemPreferencesStore`) owns every product preference; iOS `GemstonePreferencesStore`/`ObservablePreferences` and Android `UserConfig`/`ConfigStore` are storage adapters plus SwiftUI/Compose observation, nothing else. No service or view model injects both an app preferences object and `GemPreferencesServiceProtocol`. Done so far: Core `currency` (get/set/`setup_currency`), `launches_count`, `should_request_review`/`set_rate_application_shown`, `chart_period`, `perpetual_chart_period`, `is_push_notifications_enabled`, device state (`is_device_registered`, `subscriptions_version`, `pushed_device`, `pushed_subscriptions` — the `GemDeviceStore` trait is gone); `swap_slippage_bps`, `perpetual_leverage`, `perpetual_take_profit_percent`, `perpetual_stop_loss_percent` (iOS values carry over via `sharedKeys`; Android `UserConfig` exposes them as state flows over Core and its old DataStore values are not migrated); `GemBalanceService` reads the currency itself, so `set_assets_enabled`/`set_asset_pinned`, `redeem`, `discover` and `enable_transaction_assets` take none and the iOS `AssetsEnablerService` is gone; `is_perpetual_enabled`/`show_perpetuals`, `is_hide_balance_enabled`, `is_developer_enabled`, `is_accept_terms_completed`, `appearance`, `clear` (iOS `Preferences` and `ConfigurableDefaults` are deleted: `ObservablePreferences` is a pure observation layer over `GemPreferencesService`, `GemstoneWalletSessionStore` is an observable adapter over the same key-value store, and `GemstonePreferencesStore.application()` namespaces every key as `gemstone_*` except the values worth carrying over — `currency`, `appearance`, `is_perpetual_enabled`, `is_push_notifications_enabled`, `swap_slippage_bps`, the `perpetual_*` defaults and the per-chain `explorer_name_*` keys stay unprefixed, and `current_wallet_id`/`price_alerts_enabled`/`is_hide_balance_enabled`/`is_accept_terms_completed` alias the old `currentWallet`/`is_price_alerts_enabled`/`is_balance_privacy_enabled`/`is_accepted_terms` keys; the store reads legacy `Bool`/`Int` defaults as `true`/`false`/digits so values written before the move survive; `currency` is mirrored into the `group.com.gemwallet.ios` app group for the price widget; Android `UserConfig` state flows over Core and reloads them after the last wallet is deleted); `GemPerpetualService::sync_enablement`/`should_connect_perpetuals` replace the iOS `PerpetualEnablerService` and the Android enablement combine; iOS `OnstartService`, `RateService`, `AppLifecycleService`, `TransferExecutor`, `ChartSceneViewModel`, `PerpetualSceneViewModel`, `PerpetualChartModel`, `NotificationsViewModel`, `RewardsViewModel`, `PushNotificationEnablerService` read Core only. iOS is done — every reader goes through `GemPreferencesService` or the `ObservablePreferences` observation layer. Android `UserConfig` is now the same kind of adapter: chart periods, the launches count and the review prompt (`should_request_review`/`set_rate_application_shown`, replacing its own "tenth launch" check) and the 30-day notification ask window (`should_ask_notifications`/`set_notifications_asked`, the timestamp now a Core preference) all read Core. Not moving to Core: `authRequired` and `getLockInterval` — iOS keeps both in the keychain through `KeystorePassword`, so the app lock belongs in secure storage, not in the preferences store. `authRequired` now reads `TinkGemPreferences` (Core's `GemSecureStore`) and falls back to the old `ConfigStore` flag until the next write, so an existing lock survives the move and both stores stay in sync. `getLockInterval` moved with it: the value lives in `TinkGemPreferences` and the flow seeds itself from the old DataStore value on first collection, since the constructor cannot do an async read.
- **No `try? … ?? default` on Core calls.** Core preference reads are infallible (`GemPreferencesStore.get`/`GemWalletPreferencesStore.get` return `Option`), so getters return plain values; the only allowed fallback is the JSON→enum decode of a Core-provided enum in `GemPreferencesService+GemstonePrimitives.swift` (`currencyValue`, `chartPeriodValue`, …), never in a view model. Keychain-backed values are not preferences: the gateway's secure store implements the separate, fallible `GemSecureStore` trait (iOS `GemstoneSecurePreferencesStore`, Android `TinkGemPreferences`). A Core method either cannot fail (rule with a built-in default, like `includes_perpetual_collateral`) or its failure is surfaced (thrown, mapped to a localized error, or recorded through `services::failures::record`). Deliberate fail-open fallbacks (keep, do not "fix"): transaction scan outage, address-name lookup → cached names, perpetual account mode → stored mode, explorer name → first explorer, corrupt cached config → refetch, per-chain token search → no match for that chain. Same on Android: no bare `runCatching { coreCall }` without logging or a user-visible state — use `runCatchingCancellable` and handle the failure.
- **Store adapters are mirrors.** For every `Gem*Store` trait method the iOS and Android adapters must do the same reads/writes with the same filters, batching and transactions; the divergences below are bugs until proven otherwise. Settled so far: `add_missing_balances` filters to stored assets in Core, so neither adapter repeats the lookup and neither can insert a balance that violates its asset foreign key; a perpetual's asset row is written by Core through `GemAssetStore.save_assets` (`perpetual_asset_basics`), so neither perpetual adapter touches the asset table and Android stops silently ignoring an updated row; `set_swappable_assets`/`set_stakeable_assets` are additive on both platforms because `sync_swappable_chains` and `sync_default_assets` top up the same flag that `sync_availability` replaces; a transaction's derived swap asset links are rebuilt on both platforms whenever metadata arrives, inside the same database transaction as the row it belongs to; the banner row id is Core's `banner_identifier` on both platforms; balances are filtered and written in one round trip on both (`getVisibleAssetIds` in SQL, `setWalletAssetsVisibility` and `updateBalances` inside one database transaction); pending transactions read their wallets in a single query; search lists are delete-then-insert per type on both, so an empty result clears the stale rows instead of leaving them; address-name deletes are one transaction.
- **One device identity owner.** `GemDeviceKeyService` (Core, over `GemSecureStore`) owns get-or-create: it stores the private key as hex under `device_private_key`, derives the public key rather than trusting a stored copy, and caches the pair. It refuses to mint a new identity when the store errors or the stored key is unreadable — the failure propagates instead, because a silent regeneration re-registers the device and orphans its push subscriptions. The two app implementations are deleted (`GetDeviceId`/`GetDeviceIdImpl`, `SecurePreferences.getOrCreateDeviceKeyPair`/`getDeviceId`), and each app's `GemSecureStore` adapter carries the legacy read-through: iOS reads the unnamespaced keychain `devicePrivateKey` `Data` and rewrites it as hex under the namespaced key, keeping `whenUnlockedThisDeviceOnly`; Android reads `TinkSecurityStore` (`gem_device_keys`, and through it the pre-Tink DataStore layer) and rewrites it into the Core store.
- **One device owner.** `GemDeviceService` owns registration, the pushed-device diff and the in-flight lock. The Android `SyncDevice` case was a one-line forward to it and is gone: the nine call sites (app start, request preflight, FCM token, discovery, wallet import, currency change, stream connect, subscriptions observer, wallet import service) hold `GemDeviceService` and call `synchronize_if_needed`, and `DeviceRepository` keeps only what it really is — Android's `GemDevicePlatform` — reading the currency straight from `GemPreferencesService` like iOS does. Android stays on `synchronize_if_needed` everywhere while iOS forces `synchronize` after a token, currency or subscription change; Core's `needs_sync` already diffs all three, so the cheaper call is the right one and the difference is deliberate.
- **One currency owner.** `GemPreferencesService` holds the currency on both platforms. Android used to keep its own in the Room `session` row and never wrote Core's preference at all, so Core-internal reads (`GemBalanceService`, price syncs) ran on the default while the UI showed the user's choice. `SessionRepositoryImpl` now reads and writes Core, seeds it once from the old row through `setup_currency(sessionRow ?: locale)` — the same call iOS makes at start — and keeps the row updated only because the `asset_info` view joins prices on `session.currency`.
- **Coordinators depend on one Core service or repositories (plus the session), never both.** The stateless `GemExplorerService` link builder is the only exception.

### The confirm seam

Every service and store trait is migrated; what is left of the migration lives in one screen. Both apps rebuild Core's confirm records as their own aggregates, map into them, and map back out to call Core. Core already exposes the whole flow: `GemConfirmInput`, `GemConfirmLoadOptions`, `GemConfirmData`, `GemSendInput`, `GemExecuteResult`, `GemTransferData`, `GemRecipient`, `GemTransactionInputType`, `GemTransactionLoadFee`, `GemFeeRate`, `GemGasPriceType`, `GemFeeOptions`, `GemTransactionLoadMetadata`, `GemSignerInput`, `GemSignedTransaction`, plus the `GemTransferService` derivations (`transaction_type`, `asset`, `fee_asset`, `output`, `approval`, `available_value`) and the free rules `acquire_asset_flow`, `default_fee_priority`, `is_insufficient_network_fee`.

App types in the seam, with their Core home:

| App type | Platform | References | Core record |
| --- | --- | --- | --- |
| `ConfirmParams` (sealed, 15 subclasses) + `Builder` | Android | 290 | `GemConfirmInput` + `GemTransferData` |
| `AmountParams` | Android | 171 | `GemTransferData` fields |
| `TransferData` mappers (`toTransferData`, `toDto`, `toConfirmParams`) | Android | 62 | `GemTransferData`, `GemTransactionInputType` |
| `SignerParams` | Android | 17 | now carries `GemConfirmData` itself, plus the input, the typed `Fee` and the signed amount |
| `Fee` (sealed), `FeeSelection`, `DestinationAddress` | Android | — | `GemTransactionLoadFee`, `GemConfirmFeeSelection`, `GemRecipient` |
| `ConfirmTransferPreload` | iOS | 2 | `GemConfirmData` |
| `TransferAmount` | iOS | 9 | `GemTransferAmount` — the typed view of Core's calculator output |
| `Fee`, `FeeRate`, `GasPriceType`, `TransferData`, `TransferDataExtra` (Primitives) | iOS | — | `GemTransactionLoadFee`, `GemFeeRate`, `GemGasPriceType`, `GemTransferData`, `GemTransferDataExtra` |

Android converts the same payload seven times per confirm screen: `pack` → `unpack` → view model re-pack → `SignerPreloaderProxy.toConfirmInput` → `CalculateTransferAmount` → `availableValue` → `toSendInput`.

Done on iOS: `ConfirmTransferInput` carries the `GemConfirmData` Core returned plus the typed `Fee` the fee views read, `ConfirmService.confirm` sends Core's own `fee`/`metadata` instead of re-encoding them, and `TransactionData`, `TransferTransactionData`, `GemTransactionData.map()` and the unused `GatewayService.transactionLoad` are deleted. What is left on iOS is the typed-value question in blocker 1 and 5: `Fee`/`FeeRate` stay as the `BigInt` view of Core's decimal strings.

Blockers, in the order they have to be solved:

1. **Typed reads over JSON strings.** Every Core primitive crosses UniFFI as a JSON `String` typealias (`Asset`, `AssetId`, `Account`, `Wallet`, `ApplicationMetadata`, `ApprovalData`, `StakeType`, `PerpetualType`, `TransactionType`, `SimulationResult`, `ScanTransaction`). The UI reads `asset.decimals`, `asset.symbol`, `assetId.chain` everywhere, so carrying `GemConfirmInput` end to end means decoding at each read site or keeping one decoded shadow per screen. This is the largest blocker and it applies to both apps.
2. **`GemConfirmData` does not echo its input.** Android's `SignerParams.input` and iOS's `ConfirmTransferInput` exist only to keep the input beside the result. Either the apps hold the `GemConfirmInput` they sent, or Core returns it.
3. **The signed amount has no carrier.** Android's `SignerParams.finalAmount` (written once, read once) and iOS's `TransferAmount.value` both feed `GemSendInput.value`; nothing in Core carries it between `load` and `execute`.

Done on Android: `SignerParams` holds the `GemConfirmData` Core returned, so `toSendInput` sends Core's own `fee`/`metadata` and the reverse mapper `Fee.toGemSignerFee` (which dropped the priority on the way back) is deleted.
4. **Fee priority is split.** `GemTransactionLoadFee` has no priority; `GemConfirmData.selected_priority` carries it separately and both apps fuse them into their own `Fee` (iOS's `Fee` has no priority at all — the selection carries it, which is the shape Android should reach). A custom gas price still comes back as `selected_priority: normal` from `select_fee_rate`, so `Fee.priority` lies for custom fees until Android stops storing the priority in `Fee`.
5. **Fee subclass identity.** Android's `Fee` sealed subclasses (Plain/Regular/Eip1559/Solana) are chosen from the chain type and then unchecked-cast the gas price; Core models this as one `GemGasPriceType` enum. iOS keeps `BigInt` fee math against Core's decimal strings.
6. **`ConfirmParams` is narrower than Core's enum.** `TokenApprove` and `Earn` have no `ConfirmParams` subclass, so `unpack` returns `null` and the confirm screen cancels; the Earn half waits on the Android Earn surface.
7. **Presentation code switches on app subclasses.** `ConfirmProperty.Destination.map`, `BuildConfirmPropertiesImpl.getValidator` and the swap/perpetual detail builders do exhaustive `when` over `ConfirmParams`; they have to be rewritten against `GemTransactionInputType`.
8. **`GemConfirmData.scan` is never read** by either app — only the fatal verdicts surface, as errors from Core.

Defects found in this path (fixed ones are removed from this list): the WalletConnect `extra.gas_price` from a dapp is dropped by Android's `toGenericParams`/`Generic.toDto` round trip (latent — Core has no consumer yet); `GemTransferData.minimum_value` is never read by `toConfirmParams` and survives only because swaps re-derive it from the quote; `ConfirmParams` overrides `hashCode` without `equals`, so every re-emission looks like a change; `ConfirmParams.getTransactionType` and `ConfirmViewModel` construct a fresh `GemTransferService()` per call inside a `combine`.

### Rust (core/gemstone)

- One-sided exports still open: `wallet_connect::authentication_chain_ids` waits on iOS WalletConnect authentication; `nft::report` waits on an Android report screen; `wallet_preferences::is_initial_load_completed` drives the iOS wallet empty state and has no Android equivalent; `reset_transactions_timestamp` is an iOS developer action. Settled: `price::get_markets` and its `MarketsScene` are deleted — the scene had no navigation entry since March and rendered one unlocalized row, and the backend keeps serving `/markets` through `pricer::MarketsClient`; `assets::sync_availability` and `price_alert::add_price_alerts` were only reached from Core and are no longer exported, and both apps now share `node::selected_node` (the unused async `get_selected_node`/`get_node_url` are gone).
- Chain icons follow `EVMChain::is_ethereum_layer2`: the chain config carries `icon_chain` (an Ethereum layer 2 draws the Ethereum icon, SeiEvm draws Sei's) and `badge_chain` (the layer 2's own icon as the badge), so neither app keeps its own list. Celo, Mantle and XLayer draw the Ethereum icon now — they are layer 2s that both apps used to give their own icon; OpBNB and Manta keep theirs, which is what iOS already did.
- View-model rules moved to Core. The Android welcome banner asks `shows_onboarding` instead of building a context of mostly-false fields; the Android secret-data screen reads the wallet type instead of counting words (a twelve-word private key would have rendered as a phrase); `nft_sorted_collections` orders the collection lists on both apps (iOS rendered them in database order); earn's "hide zero positions" stays on iOS — Android has no earn positions list to share it with; `price_alerts_sorted` orders the manual alerts for both apps (Android rendered them in database order) and Android takes Core's fiat quote order instead of re-sorting it; the 30-day notification ask window is `should_ask_notifications` in Core (iOS still asks once through the system prompt and stores no timestamp — adopting the window there is a product call); `simulation_header`/`simulation_payload_fields` decide when the value row gives way to the header (Android showed both for a token approval, and accepted a header whose value could not be read); confirm property ordering stays in the apps — it is a layout list over app-shaped params, with no rule left once the generic case is named; `wallet_total_fiat_value`/`wallet_shows_pnl` own the wallet total (Android stopped feeding a neutral price with pre-multiplied fiat, and both apps hide P&L on the same condition); `sorted_wallets`/`wallet_display_account` order the wallet list on both apps (iOS ordered by creation index, Android grouped by type — Core's rank wins) and an unknown NFT verification status reads as unverified instead of verified; `asset_action_filters` gives both apps the send/buy/sell/swap eligibility list (swap pay now needs an available balance on both, and Android's buy list gained the enabled guard) and Android's select queries now carry the same predicates as SQL parameters, so a limited page is filled with eligible assets instead of being trimmed after the fact; `can_delete_node`/`sorted_nodes` answer node deletability and ordering, `search_matching_chains`/`search_matching_assets` replace the two client-side query matchers (they disagreed on a whitespace-only query); `price_alert_notification_type`/`price_alert_should_display` classify alerts for both apps; `nft_verified_collections`/`nft_unverified_collections` split the NFT lists for both apps; `stake_selectable_validators` filters and sorts the validator list for both apps (iOS stopped doing it in SQL, where it also excluded a legacy system id Android kept showing); the stake screen's "needs a frozen balance first" and "claimable when rewards are positive" rules are `stake_requires_frozen_balance`/`stake_can_claim_rewards` in Core, and the iOS balance model reads the freeze capability instead of listing stake chains.
- Transfer model: generate the `TransactionInputType` enum from typeshare so the primitives tuple enum, the gemstone named-field enum and the Swift/Kotlin enums collapse (685 Core, 52 Android, 5 iOS references — do it after both apps carry Core records through confirm, transaction construction is wallet-critical). **Not started.**

### iOS

- Wrappers to retire, so a view model holds the Core protocol and nothing else (see [How a service is built](#5-call-it-from-the-app--the-service-itself-on-ios-a-case-on-android)):

| Class | What it is today | Target |
| --- | --- | --- |
| [`ConnectionsService`](../../ios/Packages/FeatureServices/ConnectionsService/ConnectionsService.swift) | `setup`/`pair`/`disconnect`/`updateSessions` over `WalletConnectorServiceable`, plus an `NSLock` around setup | the scenes hold `any GemWalletConnectServiceProtocol`; pairing and setup stay with the Reown adapter. Delete the package, move its two tests onto the protocol |
| [`WalletConnectorManager`](../../ios/Features/WalletConnector/Sources/WalletConnector/Services/WalletConnectorManager.swift) | two roles in one class: it implements Core's `GemWalletConnectSigner` (an adapter, keep) and drives sheet presentation (`presentSheet`, `sessionApproval`, `isPresentingError`) | split and rename: `WalletConnectSigner` for the trait, and let the presenter own presentation. Nothing outside should hold "manager" |
| [`ChainServiceFactory`](../../ios/Packages/GemstoneServices/Sources/Gateway/ChainServiceFactory.swift) / `ChainService` | per-chain gateway construction, reached both as a factory protocol and as a namespace | one construction style. Decide whether the gateway per chain is a Core concern (`GemGatewayService`) or stays the app's gateway adapter, then delete the other path — this one is a decision, not a mechanical move |
| [`SupportTypingState`](../../ios/Packages/GemstoneServices/Sources/SupportTypingState.swift) | app-held typing state that `GemstoneSupportStore` writes and `SupportChatSceneViewModel` reads | typing belongs to `GemSupportService`: expose it on `GemSupportServiceProtocol`, keep the store trait as the persistence seam, and delete the app state and the extra `typing:` parameter on the view model and the store |

- Confirm view models still assemble `GemSendInput` from app aggregates. See **The confirm seam** below — it is the last migration item before the `TransactionInputType` collapse.
- The rule for a failing Core call on iOS, applied to the `try?` list: a Core read whose `Optional` models a real state (no wallet yet, no selected node) returns that optional and says nothing; a failure the app recovers from is **logged** and the recovery is deliberate; a failure the user must act on is thrown and mapped to localized text. Deliberate recoveries, keep them: `NodeService.node(for:)` → the chain's default node, `PerpetualService.accountMode` → the stored mode, `AmountDataProvidable.limits` → zero available (fail-closed for a send), `String+Keystore` → utf8 bytes for pre-hex passwords, `LocalKeystore.keystorePassword` → `""` is the "no password stored yet" sentinel, not a swallowed error. Now logged instead of silent: `WalletSessionService.currentWallet`/`currentWalletId`, `WalletSessionManageable.hasMulticoinWallet`, `SettingsViewModel.walletsValue` (which showed "0 wallets" for a store failure and now shows nothing). `PerpetualEnablerService` and `HyperliquidEventHandler` no longer exist, and the last silent Core calls now log: `AssetSceneViewModel.visibleBanners` and its balance refresh, `ChainSettingsSceneViewModel.onSelectExplorer` (which showed the new explorer as selected while the write had failed), `SwapSceneViewModel.performUpdate` and `CurrencySceneViewModel.updateDevice`. The 513 `try?` left outside tests are value conversions — JSON round trips, formatter parses, `wallet.account(for:)` — where `nil` is the render path, plus the developer screen; leave them.
- Store adapter divergences: `SupportStore` typing, `ConnectionsStore.updateSession`, `NodeStore.addNode`, `SearchStore.setLists` and `TransactionStore.getTransactionState` are aligned with Android (synchronous typing, skip the missing session row, upsert, single-column read). Two schema differences remain and are not adapter bugs: `banners.walletId`/`banners.assetId` are foreign keys, and SQLite does not apply `INSERT OR IGNORE` conflict resolution to foreign keys, so a banner for an asset the app has not stored throws where Android stores it; `PortfolioStore` reads through `asset_info`, which joins `accounts`, so a wallet without an account row for the chain contributes nothing.
- Dead code: the two TODOs are not stale — `NavigationHandler`'s `.stake` deep link is an unimplemented branch and `TransactionScene`'s corner radius is an open iOS 26 styling question; both mark real gaps and stay until someone closes them. The two "delete in 2026" `FileMigrator` calls (`LocalKeystore`, `DB.swift`) move the keystore and the database from the documents directory to application support on launch; deleting them strands anyone who has not opened the app since the move — losing their keystore — so this needs install-base data, not a code decision.
- Consistency: Core-error → localized text lives in `Gem/Types/Errors.swift`, `GemServiceError+GemstonePrimitives`, `ChainCoreError+Localizations` and `WalletImportError`, with no English literals left — `AlienError.Http` uses the shared `errors_network_error` (dropped from the generator's `ANDROID_ONLY_KEYS`) and `GemServiceError.Cancelled` uses the new `errors_cancelled`. `Error.isCancelled` moved from `Primitives` to `GemstonePrimitives` so it recognises Core's `Cancelled`: `StateViewType.setError` used to raise a bare "Cancelled" error state for a cancelled Core call, and `WalletConnectorManager` carried the only inline special case. JSON bridging already goes through `JsonCodable+GemstonePrimitives` — the `JSONEncoder`/`JSONSerialization` left in `WalletConnectorService` and `WalletConnectResponseType` sit on the dapp-JSON boundary (Reown's `AnyCodable` in, Core's opaque JSON string out) and belong there; one extension-file suffix (`+GemstoneSwift` vs `+GemstonePrimitives`, three `Wallet` extensions in one package); `ChainService` namespace vs `ChainServiceFactory` — one construction style; `LocalKeystore` duplicated error string; `"ws"` suffix and `region: .us` node defaults belong to Core node config; `NodeService.isValid(networkId:)` → Core validation.
- Naming: the view models' load entry point is `load()` now — GRDB's `fetch(_ db:)` is the only `fetch` left in the app layer, and the `fetch*` vocabulary went with it (`loadTrigger` and the `*LoadTrigger` types, `loadOnce`, `loadTask`, the private `loadNodes`/`loadAssets`/`loadNFTs` helpers). Still open: `GemstoneNftStore` wrapping `NFTStore`, `ConnectionStore` wrapping `ConnectionsStore`, untyped `.map()` conversions vs Android `toPrimitives()`.

### Android

- Repositories to retire, in this order — each becomes cases (interface in `gemcore` `application/<area>/coordinators/`, one implementation in `data/coordinators/<area>/` holding the Core service, or the DAO when the screen needs an observed read). Callers move to the case; the repository is deleted in the same commit:

| Repository | Callers | What it holds today | Target |
| --- | --- | --- | --- |
| `ChainInfoRepository` | 2 | ten lines: `Chain.available().sortedByDescending { it.defaultAssetRank }` | delete — the ordering is a Core rule (`GemSearchService`/chain config), the callers take it from there |
| `InAppNotificationsRepository` | 2 | one `NotificationsDao` flow | `GetInAppNotifications` case over the DAO |
| `ContactsRepository` | 1 | two DAO flows plus `GemContactService` writes | `GetContacts`/`GetContactRecipients` cases; writes already go to the service |
| `BannersRepository` | 1 | `GemBannerService` plus `BannersDao` | its case interfaces already exist (`GetActiveBanners`, `ApplyBannerAction`, `HasMultiSign`) — move the bodies into their `*Impl` and drop the repository |
| `PriceAlertRepository` (+`Impl`) | 6 | four observed reads over `PriceAlertsDao` | the matching cases already exist (`GetPriceAlerts`, `GetAssetPriceAlertState`, `HasAssetPriceAlerts`) — give them the DAO |
| `TransactionRepository` (+`Impl`) | 6 | `getTransactions(filters)`, `getTransaction(id)` | `GetTransactions`/`GetTransaction` cases over `TransactionsDao` |
| `AddressesRepository` | 5 | address-name flow plus `saveWalletAddresses` over `GemNameService` | `GetAddressName`/`SaveWalletAddresses` cases |
| `NftRepository` | 2 | two observed reads plus `GemNftService` and `nftCollectionStatus` | `GetNftList`/`GetNftAsset` cases; the `fetch*` names go with it |
| `DeviceRepository` | 2 | push token and push-enabled over DataStore, device info for `GemDeviceService` | push token and flag are preferences — move the values into Core (`GemPreferencesService`) and keep a `GetPushEnabled`/`SetPushToken` case |
| `PerpetualRepository` (+`Impl`) | 18 | six observed reads over the perpetual DAOs | one case per read (`GetPerpetuals`, `GetPerpetual`, `GetPositions`, `GetPosition`, `GetPerpetualBalance`) |
| `StakeRepository` | 12 | DAO reads plus `GemStakeService` and the validator free functions | stake cases; the recommended/selectable validator calls go straight to Core |
| `TokensRepository` | 4 | three search strategies over `GemSearchService`/`GemAssetsService` | the query strategy becomes `SearchTokensCase` holding `GemSearchService`; the Android-only "retry as a text query" fallback is decided against iOS first |
| `SupportChatRepository` | 3 | forwards eight calls to `GemSupportService`, plus `SupportTypingState` | delete — the view model holds `GemSupportService`; typing moves into Core with the iOS item above |
| `BridgesRepository` | 9 | 272 lines of Reown SDK calls with Core mixed in | it is the WalletConnect adapter, not a repository: rename, keep only the SDK seam, and expose approve/reject/pair/respond as cases |
| `WalletsRepository` (+`Impl`) | 31 | wallet CRUD over `WalletsDao` while Core has `GemWalletService`/`GemWalletStore` | cases over `GemWalletService`; the DAO stays behind the `GemWalletStore` adapter |
| `AssetsRepository` | 41 | 262 lines: observed asset lists, plus `GemAssetsService`/`GemBalanceService` orchestration | split — observed reads become `GetAssetsInfo*` cases over `AssetsDao`, the orchestration is already Core's |
| `SessionRepository` (+`Impl`) | 107 | current wallet and currency, `session()` StateFlow | last, in slices: `GetCurrentWallet`/`SetCurrentWallet`/`ObserveSession` over `GemWalletSessionService` and currency over `GemPreferencesService` |

  The observer services in the same module (`DeviceObserverService`, `StreamObserverService`, `HyperliquidObserverService`, `HyperliquidSubscriptionService`) are long-lived stream adapters, not cases — they stay, but move out of `data/repositories` when the repositories are gone.

- Apply the preferences decision above: `UserConfig`/`ConfigStore` become the adapter, and the keys still in DataStore/`ConfigStore` (`authRequired`, `getLaunchNumber`, `chartPeriod`, `perpetualChartPeriod`, `getLockInterval`, `isAskNotifications` + its 30-day permission window) move to Core.
- Error handling: the sync coordinators, `UpdateBalances`, `TokensRepository`, `SupportChatRepository`, `StreamObserverService` and `GetWalletSecretDataImpl` now use `runCatchingCancellable` and log what failed; `Chain.isSwapSupport()` no longer turns a config error into "swap unsupported"; `AddAssetViewModel` keeps the screen open when the add fails; the `!!` on Core data in `GetNftAssetDetailsImpl`, `PhraseAddressImportWalletService`, `NftDetailsScene` and `StakeScreen` are gone. `PriceAlertViewModel`, `WalletImageViewModel`, `SyncService` and `SwapViewModel` now use `runCatchingCancellable` and log. The `runCatching` calls left are around non-suspend work (URI parsing, `valueOf`, `startActivity`, focus requests, JSON decode helpers), where cancellation cannot be swallowed — leave them.
- Compose/platform deprecations still warned by the Kotlin compiler: `rememberModalBottomSheetState(skipPartiallyExpanded:confirmValueChange:)` (3 sites, needs the `rememberBottomSheetState(Hidden)` migration), `TabRowDefaults.containerColor` in `PortfolioChartScene`, `FirebaseMessaging.token` `Task` in `RequestrPushToken`. The clipboard ones are gone: the 14 screens take the platform `ClipboardManager` from the context (`Context.clipboardManager()`), so neither `Clipboard.nativeClipboard` nor the `NativeClipboard` typealias is used.
- Localization: done so far — the copy toast (`common_copied_to_clipboard`), the keystore access paragraph in `WalletSecretDataNavScreen` (`errors_keystore_access`, a string iOS should adopt in `LocalKeystore` once Core errors have one localization owner) and the confirm screen's missing-account state (`errors_wallet_account_missing`, now a `ConfirmState.FatalError(messageRes)` instead of an English literal). The WalletConnect view models now pass the failure's own message (empty when it has none) and `RequestScene`/`ProposalScene`/`AuthRequestScene` resolve the empty case to `errors_unknown_try_again`, so "Request failed", "Sign failed", "Connection failed" and "Authentication failed" are gone (`BridgeRequestError.MaliciousSession`'s English constructor text is never rendered — the scenes show `errors_connections_malicious_origin`). `CreateWalletViewModel`, `ImportViewModel` and `AmountViewModel` follow the same shape now: the state carries the failure's own message (null or empty when it has none) and the screen resolves that to `errors_unknown_try_again`. `DevelopScene`/`PaymentsScene` are developer-only screens and stay English; the `AssetSelectScene` literal is inside a `@Preview`. Still open: 59 hardcoded `dp` values (worst: `SupportMessageBubble`, `ReceiveScreen`, `ImportScreen`, `WalletTypeTab`, `FiatScene`).
- Store adapter divergences (mirror iOS or fix iOS, per method): `AssetStore.saveAsset` (drops the market and bumps `updatedAt`), `ConnectionStore` (three non-trait methods on the adapter), `PriceStore` (currency-tagged rows iOS never writes, `mapNotNull` on unparsable ids), `TransactionStateStore` (writes swap amounts nothing reads), `NftStore` (two legacy image columns fed from Core's single preview url), `WalletSessionStore` (`setCurrentWalletId` still creates the session row, now seeding its currency from Core).
- Layering: coordinators reaching DAOs directly (`GetShowWelcomeBannerImpl` via `BannersDao`, `ObserveFiatTransactionsImpl`, `GetAssetPriceUsdImpl`), coordinators mixing Core with non-session repositories (`SearchSwapAssetsImpl`, `ConfirmTransactionImpl` + `RecentAssetsService`, `GetWalletSummaryImpl` six collaborators), repositories doing orchestration (`AssetsRepository.add` also calls the stream service, `TokensRepository`'s three-strategy search — the asset-id strategy now calls Core's `sync_assets(asset_ids, currency)` (fetch, store, price in one place, the same steps `GemSearchService::save_assets` already ran) instead of writing the DAO itself, and the dead `SyncAssetPrices` case is deleted; what is left is the query strategy, whose seven callers should hold `GemSearchService` and pick `searchAssets` or `search` per site the way iOS does, and the `ensure_token_asset` strategy with its Android-only "retry as a text query" fallback, `BannersRepository` merge rule); **app-side wrappers around a Core service that are not coordinator implementations** — an extra indirection inside the data layer with nothing of its own: `UpdateBalances` (deleted, `AssetsRepository` calls `GemBalanceService.update`), `PricesRepository` (deleted whole — nothing injected it after the price migration), `InAppNotificationsRepository.sync`/`markNotificationsRead` (deleted — the view model holds `GemNotificationService` and the repository keeps only the DAO read), `PricesRepository.updatePrices` (keeps its place — it resolves the session currency and maps `AssetBasic` to `AssetPrice` before calling Core); the thin coordinators are not the problem — `GetSelectAssetsInfoImpl` and `GetActiveAssetsInfoImpl` are what keep view models off the repositories, and deleting them would create the very dependency the item above objects to; `GetAssetPriceUsdImpl` is the real violation because it reads `PricesDao` instead of a repository; `data/services/remote-gem` and the `walletconnect/{reown,noop}` modules as candidates to fold.
- Consistency: `fromJson` (swallows) vs `decodeJson` (throws) vs `GemAsset.toPrimitives()` `runCatching` — one bridging convention; `toChain()` vs `requireChain()` used interchangeably; `NodeStore` String/Chain twin methods; five `getAssetsInfo*` overloads; `tx` abbreviations (`txProperties`, `DbTxSwapMetadata`, `txId`); `fetch*`/`resolve*` names (`NftRepository`, `RequestSwapQuotesImpl`, `AssetsResultsViewModel`, `SyncFiatTransactionsImpl`); `*Service` classes inside the repositories and coordinators modules; non-adapter files in the `gemstone` package (`NotificationPermissions`, `StreamConnection`, `FileStore`, `WalletConnectSigner`); adapter names differing from iOS for the same trait (`ConnectionsDao`/`ConnectionsStore`, merged price+rate store, `TransactionStateStore` collaborators).
- Earn flow: Android has no Earn surface yet (no `StakeProviderType.Earn` reader, no `AmountParams.Earn`, no `ConfirmParams.Earn`, `GemDelegationAction.DEPOSIT` maps to nothing); build the Earn scene, the amount provider and the confirm params on top of `GemStakeService.sync_earn`/`get_earn_data`, `GemAmountType::Earn` and `GemTransactionInputType::Earn` (iOS `EarnSceneViewModel` + `AmountEarnViewModel` are the reference). This is a feature, not a consolidation, so plan it as its own batch.

### How to work this list (for whoever picks it up)

- One change at a time: implement in Core → regenerate bindings only if an exported signature changed (`just generate-stone` and `just generate-android-stone` from the repo root, never while an iOS build is running) → wire both apps → delete the app code it replaces → verify → commit and push to `main` directly (no PR, no `Co-Authored-By`/session trailers) → arm a CI watch for the pushed commit and fix red CI before anything else.
- Every commit that finishes an item removes its line from this file in the same commit; do not add checkboxes or "done" notes.
- Rules go in `rules.rs` with a unit test that would fail if the rule flipped; services are `services/<name>/{mod,model,rules,store,error}.rs` with only the files they need. No code comments. No `utils`/`helper`/`fetch`/`resolve` names, no `tx`.
- Compare the pre-change app logic (`git show <sha>^:<path>`) with the Core rule before deleting it; when iOS and Android disagree, pick the iOS rule unless it is clearly wrong, and say so in the commit message.
- Stores only write rows whose values differ; coordinators depend on one Core service or on repositories (plus the session), not both.

### Verification

- Core: `cargo fmt --all && cargo clippy -p gemstone --all-targets --all-features -- -D warnings && cargo test -p gemstone --lib --all-features`; regenerate bindings only when an exported signature changes. CI also compiles the workspace with `--features unit_tests` and `chain_integration_tests`.
- Android: root `./gradlew compileGoogleDebugKotlin compileGoogleDebugUnitTestKotlin` plus the touched modules' `testDebugUnitTest` (the root task does not compile every module's unit tests).
- iOS: `just build && just test` from `ios/`.
- After each batch: compare the pre-change app logic (`git show <sha>^:<path>`) with the Core rule and add the Core test that would flip if the rule flipped.

## Conventions

- Identifiers cross the FFI typed: `WalletId`, `AssetId`, `Chain`, `NFTAssetId`, `Currency`; store row ids stay `String`.
- Store methods: `get_*` reads, `is_*` boolean reads, `set_*` preferences and stored flags or sets (`set_buyable_assets`, `set_assets_enabled`, `search::set_assets`), `save_*` upserts, `add_*` inserts that must not overwrite existing rows, `update_<items>(…, items, delete_ids)` for reconcile writes, `delete_*` removals, and `clear*` for wiping a whole scope (`preferences::clear`, `support::clear_typing`).
- Rules live in `rules.rs` with unit tests; `primitives` types stay policy-free.
