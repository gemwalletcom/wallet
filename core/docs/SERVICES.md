# Gemstone Services

Core-owned services live in [`core/gemstone/src/services/`](../gemstone/src/services/) as `<name>/{mod,model,rules,store}.rs` (only the files a service needs); every service and store returns the shared [`GemServiceError`](../gemstone/src/services/error.rs). A service owns the flow (API + rules); each app implements the `Gem*Store` trait over its database or preferences and constructs the service in DI ([`ServicesFactory.swift`](../../ios/Gem/Services/ServicesFactory.swift), Hilt modules under [`android/data/repositories/.../di`](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/di/) and [`android/data/coordinators/.../di`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/di/)). Read [How a service is built](#how-a-service-is-built) before adding or changing one.

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

**There is no `*RulesService`.** A domain has one service, and a rule that domain's service owns is a method on it. When a rule has to be reachable from code that can never hold an injected service — a SwiftUI `Identifiable`, a Compose scene, a value-type extension, an enum property, a `Gem*Store` adapter (Core constructs the service *from* it) — it belongs on a service that needs no store and no API and so can be constructed anywhere: `GemChainService`, `GemAddressService`, `GemAssetConfigService` ([`assets/config.rs`](../gemstone/src/services/assets/config.rs)), `GemStakeConfigService` ([`stake/config.rs`](../gemstone/src/services/stake/config.rs)), `GemApplicationMetadataService`, `PriceAlertFormatter`, `BalanceCalculator`, `GemPerpetual`, `GemKeystore`. The I/O service never carries a second copy. This is also what keeps rule tests honest: a rule hung off a service that needs a gateway can only be reached through a mock, and the mock's answer is what the test then asserts (four iOS stake tests were doing exactly that).

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

A case may compose other cases ([`SyncAssetPriceAlertsImpl`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/pricealerts/SyncAssetPriceAlertsImpl.kt) calls `HasAssetPriceAlerts` and `UpdatePriceAlerts`), and it holds the Room DAO when the screen needs an observed read. It must not take a repository for data a Core service owns. The repositories left in the graph are legacy and shrink as services land; `SessionRepository` is the one still in wide use, and Core's `GemWalletSessionService` is replacing it.

**Observed reads.** Core has no observation primitive, so a screen that must update as rows change observes the app's own database — iOS with `ObservableQuery` over a GRDB request, Android with a Room `Flow` returned by the case. Everything else — writes, remote sync, point reads, every decision — goes through the service.

**Tests.** iOS mocks the protocol from [`GemstoneServices/TestKit`](../../ios/Packages/GemstoneServices/TestKit/); Android fakes the case interface, or mocks `Gem*Service` with MockK, using fixtures from `gemcore` `testFixtures`. Never mock a constructible service (`GemStakeConfigService`, `GemAssetConfigService`, `GemChainService`, …) — construct the real one, or the test asserts the mock. Neither app tests a rule that lives in Core — that test belongs in `rules.rs`.

### Done means

- Core has the flow, the rules and their tests; the app code it replaced is deleted in the same commit.
- Both apps implement the same store trait the same way, and both build and pass their suites.
- No app-side copy of a Core decision, no raw preference keys, no swallowed store failure.
- Nothing was added to reach it: iOS injects the protocol, Android calls a case — no wrapper service, no repository.
- The service is listed in [Core services](#core-services) with its store and both adapters, and its line in the plan below is removed.

## Core services

| Service | Store | iOS adapter | Android adapter | Notes |
| --- | --- | --- | --- | --- |
| [`GemAssetDiscoveryService`](../gemstone/src/services/asset_discovery/mod.rs) | `GemWalletPreferencesService` | — | — | Discovers wallet assets, enables them, prefetches metadata; timestamps and initial-load steps live in wallet preferences |
| [`GemAssetsService`](../gemstone/src/services/assets/mod.rs) | [`GemAssetStore`](../gemstone/src/services/assets/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/AssetStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/AssetStore.kt) | Asset details, search, prefetch, missing balances, buy/sell/swap availability from config versions, wallet default balances, default asset seeding (`sync_default_assets`, stakeable flags) |
| [`GemBalanceService`](../gemstone/src/services/balance/mod.rs) | [`GemBalanceStore`](../gemstone/src/services/balance/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/BalanceStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/BalanceStore.kt) | Coin/token/stake/earn balance updates via gateway |
| [`GemBannerService`](../gemstone/src/services/banner/mod.rs) | [`GemBannerStore`](../gemstone/src/services/banner/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/BannerStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/BannerStore.kt) | Banner state rules |
| [`GemContactService`](../gemstone/src/services/contact/mod.rs) | [`GemContactStore`](../gemstone/src/services/contact/store.rs), [`GemFileStore`](../gemstone/src/services/file/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/ContactStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/ContactStore.kt) | Contact writes, address-name sync and avatar files (`save_avatar`, `remove_avatar`, removed on delete) |
| [`GemDeviceService`](../gemstone/src/services/device/mod.rs) | [`GemDeviceStore`](../gemstone/src/services/device/store.rs), [`GemDevicePlatform`](../gemstone/src/services/device/platform.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/DeviceStore.swift), [Swift platform](../../ios/Packages/GemstoneServices/Sources/Device/DevicePlatform.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/DeviceStore.kt), [Kotlin platform](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/device/DeviceRepository.kt) | Device registration and subscription sync; Core builds the `Device` from the foreign platform facts (id, push token/enabled, os/model/version/locale/store, currency) plus its own price-alert preference, and owns `synchronize`/`synchronize_if_needed` with in-flight sync coordination — `GemPriceAlertService` calls it directly (the foreign `GemDeviceSync` and both apps' sync coordinators are gone) |
| [`GemNameService`](../gemstone/src/services/name/mod.rs) | [`GemAddressStore`](../gemstone/src/services/name/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/AddressStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/AddressStore.kt) | Name resolution (incl. the `can_resolve_name` rule), address names; iOS `NameServiceable`/`GemstoneNameService` adapter removed, view models use the Core protocol |
| [`GemNftService`](../gemstone/src/services/nft/mod.rs) | [`GemNftStore`](../gemstone/src/services/nft/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/NftStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/NftStore.kt) | NFT sync, asset refresh, reporting |
| [`GemNodeService`](../gemstone/src/services/node/mod.rs) | [`GemNodeStore`](../gemstone/src/services/node/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/NodeStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/NodeStore.kt) | Node selection and custom nodes |
| [`GemNotificationService`](../gemstone/src/services/notification/mod.rs) | [`GemNotificationStore`](../gemstone/src/services/notification/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/NotificationStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/NotificationStore.kt) | In-app notifications sync; sync timestamp in wallet preferences |
| [`GemPerpetualService`](../gemstone/src/services/perpetual/mod.rs) | [`GemPerpetualStore`](../gemstone/src/services/perpetual/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PerpetualStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PerpetualStore.kt) | Perpetual markets and positions |
| [`GemPreferencesService`](../gemstone/src/services/preferences/mod.rs) | [`GemPreferencesStore`](../gemstone/src/services/preferences/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PreferencesStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PreferencesStore.kt) | Typed app preferences over a key-value store; also backs the gateway; `default_currency(locale)` rule used by both apps when no currency is set |
| [`GemPriceService`](../gemstone/src/services/price/mod.rs) | [`GemPriceStore`](../gemstone/src/services/price/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PriceStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PriceStore.kt) | Prices, rates, currency change, market data |
| [`GemPriceAlertService`](../gemstone/src/services/price_alert/mod.rs) | [`GemPriceAlertStore`](../gemstone/src/services/price_alert/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PriceAlertStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PriceAlertStore.kt) | Alerts sync; `set_enabled`/`enable_price_alert` request notification permissions (`GemNotificationPermissions`) and sync the device (`GemDeviceSync`) |
| [`GemStakeService`](../gemstone/src/services/stake/mod.rs) | [`GemStakeStore`](../gemstone/src/services/stake/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/StakeStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/StakeStore.kt) | Validators, delegations, and earn positions sync |
| [`GemStreamService`](../gemstone/src/services/stream/mod.rs) | — | — | — | Dispatches WebSocket events to the Core services; support typing goes through `GemSupportStore.update_typing`/`clear_typing` |
| [`GemStreamSubscriptionService`](../gemstone/src/services/stream/subscription.rs) | [`GemStreamConnection`](../gemstone/src/services/stream/connection.rs) | [Swift](../../ios/Packages/FeatureServices/StreamService/StreamConnection.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/StreamConnection.kt) | Price subscription bookkeeping (`setup_assets`, `resubscribe`, `add_prices`, `reset`) over the app WebSocket, which stays app-side |
| [`GemSubscriptionService`](../gemstone/src/services/subscription/mod.rs) | [`GemWalletStore`](../gemstone/src/services/wallet/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletStore.kt) | Wallet subscription changes |
| [`GemSupportService`](../gemstone/src/services/support/mod.rs) | [`GemSupportStore`](../gemstone/src/services/support/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/SupportStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/SupportStore.kt) | Message sync, pending/failed delivery of text and images, image preview files cached through `GemFileStore` (`image_file`, downloaded via `AlienProvider`) |
| [`GemAmountService`](../gemstone/src/services/amount/mod.rs) | — | [Swift](../../ios/Features/Transfer/Sources/Protocols/AmountDataProvidable.swift) | [Kotlin](../../android/features/transfer_amount/viewmodels/src/main/kotlin/com/gemwallet/android/features/transfer_amount/viewmodels/providers/AmountDataProvider.kt) | Amount screen rules for every flow (`GemAmountType`: transfer, deposit, withdraw, stake types, earn, perpetual): minimum value (stake/redelegate/USDC deposit-withdraw/perpetual order), reserve-for-fee, can-change-value, shows-asset-balance (`rules`), available/max value and whether the fee reserve applies (`limits` over `GemAmountBalance` incl. Tron votes, delegation balances, rewards, resources, perpetual reduce), and `validate` (zero, below minimum, insufficient balance) ([rules](../gemstone/src/services/amount/rules.rs)); iOS providers only supply `gemAmountType`, Android providers only `amountType` (+ perpetual balance) |
| [`GemRecipientService`](../gemstone/src/services/recipient/mod.rs) | — | [Swift](../../ios/Packages/PrimitivesComponents/Sources/ViewModels/AddressInputViewModel.swift) | [Kotlin](../../android/ui-models/src/main/kotlin/com/gemwallet/android/ui/models/name/AddressInputModel.kt) | Recipient input rules shared by the address input and recipient screens: validity of typed address vs a resolved name record (name, chain and address must match), checksummed recipient address, when to show the address error (never for name-shaped input), `recipient(chain, input, name_record, memo, references)` → `GemRecipient` ([rules](../gemstone/src/services/recipient/rules.rs)); name lookups stay on `GemNameService` |
| [`GemTransferService`](../gemstone/src/services/transfer/mod.rs) | — | [Swift](../../ios/Packages/GemstonePrimitives/Sources/Extensions/TransferData+GemstonePrimitives.swift) | [Kotlin](../../android/gemcore/src/main/kotlin/com/gemwallet/android/domains/confirm/ConfirmInputMapper.kt) | Shared transfer model `GemTransferData { input_type, recipient: GemRecipient, value, use_max_amount, minimum_value }` (`GemConfirmInput = from + transfer`) and the rules both apps duplicated: transaction type, transaction metadata, asset ids, fee asset, output type/action, approval data, spends-balance, available balance source (delegation/rewards/frozen/locked/withdrawable) and the pending-transaction factory incl. which HyperCore legs are tracked ([rules](../gemstone/src/services/transfer/rules.rs)); iOS `TransferData`/`TransferDataType` and Android `ConfirmParams` keep their typed shape but forward these rules to Core; Android `ConfirmParams` is packed/unpacked for navigation only through Core `confirm_input_encode/decode` (every variant), `SwapParams` holds the Core `SwapData`, `Generic` carries the Core output type/action, `TokenApprovalParams` and the kotlinx fallback are gone |
| [`GemTransactionStateService`](../gemstone/src/services/transaction_state/mod.rs) | [`GemTransactionStateStore`](../gemstone/src/services/transaction_state/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/TransactionStateStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/TransactionStateStore.kt) | Owns the pending-transaction poll loop ([tracker](../gemstone/src/services/transaction_state/tracker.rs)): per-chain intervals, hash replacement, cancellation and post-processing (balances on in-transit/completion; stake, earn and NFT syncs on completion, [rules](../gemstone/src/services/transaction_state/rules.rs)); the apps only call `track_pending`/`track`/`stop_tracking` and observe their database |
| [`GemTransactionsService`](../gemstone/src/services/transactions/mod.rs) | [`GemTransactionStore`](../gemstone/src/services/transactions/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/TransactionStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/TransactionStore.kt) | Transaction history sync; per-wallet and per-asset sync timestamps in wallet preferences |
| [`GemWalletConfigurationService`](../gemstone/src/services/wallet_configuration/mod.rs) | `GemWalletPreferencesService` | — | — | Initial wallet configuration sync, multi-signature banners; completion flag in wallet preferences |
| [`GemWalletPreferencesService`](../gemstone/src/services/wallet_preferences/mod.rs) | [`GemWalletPreferencesStore`](../gemstone/src/services/wallet_preferences/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletPreferencesStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletPreferencesStore.kt) | Per-wallet key-value preferences (sync timestamps, initial-load steps, wallet configuration flag, perpetual account mode; keys are a strum enum); cleared on wallet delete, completed for created wallets; Android keeps its reactive `UserConfig` copy of the perpetual mode for `GetWalletSummary` |
| [`GemWalletSessionService`](../gemstone/src/services/wallet_session/mod.rs) | [`GemWalletSessionStore`](../gemstone/src/services/wallet_session/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletSessionStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletSessionStore.kt) | Current wallet session |
| [`GemWalletService`](../gemstone/src/services/wallet/mod.rs) | [`GemWalletStore`](../gemstone/src/services/wallet/store.rs), [`GemKeystorePassword`](../gemstone/src/services/wallet/password.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletStore.kt) | Wallet import, creation, deletion (removes the avatar file through `GemFileStore`), chain setup, pin, rename and image url; wallet reads on `GemWalletStore` are synchronous so session lookups need no `await` |
| [`GemAvatarService`](../gemstone/src/services/avatar/mod.rs) | [`GemFileStore`](../gemstone/src/services/file/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Wallet/FileStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/FileStore.kt) | Wallet avatar: stores the image file through the app file store (removing the previous one), downloads image urls through `AlienProvider`, writes the wallet image url |
| [`GemDeviceRequestSigner`](../gemstone/src/services/device/signer.rs) | — | [Swift](../../ios/Packages/GemstonePrimitives/Sources/Extensions/GemDeviceRequestSigner+GemstonePrimitives.swift) | — | Device auth header for the stream WebSocket request; Android builds it in `AssetsModule` |
| [`GemSwapService`](../gemstone/src/services/swap/mod.rs) | [`GemKeystorePassword`](../gemstone/src/services/wallet/password.rs), [`GemSwapStore`](../gemstone/src/services/swap/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/SwapStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/SwapStore.kt) | Quotes (request from the wallet accounts, route preload, sorted by output), `swap_quote` mapping, `get_transfer` (quote data with permit2 signing through the Core keystore, recipient account, amount) and `suggest_pair` (most swapped receive asset, then recent swap assets, then the receive picker head; pay asset when the screen opens empty) ([rules](../gemstone/src/services/swap/rules.rs)) |
| [`GemAppUpdateService`](../gemstone/src/services/app_update/mod.rs) | — | — | — | Release for the store, version compare, skipped version via `GemPreferencesService` |
| [`GemFiatService`](../gemstone/src/services/fiat/mod.rs) | [`GemFiatStore`](../gemstone/src/services/fiat/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/FiatStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/FiatStore.kt) | Fiat quotes, transaction sync with asset prefetch |
| [`GemAuthService`](../gemstone/src/services/auth/mod.rs) | [`GemKeystorePassword`](../gemstone/src/services/wallet/password.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/KeystorePasswordStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/KeystorePasswordStore.kt) | Auth payload: nonce → auth message → Core keystore signature, device id from the device key ([rules](../gemstone/src/services/auth/rules.rs)) |
| [`GemChartService`](../gemstone/src/services/chart/mod.rs) | — | — | — | Price charts |
| [`GemExplorerService`](../gemstone/src/services/explorer/mod.rs) | — | — | — | Block explorer selection (preference) and transaction/address/token/NFT/validator links |
| [`GemAppStartService`](../gemstone/src/services/app_start/mod.rs) | — | — | — | App start (`run`), wallet start (`setup_wallet`) and `setup_wallets` (default asset seeding + chain setup for every wallet, used by iOS `OnstartService` and Android `CheckAccountsService`); each `run` step reports failures without stopping the rest |
| [`GemConfigService`](../gemstone/src/services/config/mod.rs) | — | — | — | Remote config, cached via `GemPreferencesService`; concurrent updates share one request |
| [`GemPortfolioService`](../gemstone/src/services/portfolio/mod.rs) | [`GemPortfolioStore`](../gemstone/src/services/portfolio/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PortfolioStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PortfolioStore.kt) | Portfolio chart for the wallet's held assets |
| [`GemRewardsService`](../gemstone/src/services/rewards/mod.rs) | — | — | — | Rewards and referrals; authenticated calls take a wallet and build the auth payload through `GemAuthService`; `redeem` enables the redeemed asset through `GemBalanceService` |
| [`GemScanService`](../gemstone/src/services/scan/mod.rs) | — | — | — | Transaction scanning |
| [`GemSearchService`](../gemstone/src/services/search/mod.rs) | [`GemSearchStore`](../gemstone/src/services/search/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/SearchStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/SearchStore.kt) | Wallet search: API + token lookup merge, asset/price/balance persistence, perpetuals, lists and search history keys ([rules](../gemstone/src/services/search/rules.rs)) |
| [`GemWalletConnectService`](../gemstone/src/services/wallet_connect/mod.rs) | [`GemConnectionStore`](../gemstone/src/services/wallet_connect/store.rs), [`GemWalletConnectSigner`](../gemstone/src/services/wallet_connect/signer.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/ConnectionStore.swift), [Swift signer](../../ios/Features/WalletConnector/Sources/WalletConnector/Services/WalletConnectorSigner.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/ConnectionStore.kt), [Kotlin signer](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletConnectSigner.kt) | Session proposal preparation (CAIP-2 chain parsing, wallet selection, origin verification), app metadata rule, session approval data (chains, accounts, methods, events), session model from namespace accounts, connection persistence (`add_connection`, `update_sessions` sync rule, `delete_session`), request parse → connection/account lookup and session chain validation → simulate → decode → app signer (`GemWalletConnectSignRequest` with wallet, account, session, simulation and a message or transaction payload) → encode ([rules](../gemstone/src/services/wallet_connect/rules.rs)); Android `WalletConnectRequestHandler` runs `handle_request` and the signer publishes a `WalletConnectPendingRequest` the request screen approves or rejects |

## App services

What stays on the app side, because it is a platform concern with no Core counterpart:

| Service | Notes |
| --- | --- |
| [`AppService/RateService`](../../ios/Packages/FeatureServices/AppService/RateService.swift) | App Store review prompt |
| [`AppService/AppLifecycleService`](../../ios/Packages/FeatureServices/AppService/AppLifecycleService.swift) | Scene phase orchestration of observers |
| [`ConnectionStatusService`](../../ios/Packages/FeatureServices/ConnectionStatusService) | Connectivity |
| [`SystemServices`](../../ios/Packages/SystemServices) | Connectivity, image gallery, local store |

Every other app service is gone and its callers hold the Core service; the four wrappers still to remove are listed under [iOS](#ios).

## Remaining

- **iOS `Packages/GemAPI`** — only the widget's one-endpoint client (`GemPriceWidget`) is left; the app and feature packages no longer depend on it.
- iOS `Primitives` keeps hand-written views of Core types (`GasPriceType`, `FeeRate`, `Fee`, `FeeSelection`, `CustomFeeEstimate`, `TransferAmount`, `BalanceRequirement`, and ids such as `WalletId`/`TransactionId`/`AssetId`). They stay: they are typed views bridged once at the seam, not a second source of truth.

## TODO — finish Core as the single owner of logic

Every app service forwards to a Core service or is platform glue, and the rules that used to live in view models live in `gemstone` with unit tests. What is left is below. Work it top to bottom inside each section; each item lands with the app-side code it replaces deleted, a Core test that would flip if the rule flipped, and its line removed from this file in the same commit. When the list is empty, audit again and write a new one.

### Design decisions to apply everywhere

- **One preferences owner.** `GemPreferencesService` (Core, over the key-value `GemPreferencesStore`) owns every product preference; iOS `GemstonePreferencesStore`/`ObservablePreferences` and Android `UserConfig`/`ConfigStore` are storage adapters plus SwiftUI/Compose observation, nothing else, and no service or view model injects both an app preferences object and `GemPreferencesServiceProtocol`. Two things to know when adding a key: iOS namespaces every key as `gemstone_*` except the handful whose pre-move values must carry over (`currency`, `appearance`, `is_perpetual_enabled`, `is_push_notifications_enabled`, `swap_slippage_bps`, the `perpetual_*` defaults, the per-chain `explorer_name_*` keys, and the four aliased ones — `current_wallet_id`, `price_alerts_enabled`, `is_hide_balance_enabled`, `is_accept_terms_completed`), and `currency` is mirrored into the `group.com.gemwallet.ios` app group for the price widget. The app lock is not a preference: `authRequired` and `getLockInterval` live in secure storage on both platforms (iOS `KeystorePassword`, Android `TinkGemPreferences` over `GemSecureStore`), each still falling back to its pre-move DataStore/`ConfigStore` value until the next write.
- **No `try? … ?? default` on Core calls.** Core preference reads are infallible (`GemPreferencesStore.get`/`GemWalletPreferencesStore.get` return `Option`), so getters return plain values; the only allowed fallback is the JSON→enum decode of a Core-provided enum in `GemPreferencesService+GemstonePrimitives.swift` (`currencyValue`, `chartPeriodValue`, …), never in a view model. Keychain-backed values are not preferences: the gateway's secure store implements the separate, fallible `GemSecureStore` trait (iOS `GemstoneSecurePreferencesStore`, Android `TinkGemPreferences`). A Core method either cannot fail (rule with a built-in default, like `includes_perpetual_collateral`) or its failure is surfaced (thrown, mapped to a localized error, or recorded through `services::failures::record`). Deliberate fail-open fallbacks (keep, do not "fix"): transaction scan outage, address-name lookup → cached names, perpetual account mode → stored mode, explorer name → first explorer, corrupt cached config → refetch, per-chain token search → no match for that chain. Same on Android: no bare `runCatching { coreCall }` without logging or a user-visible state — use `runCatchingCancellable` and handle the failure.
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
| `ConfirmParams` (sealed, 15 subclasses) + `Builder` | Android | 290 | `GemConfirmInput` + `GemTransferData` |
| `AmountParams` | Android | 171 | `GemTransferData` fields |
| `TransferData` mappers (`toTransferData`, `toDto`, `toConfirmParams`) | Android | 62 | `GemTransferData`, `GemTransactionInputType` |
| `SignerParams` | Android | 17 | now carries `GemConfirmData` itself, plus the input, the typed `Fee` and the signed amount |
| `Fee` (sealed), `FeeSelection`, `DestinationAddress` | Android | — | `GemTransactionLoadFee`, `GemConfirmFeeSelection`, `GemRecipient` |
| `ConfirmTransferPreload` | iOS | 2 | `GemConfirmData` |
| `TransferAmount` | iOS | 9 | `GemTransferAmount` — the typed view of Core's calculator output |
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

A screen should ask one thing for its data, and that thing should be the same on both platforms: on iOS the Core `Gem*Service` for the feature, on Android the coordinator that implements that feature's cases. Where a feature's decisions need no store or API they live on its constructible service (`GemStakeConfigService`, `GemAssetConfigService`, `GemChainService`); where it needs I/O it holds the I/O service too, and the split stays visible.

The features to shape this way, in the order the screens were built:

| Feature | iOS today | Android today | Target |
| --- | --- | --- | --- |
| Validator selection | `ValidatorSelectSceneViewModel` + `StakeSceneViewModel` each call `GemStakeConfigService` | `GetValidators`, `GetRecommendedValidator`, `GetRecommendedValidatorIds` cases | one `GemValidatorService` in Core over `GemStakeStore` (selectable, recommended, ids, by id), so neither app filters or sorts validators itself |
| Delegation | `DelegationSceneViewModel` holds `GemStakeConfigService` | `GetDelegation`/`GetDelegations` cases over `StakeDao` | delegation reads move behind `GemStakeService`; actions and claimability stay on `GemStakeConfigService` |
| Price alerts | `PriceAlertsSceneViewModel` holds `GemPriceAlertServiceProtocol` | four cases, three of them over `PriceAlertsDao` | the reads move onto `GemPriceAlertService` (it already owns the store trait), leaving the cases as one-line forwards or nothing |
| Transactions | `TransactionsViewModel` over GRDB requests | `GetTransactions`/`GetTransaction` over `TransactionsDao` | `GemTransactionsService` answers the point reads; only the observed list stays platform-side |
| Perpetuals | `PerpetualSceneViewModel` + observer trio | `PerpetualRepository` + observer trio | see the Hyperliquid item below; the reads join `GemPerpetualService` |
| NFT, contacts, support, banners, notifications | view models hold the Core service | cases over the DAO | same shape: the Core service answers, the app observes |

- **A case must not take a Room DAO.** It is the same violation as taking a repository: the case then owns a second read path Core cannot see, and the two drift. The cases that currently do (`GetPriceAlertsImpl`, `GetTransactionsImpl`, `GetContactsImpl`, `GetInAppNotificationsImpl`, `SupportMessagesImpl`, `NftCasesImpl`, `GetActiveBannersImpl`, `HasMultiSignImpl`, `GetShowWelcomeBannerImpl`, `ObserveFiatTransactionsImpl`, `GetAssetPriceUsdImpl`, `PendingTransactionsImpl`, `AddressNamesImpl`, `GetAssetPriceAlertStateImpl`, `HasAssetPriceAlertsImpl`, `GetTransactionImpl`) took the DAO when their repository was deleted — that was the mechanical half. The second half is moving each read onto the Core service that already owns the matching `Gem*Store`, and leaving the app only what Core cannot express: **an observed list**. For those, the case holds the platform *store adapter* — the same `Gemstone*Store` Core writes through, which is where `GemstoneSupportStore.typingAgent` already lives — never the DAO. The stake cases are converted: `GemstoneStakeStore` carries the observed reads (`observeValidators`, `observeDelegations`, `observeDelegation`, `getValidator`) and the four stake cases hold the adapter, so `StakeDao` has exactly one reader again.

### Hyperliquid streaming belongs in Core

Both apps carry the same five-file perpetual streaming stack — iOS `HyperliquidObserverService`, `HyperliquidSubscriptionService`, `HyperliquidEventHandler`, `PerpetualNodeService`, `PerpetualObservable` (238 lines); Android `HyperliquidObserverService`, `HyperliquidSubscriptionService`, `HyperliquidEventHandler`, `ObservePerpetualWallet` (216 lines) — and they duplicate the same three decisions: which subscriptions a wallet needs, how a frame is built, and which service each message feeds. Core already has this exact shape for the device stream (`GemStreamService` + `GemStreamSubscriptionService` over the `GemStreamConnection` foreign trait), so the move is: `GemPerpetualStreamService` owns the subscription set, the frames and the dispatch to `GemPerpetualService`; each app keeps only its socket (`GemStreamConnection`) and the chart sink. The `"ws"` path suffix and the websocket URL rule go with it, which also clears the last item from the iOS consistency list.

### Rust (core/gemstone)

- Free functions become services. A `#[uniffi::export] pub fn` is a rule with no home: each app reaches it through its own extension (`Chain.matches(query:)`, `List<Chain>.filter(query)`), and those extensions are where app-side variants creep back in. `GemChainService` (`get_chains`, `get_matching_chains`, `is_valid_network_id`) is the shape: a `uniffi::Object` with a `new()` constructor, held like any other service. The remaining 82 exports, grouped by the service they belong in — take one group per commit, delete the app extensions that wrapped them, and keep the free function only where a single call has no siblings:

| Target service | Functions to fold in |
| --- | --- |
| ~~`GemAssetService` (new)~~ → `GemAssetConfigService` (done) | the twelve static asset lookups (rank, default assets, fee asset ids, enabled-by-default, swapable, chain asset, action filters, popular ids, default token chain, matching assets) on the constructible service the value extensions can hold; `GemAssetsService` keeps only what needs the API or the store |
| ~~`GemAddressService`~~ (done) | validation, checksum, short form and formatting; the `Chain` extensions and the address formatters on both apps hold one instance |
| ~~`GemStakeService`~~ → `GemStakeConfigService` (done) | the seven stake rules sit on the constructible config service, not the I/O one: `GemStakeService` needs a gateway, a static API client and two stores, so rules hung off it are only reachable through a mock — which silently emptied four iOS view-model tests. |
| ~~`GemNftService`~~ (done) | sorting and the verified/unverified split are methods on `GemNftService`; both callers are view models that already hold it. The nullable-status default went back to the two store adapters, where it is a column default, not a rule |
| ~~`GemPriceAlertService`~~ → `PriceAlertFormatter` (done) | id, ordering, notification type and display rule joined the price suggestions on the constructible formatter, because `PriceAlert.id` is the SwiftUI `Identifiable` conformance and the iOS store adapter writes it |
| ~~`GemPerpetualService`~~ → `GemPerpetual` (done) | funding APR and the two order builders joined the provider-scoped formatter object (renamed `Perpetual` → `GemPerpetual`), which also let `provider` leave both order inputs; `collateral_asset_id` stayed on `GemPerpetualService` — its one caller injects it |
| ~~`GemWalletService`~~ (done) | ordering and display account stayed on `GemWalletService` (both callers hold it); the keystore id moved to `GemKeystore`, which every caller already had open, and the wallet total and PnL rule to `BalanceCalculator` |
| ~~`GemConfirmService`~~ (done) | each one went to the constructible service that already owns its concept: the confirm-input codec to `GemTransferService` (it owns `GemConfirmInput`), the transfer amount to `GemAmountService`, the acquire flow to `GemAssetConfigService`, and the four fee decisions — custom gas price, custom fee estimate, default priority, insufficient network fee — to a new `GemFeeService` ([fee.rs](../gemstone/src/fee.rs)). Android's `ConfirmParams` now holds one `GemTransferService` instead of building one per call |
| ~~`GemPaymentService`~~ (done) | `decode_url`, `destination`, `transfer_destination` and `decoded_transfer` on a constructible service (the QR decode path runs from value types on both apps); the network half — loading a Solana Pay link — is `GemPaymentLinkService`, renamed from `PaymentService` |
| ~~`GemDeeplinkService`~~ (done) | `build_url`, `build_gem_url` and `url_action`; every caller is a value extension, a Compose menu or a navigation object, so the service is constructible |
| ~~`GemWalletConnectService`~~ (done) | the CAIP-2 lookups are `GemChainService.caip2_namespace/caip2_reference/chain_from_caip2` and the dapp short name is `GemApplicationMetadataService.short_name` (both are read from Compose scenes and value extensions); `siwe_try_parse` and `siwe_validate` had no caller on either platform and `permit2_data_to_eip712_json` has only a Core one, so all three left the FFI |
| ~~`GemTransactionStateService`~~ (done) | neither app called these four: they are internal Core functions now, and the unused `transaction_timeout_ms` is deleted |
| `GemSupportService` | `parse_support_message_display_content` stays a free function: it is one markdown parse with no siblings, and both callers (an iOS message view model, an Android message composable) are per-message value code |
| ~~`GemAppUpdateService`~~ (done) | `is_version_higher` is a method and `lib_version` is no longer exported |
| Keep as free functions | `generate_device_key_pair`, `decode_private_key`, `encode_private_key`, `supports_private_key_import`, `create_auth_message` — key material, called once at a boundary that already owns the secret |

- No service is constructed at a call site. Every `Gem*Service` is built once in `ServicesFactory` (iOS) or a Hilt module (Android) and injected. Two places still break this and need a home for the instance: iOS `NetworkSelectorViewModel` (its `SelectableListAdoptable` initializer is fixed by the protocol, so the sheet holds its own `GemChainService()`), and Android's `selectFilterChain`/`ContactChainSelectScene` composables, which build one per composition.

- One-sided exports still open: `wallet_connect::authentication_chain_ids` waits on iOS WalletConnect authentication; `nft::report` waits on an Android report screen; `wallet_preferences::is_initial_load_completed` drives the iOS wallet empty state and has no Android equivalent; `reset_transactions_timestamp` is an iOS developer action.
- Transfer model: generate the `TransactionInputType` enum from typeshare so the primitives tuple enum, the gemstone named-field enum and the Swift/Kotlin enums collapse (685 Core, 52 Android, 5 iOS references — do it after both apps carry Core records through confirm, transaction construction is wallet-critical). **Not started.**

### iOS

- Confirm view models still assemble `GemSendInput` from app aggregates. See **The confirm seam** below — it is the last migration item before the `TransactionInputType` collapse.
- The rule for a failing Core call on iOS: a Core read whose `Optional` models a real state (no wallet yet, no selected node) returns that optional and says nothing; a failure the app recovers from is **logged** and the recovery is deliberate; a failure the user must act on is thrown and mapped to localized text. Deliberate recoveries, keep them: `PerpetualService.accountMode` → the stored mode, `AmountDataProvidable.limits` → zero available (fail-closed for a send), `String+Keystore` → utf8 bytes for pre-hex passwords, `LocalKeystore.keystorePassword` → `""` is the "no password stored yet" sentinel, not a swallowed error. The 513 `try?` left outside tests are value conversions — JSON round trips, formatter parses, `wallet.account(for:)` — where `nil` is the render path, plus the developer screen; leave them.
- Two store differences remain, and neither is an adapter bug: `banners.walletId`/`banners.assetId` are foreign keys and SQLite does not apply `INSERT OR IGNORE` conflict resolution to foreign keys, so a banner for an asset the app has not stored throws where Android stores it; `PortfolioStore` reads through `asset_info`, which joins `accounts`, so a wallet without an account row for the chain contributes nothing.
- Dead code: the two TODOs are not stale — `NavigationHandler`'s `.stake` deep link is an unimplemented branch and `TransactionScene`'s corner radius is an open iOS 26 styling question; both mark real gaps and stay until someone closes them. The two "delete in 2026" `FileMigrator` calls (`LocalKeystore`, `DB.swift`) move the keystore and the database from the documents directory to application support on launch; deleting them strands anyone who has not opened the app since the move — losing their keystore — so this needs install-base data, not a code decision.
- Consistency, still open: one extension-file suffix (`+GemstoneSwift` vs `+GemstonePrimitives`, three `Wallet` extensions in one package); `LocalKeystore` throws the same English sentence from two places and Android already localizes it as `errors_keystore_access`; `PerpetualNodeService`'s `"ws"` path suffix belongs to Core node config; the network-id check inlined in `AddNodeSceneViewModel` (`ChainConfig.config(chain:).networkId == nodeStatus.chainId`) is a Core rule. The `JSONEncoder`/`JSONSerialization` in `WalletConnectorService` and `WalletConnectResponseType` sit on the dapp-JSON boundary and belong there.
- Naming, still open: `GemstoneNftStore` wraps `NFTStore` and `ConnectionStore` wraps `ConnectionsStore` (one name per store), and untyped `.map()` conversions where Android has `toPrimitives()`.

### Android

- Case naming: fold the legacy `com.gemwallet.android.cases.<area>/` (39 interfaces, 133 importers) into `application/<area>/cases/` and drop the `Case` suffix the package already implies (`SearchTokensCase` → `SearchTokens`, `GetListNftCase` → `GetListNft`, `AddNodeCase` → `AddNode`). Needs a judgement call per area where the two trees disagree (`cases/banners` vs `application/banner`; `cases/nodes` and `cases/wallet` have no `application` counterpart). Do it before the repository work so converted code lands in the right package.

- Repositories to retire, in this order — each becomes cases (interface in `gemcore` `application/<area>/cases/`, one implementation in `data/coordinators/<area>/` holding the Core service, or the DAO when the screen needs an observed read). Callers move to the case; the repository is deleted in the same commit:

| Repository | Callers | What it holds today | Target |
| --- | --- | --- | --- |
| `PerpetualRepository` (+`Impl`) | 18 | six observed reads over the perpetual DAOs | one case per read (`GetPerpetuals`, `GetPerpetual`, `GetPositions`, `GetPosition`, `GetPerpetualBalance`). Do this one with a device: the reads are covered by `androidTest` suites through `FakePerpetualRepository`, which has to be reworked into fakes of the new cases and cannot be verified by `just test` |
| `WalletConnectorService` (was `BridgesRepository`) | 9 | 272 lines of Reown SDK session handling with Core mixed in; named like iOS now | expose approve/reject/pair/respond as cases so the bridge view models stop holding the adapter itself |
| `WalletsRepository` (+`Impl`) | 31 | wallet CRUD over `WalletsDao` while Core has `GemWalletService`/`GemWalletStore` | invert it first: `GemstoneWalletStore`, `GemstoneConnectionStore` and `GemstoneTransactionStateStore` currently read Core's wallet data *through* this repository, so the DAO has to move into those adapters before the remaining callers can become cases over `GemWalletService` |
| `AssetsRepository` | 41 | 262 lines: observed asset lists, plus `GemAssetsService`/`GemBalanceService` orchestration | split — observed reads become `GetAssetsInfo*` cases over `AssetsDao`, the orchestration is already Core's |
| `SessionRepository` (+`Impl`) | 107 | current wallet and currency, `session()` StateFlow | last, in slices: `GetCurrentWallet`/`SetCurrentWallet`/`ObserveSession` over `GemWalletSessionService` and currency over `GemPreferencesService` |

  The observer services in the same module (`DeviceObserverService`, `StreamObserverService`, `HyperliquidObserverService`, `HyperliquidSubscriptionService`) are long-lived stream adapters, not cases — they stay, but move out of `data/repositories` when the repositories are gone.

- `UserConfig` is the preferences adapter now; only the app-lock keys are left outside Core, in secure storage by design. What remains is deleting the `ConfigStore` fallback for `auth` once enough installs have written the secure value.
- Error handling: suspend work uses `runCatchingCancellable` and logs what failed. The plain `runCatching` calls left are around non-suspend work (URI parsing, `valueOf`, `startActivity`, focus requests, JSON decode helpers), where cancellation cannot be swallowed — leave them.
- Compose/platform deprecations still warned by the Kotlin compiler: `rememberModalBottomSheetState(skipPartiallyExpanded:confirmValueChange:)` (3 sites, needs the `rememberBottomSheetState(Hidden)` migration), `TabRowDefaults.containerColor` in `PortfolioChartScene`, `FirebaseMessaging.token` `Task` in `RequestrPushToken`. The clipboard ones are gone: the 14 screens take the platform `ClipboardManager` from the context (`Context.clipboardManager()`), so neither `Clipboard.nativeClipboard` nor the `NativeClipboard` typealias is used.
- Localization: `DevelopScene`/`PaymentsScene` are developer-only screens and stay English. Still open: 59 hardcoded `dp` values (worst: `SupportMessageBubble`, `ReceiveScreen`, `ImportScreen`, `WalletTypeTab`, `FiatScene`).
- Store adapter divergences (mirror iOS or fix iOS, per method): `AssetStore.saveAsset` bumps `updatedAt` where iOS does not (the market is not a divergence — it is written through `PriceStore.saveMarket` from `GemPriceService.update_market`); `ConnectionStore` carries three non-trait methods on the adapter; `PriceStore` writes currency-tagged rows iOS never writes and `mapNotNull`s unparsable ids; `TransactionStateStore` writes swap amounts nothing reads; `NftStore` fills two legacy image columns from Core's single preview url; `WalletSessionStore.setCurrentWalletId` still creates the session row.
- Layering, still open: cases that mix a Core service with a repository (`SearchSwapAssetsImpl`, `ConfirmTransactionImpl` + `RecentAssetsService`, `GetWalletSummaryImpl` with six collaborators); `AssetsRepository.add` also calling the stream service; `TokensRepository`'s query strategy, whose seven callers should hold `GemSearchService` and pick `searchAssets` or `search` per site the way iOS does, and its `ensure_token_asset` strategy with an Android-only "retry as a text query" fallback; `BannersRepository`'s merge rule; `data/services/remote-gem` and the `walletconnect/{reown,noop}` modules as candidates to fold. A case holding a Room DAO for an observed read is **not** a violation — `GetShowWelcomeBannerImpl`, `ObserveFiatTransactionsImpl` and `GetAssetPriceUsdImpl` are the shape to copy once their repositories are gone.
- Consistency: `fromJson` (swallows) vs `decodeJson` (throws) vs `GemAsset.toPrimitives()` `runCatching` — one bridging convention; `toChain()` vs `requireChain()` used interchangeably; `NodeStore` String/Chain twin methods; five `getAssetsInfo*` overloads; `tx` abbreviations (`txProperties`, `DbTxSwapMetadata`, `txId`); `fetch*`/`resolve*` names (`NftRepository`, `RequestSwapQuotesImpl`, `AssetsResultsViewModel`, `SyncFiatTransactionsImpl`); `*Service` classes inside the repositories and coordinators modules; non-adapter files in the `gemstone` package (`NotificationPermissions`, `StreamConnection`, `FileStore`, `WalletConnectSigner`); adapter names differing from iOS for the same trait (`ConnectionsDao`/`ConnectionsStore`, merged price+rate store, `TransactionStateStore` collaborators).
- Earn flow: Android has no Earn surface yet (no `StakeProviderType.Earn` reader, no `AmountParams.Earn`, no `ConfirmParams.Earn`, `GemDelegationAction.DEPOSIT` maps to nothing); build the Earn scene, the amount provider and the confirm params on top of `GemStakeService.sync_earn`/`get_earn_data`, `GemAmountType::Earn` and `GemTransactionInputType::Earn` (iOS `EarnSceneViewModel` + `AmountEarnViewModel` are the reference). This is a feature, not a consolidation, so plan it as its own batch.

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
