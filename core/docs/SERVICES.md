# Gemstone Services

Core-owned services live in [`core/gemstone/src/services/`](../gemstone/src/services/) as `<name>/{mod,model,rules,store}.rs` (only the files a service needs); every service and store returns the shared [`GemServiceError`](../gemstone/src/services/error.rs). A service owns the flow (API + rules); each app implements the `Gem*Store` trait over its database or preferences and constructs the service in DI ([`ServicesFactory.swift`](../../ios/Gem/Services/ServicesFactory.swift), Hilt modules under [`android/data/repositories/.../di`](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/di/) and [`android/data/coordinators/.../di`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/di/)).

Status: **Done** = flow in Core, both apps use it · **In progress** = being migrated · **Review** = app service not yet reviewed for Core-movable logic · **App-only** = platform concern, stays in the app · **Planned** = queued.

## Core services

| Service | Store | iOS adapter | Android adapter | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| [`GemAssetDiscoveryService`](../gemstone/src/services/asset_discovery/mod.rs) | [`GemAssetDiscoveryStore`](../gemstone/src/services/asset_discovery/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/AssetDiscoveryStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/AssetDiscoveryStore.kt) | Done | Discovers wallet assets, enables them, prefetches metadata |
| [`GemAssetsService`](../gemstone/src/services/assets/mod.rs) | [`GemAssetStore`](../gemstone/src/services/assets/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/AssetStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/AssetStore.kt) | Done | Asset details, search, prefetch, missing balances, buy/sell/swap availability from config versions |
| [`GemBalanceService`](../gemstone/src/services/balance/mod.rs) | [`GemBalanceStore`](../gemstone/src/services/balance/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/BalanceStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/BalanceStore.kt) | Done | Coin/token/stake/earn balance updates via gateway |
| [`GemBannerService`](../gemstone/src/services/banner/mod.rs) | [`GemBannerStore`](../gemstone/src/services/banner/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/BannerStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/BannerStore.kt) | Done | Banner state rules |
| [`GemContactService`](../gemstone/src/services/contact/mod.rs) | [`GemContactStore`](../gemstone/src/services/contact/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/ContactStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/ContactStore.kt) | Done | Contact writes and address-name sync |
| [`GemDeviceService`](../gemstone/src/services/device/mod.rs) | [`GemDeviceStore`](../gemstone/src/services/device/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/DeviceStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/DeviceStore.kt) | Done | Device registration and subscription sync |
| [`GemNameService`](../gemstone/src/services/name/mod.rs) | [`GemAddressStore`](../gemstone/src/services/name/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/AddressStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/AddressStore.kt) | Done | Name resolution, address names |
| [`GemNftService`](../gemstone/src/services/nft/mod.rs) | [`GemNftStore`](../gemstone/src/services/nft/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/NftStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/NftStore.kt) | Done | NFT sync, asset refresh, reporting |
| [`GemNodeService`](../gemstone/src/services/node/mod.rs) | [`GemNodeStore`](../gemstone/src/services/node/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/NodeStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/NodeStore.kt) | Done | Node selection and custom nodes |
| [`GemNotificationService`](../gemstone/src/services/notification/mod.rs) | [`GemNotificationStore`](../gemstone/src/services/notification/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/NotificationStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/NotificationStore.kt) | Done | In-app notifications sync |
| [`GemPerpetualService`](../gemstone/src/services/perpetual/mod.rs) | [`GemPerpetualStore`](../gemstone/src/services/perpetual/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PerpetualStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PerpetualStore.kt) | Done | Perpetual markets and positions |
| [`GemPreferencesService`](../gemstone/src/services/preferences/mod.rs) | [`GemPreferencesStore`](../gemstone/src/services/preferences/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PreferencesStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PreferencesStore.kt) | Done | Typed app preferences over a key-value store; also backs the gateway |
| [`GemPriceService`](../gemstone/src/services/price/mod.rs) | [`GemPriceStore`](../gemstone/src/services/price/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PriceStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PriceStore.kt) | Done | Prices, rates, currency change, market data |
| [`GemPriceAlertService`](../gemstone/src/services/price_alert/mod.rs) | [`GemPriceAlertStore`](../gemstone/src/services/price_alert/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/PriceAlertStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/PriceAlertStore.kt) | Done | Alerts sync; enabled flag via `GemPreferencesService` |
| [`GemStakeService`](../gemstone/src/services/stake/mod.rs) | [`GemStakeStore`](../gemstone/src/services/stake/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/StakeStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/StakeStore.kt) | Done | Validators, delegations, and earn positions sync |
| [`GemStreamService`](../gemstone/src/services/stream/mod.rs) | — | — | — | Done | Dispatches WebSocket events to the Core services; typing state stays app-side |
| [`GemSubscriptionService`](../gemstone/src/services/subscription/mod.rs) | [`GemWalletStore`](../gemstone/src/services/subscription/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletStore.kt) | Done | Wallet subscription changes |
| [`GemSupportService`](../gemstone/src/services/support/mod.rs) | [`GemSupportStore`](../gemstone/src/services/support/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/SupportStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/SupportStore.kt) | Done | Message sync, pending/failed delivery of text and images |
| [`GemTransactionStateService`](../gemstone/src/services/transaction_state/mod.rs) | [`GemTransactionStateStore`](../gemstone/src/services/transaction_state/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/TransactionStateStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/TransactionStateStore.kt) | Done | Pending transaction status updates |
| [`GemTransactionsService`](../gemstone/src/services/transactions/mod.rs) | [`GemTransactionStore`](../gemstone/src/services/transactions/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/TransactionStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/TransactionStore.kt) | Done | Transaction history sync |
| [`GemWalletConfigurationService`](../gemstone/src/services/wallet_configuration/mod.rs) | [`GemWalletConfigurationStore`](../gemstone/src/services/wallet_configuration/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletConfigurationStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletConfigurationStore.kt) | Done | Initial wallet configuration sync, multi-signature banners |
| [`GemWalletSessionService`](../gemstone/src/services/wallet_session/mod.rs) | [`GemWalletSessionStore`](../gemstone/src/services/wallet_session/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/WalletSessionStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/WalletSessionStore.kt) | Done | Current wallet session |
| [`GemAppUpdateService`](../gemstone/src/services/app_update/mod.rs) | — | — | — | Done | Release for the store, version compare, skipped version via `GemPreferencesService` |
| [`GemFiatService`](../gemstone/src/services/fiat/mod.rs) | [`GemFiatStore`](../gemstone/src/services/fiat/store.rs) | [Swift](../../ios/Packages/GemstoneServices/Sources/Stores/FiatStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/gemstone/FiatStore.kt) | Done | Fiat quotes, transaction sync with asset prefetch |
| [`GemAuthService`](../gemstone/src/services/auth/mod.rs) | — | — | — | Done | Wallet auth payloads |
| [`GemChartService`](../gemstone/src/services/chart/mod.rs) | — | — | — | Done | Price charts |
| [`GemExplorerService`](../gemstone/src/services/explorer/mod.rs) | — | — | — | Done | Block explorer selection (preference) and transaction/address/token/NFT/validator links |
| [`GemConfigService`](../gemstone/src/services/config/mod.rs) | — | — | — | Done | Remote config, cached via `GemPreferencesService` |
| [`GemPortfolioService`](../gemstone/src/services/portfolio/mod.rs) | — | — | — | Done | Portfolio |
| [`GemRewardsService`](../gemstone/src/services/rewards/mod.rs) | — | — | — | Done | Rewards and referrals |
| [`GemScanService`](../gemstone/src/services/scan/mod.rs) | — | — | — | Done | Transaction scanning |

## App services (iOS is the reference)

Every app service is listed so nothing is missed; "Review" rows are the remaining migration work.

| App service | Core service | Status | Notes |
| --- | --- | --- | --- |
| [`ActivityService`](../../ios/Packages/FeatureServices/ActivityService) | — | App-only | Recent activity rows, no rules; Android: `RecentAssetsService` |
| `AddAssetService` | `GemGateway` | Done | Wrapper removed; the add-asset view model reads token data through `GatewayService` |
| [`AddressNameService`](../../ios/Packages/GemstoneServices/Sources) | `GemNameService` | Done | |
| [`AppService/OnstartService`](../../ios/Packages/FeatureServices/AppService/OnstartService.swift) | — | App-only | Preference migrations and bundled asset seeding from the app asset configuration |
| [`AppService/OnstartAsyncService`](../../ios/Packages/FeatureServices/AppService/OnstartAsyncService.swift) | — | Done | Runs config update, availability sync, banner setup; Android: [`SyncService`](../../android/app/src/main/kotlin/com/gemwallet/android/services/SyncService.kt) |
| [`AppService/OnstartWalletService`](../../ios/Packages/FeatureServices/AppService/OnstartWalletService.swift) | `GemWalletConfigurationService` | Done | Configuration sync in Core; iOS-only banner seeding and push permission prompt stay app-side |
| [`AppService/ConfigService`](../../ios/Packages/FeatureServices/AppService/ConfigService.swift) | `GemConfigService` | Done | Thin actor that dedupes concurrent updates |
| [`AppService/ReleaseAlertService`](../../ios/Packages/FeatureServices/AppService/ReleaseAlertService.swift) | `GemAppUpdateService` | Done | Android: [`AppUpdateCoordinator`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/update/AppUpdateCoordinator.kt) |
| [`AppService/RateService`](../../ios/Packages/FeatureServices/AppService/RateService.swift) | — | App-only | App Store review prompt |
| [`AppService/AppLifecycleService`](../../ios/Packages/FeatureServices/AppService/AppLifecycleService.swift) | — | App-only | Scene phase orchestration of observers |
| [`AssetsService`](../../ios/Packages/GemstoneServices/Sources) | `GemAssetsService` | Done | Sync, details, availability in Core; remaining methods are store reads and bundled asset seeding |
| [`AuthService`](../../ios/Packages/GemstoneServices/Sources) | `GemAuthService` | Done | |
| [`AvatarService`](../../ios/Packages/FeatureServices/AvatarService) | — | App-only | Image files |
| [`BalanceService`](../../ios/Packages/GemstoneServices/Sources) | `GemBalanceService` | Done | |
| [`BannerService`](../../ios/Packages/GemstoneServices/Sources) | `GemBannerService` | Done | |
| [`ConnectionsService`](../../ios/Packages/FeatureServices/ConnectionsService) | — | App-only | WalletConnect SDK sessions |
| [`ConnectionStatusService`](../../ios/Packages/FeatureServices/ConnectionStatusService) | — | App-only | Connectivity |
| [`ContactService`](../../ios/Packages/GemstoneServices/Sources) | `GemContactService` | Done | Avatar files stay app-side; Android: `ContactsRepository` |
| [`DeviceService`](../../ios/Packages/GemstoneServices/Sources) | `GemDeviceService` | Done | |
| `DiscoverAssetsService` | `GemAssetDiscoveryService` | Done | Wrapper removed; view models call the Core service directly |
| `EarnService` | `GemStakeService` | Done | Wrapper removed; `sync_earn` and `get_earn_data` on the Core service, APR read from `GemStakeStore`; Android has no earn UI yet |
| `ExplorerService` | `GemExplorerService` | Done | Moved to Core with the selected-explorer preference; both apps call the Core service directly (legacy selection migrated once) |
| [`FiatService`](../../ios/Packages/GemstoneServices/Sources) | `GemFiatService` | Done | Android: `SyncFiatTransactionsImpl`, `GetBuyQuotesImpl` |
| [`NFTService`](../../ios/Packages/GemstoneServices/Sources) | `GemNftService` | Done | |
| `InAppNotificationService` | `GemNotificationService` | Done | Wrapper removed; view models call the Core service directly |
| [`PerpetualService`](../../ios/Packages/GemstoneServices/Sources) | `GemPerpetualService` | Done | |
| [`PriceAlertService`](../../ios/Packages/GemstoneServices/Sources) | `GemPriceAlertService` | Done | |
| [`PriceService`](../../ios/Packages/GemstoneServices/Sources) | `GemPriceService` | Done | |
| [`RewardsService`](../../ios/Packages/GemstoneServices/Sources) | `GemRewardsService` | Done | |
| `ServiceStatusService` | `GemServiceStatus` | Done | Wrapper removed on both apps; view models use the Core client directly |
| [`StreamService`](../../ios/Packages/FeatureServices/StreamService) | `GemStreamService` | Done | Event handling in Core; socket connection and subscriptions app-side, see [DEVICE_WEBSOCKETS.md](DEVICE_WEBSOCKETS.md); Android: [`StreamEventHandler`](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/stream/StreamEventHandler.kt) |
| [`SupportChatService`](../../ios/Packages/GemstoneServices/Sources) | `GemSupportService` | Done | Typing state and image files stay app-side |
| [`SwapService`](../../ios/Packages/FeatureServices/SwapService) | `GemSwapper` | Done | Thin wrapper on both apps, see [SWAPPER.md](../../docs/SWAPPER.md) |
| [`TransactionsService`](../../ios/Packages/GemstoneServices/Sources) | `GemTransactionsService` | Done | |
| [`TransactionStateService`](../../ios/Packages/GemstoneServices/Sources) | `GemTransactionStateService` | Done | |
| [`WalletService`](../../ios/Packages/FeatureServices/WalletService) | `GemKeystore` | Done | Keystore in Core, see [KEYSTORE_V4.md](KEYSTORE_V4.md); wallet rows app-side |
| [`WalletSessionService`](../../ios/Packages/GemstoneServices/Sources/WalletSessionService.swift) | `GemWalletSessionService` | Done | Current wallet id and wallet lookups in Core over `GemWalletSessionStore`; iOS keeps a thin sync wrapper, Android `SessionRepositoryImpl` uses it |
| [`GatewayService`](../../ios/Packages/GemstoneServices/Sources/Gateway) | `GemGateway` | Done | Typed gateway wrapper and chain service factory, moved from `Blockchain`/`ChainServices` into `GemstoneServices` |
| [`Signer`](../../ios/Packages/GemstoneServices/Sources/Signer) | `GemSigner` | Done | Transaction and message signing over the Core signer, moved into `GemstoneServices` |
| [`NodeService`](../../ios/Packages/GemstoneServices/Sources) | `GemNodeService` | Done | |
| `StakeService` | `GemStakeService` | Done | Wrapper removed; view models call `sync` on the Core service, APR read from `GemStakeStore` |
| [`ChainServices/WalletConnectorService`](../../ios/Packages/ChainServices/WalletConnectorService) | — | App-only | WalletConnect SDK bridge |
| [`SystemServices`](../../ios/Packages/SystemServices) | — | App-only | Connectivity, image gallery, local store |

## App packaging goal

Store adapters and the thin app services are being consolidated so both apps look the same:

- **iOS `Packages/GemstoneServices/Sources/Stores`** — every `Gem*Store` implementation, one file per store (`FiatStore.swift`, `BalanceStore.swift`, …) over the `Store`/`Preferences` packages. Feature packages depend on `GemstoneServices` instead of owning adapters; the app injects the gateway's preference stores into `GatewayService`.
- **iOS `Packages/GemstoneServices`** — the thin app-side services that wrap the Core services (`BalanceService.swift`, `FiatService.swift`, …), one file per service, tests under `Tests/`. The migrated `FeatureServices` move here as they are wired to `GemstoneStore`; wrappers that only forward calls (for example `AssetDiscoveryService`, `InAppNotificationService`, `MarketService`, `ChartService`) are removed and callers use the Core service directly; a wrapper stays only when it combines Core calls with app-side stores or files, or maps bridged JSON to typed models for several call sites.
- **Android** — the same split in Android's patterns: store adapters in one package (`data/repositories/.../gemstone`, one file per store) and the coordinators/repositories stay thin wrappers over the Core services.

Status: stores — done on both apps (iOS `GemstoneServices/Sources/Stores`, Android package `data.repositories.gemstone`); GemstoneServices — in progress: all migrated feature services moved (Fiat, NFT, Transactions, SupportChat, AddressName, Contact, Price, Device, Auth, Rewards, Notification, PriceAlert, Banner, DiscoverAssets, Assets, Balance, Earn, TransactionState, Perpetual); Stake and Node moved too; `Blockchain`, `ChainService` and `Signer` packages folded in (`Sources/Gateway`, `Sources/Signer`); pure-forwarding wrappers removed (AssetDiscovery, InAppNotification, Market, Chart, Stake, Earn, AddAsset).

## Conventions

- Identifiers cross the FFI typed: `WalletId`, `AssetId`, `Chain`, `NFTAssetId`, `Currency`; store row ids stay `String`.
- Store methods: `get_*` reads, `set_*` preferences, `save_*` upserts, `update_<items>(…, items, delete_ids)` for reconcile writes, `delete_*` removals.
- Rules live in `rules.rs` with unit tests; `primitives` types stay policy-free.
