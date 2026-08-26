# Gemstone Services

Core-owned services live in [`core/gemstone/src/services/`](../gemstone/src/services/) as `<name>/{mod,model,rules,store,error}.rs` (only the files a service needs). A service owns the flow (API + rules); each app implements the `Gem*Store` trait over its database or preferences and constructs the service in DI ([`ServicesFactory.swift`](../../ios/Gem/Services/ServicesFactory.swift), Hilt modules under [`android/data/repositories/.../di`](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/di/) and [`android/data/coordinators/.../di`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/di/)).

Status: **Done** = flow in Core, both apps use it · **In progress** = being migrated · **Review** = app service not yet reviewed for Core-movable logic · **App-only** = platform concern, stays in the app · **Planned** = queued.

## Core services

| Service | Store | iOS adapter | Android adapter | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| [`GemAssetDiscoveryService`](../gemstone/src/services/asset_discovery/mod.rs) | [`GemAssetDiscoveryStore`](../gemstone/src/services/asset_discovery/store.rs) | [Swift](../../ios/Packages/FeatureServices/DiscoverAssetsService/GemstoneAssetDiscoveryStore.swift) | [Kotlin](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/asset/GemstoneAssetDiscoveryStore.kt) | Done | Discovers wallet assets, enables them, prefetches metadata |
| [`GemAssetsService`](../gemstone/src/services/assets/mod.rs) | [`GemAssetStore`](../gemstone/src/services/assets/store.rs) | [Swift](../../ios/Packages/FeatureServices/AssetsService/GemstoneAssetStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/assets/GemstoneAssetStore.kt) | Done | Asset details, search, prefetch, missing balances, buy/sell/swap availability from config versions |
| [`GemBalanceService`](../gemstone/src/services/balance/mod.rs) | [`GemBalanceStore`](../gemstone/src/services/balance/store.rs) | [Swift](../../ios/Packages/FeatureServices/BalanceService/GemstoneBalanceStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/assets/GemstoneBalanceStore.kt) | Done | Coin/token/stake/earn balance updates via gateway |
| [`GemBannerService`](../gemstone/src/services/banner/mod.rs) | [`GemBannerStore`](../gemstone/src/services/banner/store.rs) | [Swift](../../ios/Packages/FeatureServices/BannerService/GemstoneBannerStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/banners/GemstoneBannerStore.kt) | Done | Banner state rules |
| [`GemDeviceService`](../gemstone/src/services/device/mod.rs) | [`GemDeviceStore`](../gemstone/src/services/device/store.rs) | [Swift](../../ios/Packages/FeatureServices/DeviceService/GemstoneDeviceStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/device/GemstoneDeviceStore.kt) | Done | Device registration and subscription sync |
| [`GemNameService`](../gemstone/src/services/name/mod.rs) | [`GemAddressStore`](../gemstone/src/services/name/store.rs) | [Swift](../../ios/Packages/FeatureServices/AddressNameService/GemstoneAddressStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/addresses/GemstoneAddressStore.kt) | Done | Name resolution, address names |
| [`GemNftService`](../gemstone/src/services/nft/mod.rs) | [`GemNftStore`](../gemstone/src/services/nft/store.rs) | [Swift](../../ios/Packages/FeatureServices/NFTService/GemstoneNftStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/nft/GemstoneNftStore.kt) | Done | NFT sync, asset refresh, reporting |
| [`GemNodeService`](../gemstone/src/services/node/mod.rs) | [`GemNodeStore`](../gemstone/src/services/node/store.rs) | [Swift](../../ios/Packages/ChainServices/NodeService/GemstoneNodeStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/nodes/GemstoneNodeStore.kt) | Done | Node selection and custom nodes |
| [`GemNotificationService`](../gemstone/src/services/notification/mod.rs) | [`GemNotificationStore`](../gemstone/src/services/notification/store.rs) | [Swift](../../ios/Packages/FeatureServices/NotificationService/GemstoneNotificationStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/notifications/GemstoneNotificationStore.kt) | Done | In-app notifications sync |
| [`GemPerpetualService`](../gemstone/src/services/perpetual/mod.rs) | [`GemPerpetualStore`](../gemstone/src/services/perpetual/store.rs) | [Swift](../../ios/Packages/FeatureServices/PerpetualService/GemstonePerpetualStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/perpetual/GemstonePerpetualStore.kt) | Done | Perpetual markets and positions |
| [`GemPreferencesService`](../gemstone/src/services/preferences/mod.rs) | [`GemPreferencesStore`](../gemstone/src/services/preferences/store.rs) | [Swift](../../ios/Packages/Blockchain/Sources/Gateway/GemstonePreferencesStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/config/GemstonePreferencesStore.kt) | Done | Typed app preferences over a key-value store; also backs the gateway |
| [`GemPriceService`](../gemstone/src/services/price/mod.rs) | [`GemPriceStore`](../gemstone/src/services/price/store.rs) | [Swift](../../ios/Packages/FeatureServices/PriceService/GemstonePriceStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/prices/GemstonePriceStore.kt) | Done | Prices, rates, currency change, market data |
| [`GemPriceAlertService`](../gemstone/src/services/price_alert/mod.rs) | [`GemPriceAlertStore`](../gemstone/src/services/price_alert/store.rs) | [Swift](../../ios/Packages/FeatureServices/PriceAlertService/GemstonePriceAlertStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/pricealerts/GemstonePriceAlertStore.kt) | Done | Alerts sync; enabled flag via `GemPreferencesService` |
| [`GemStakeService`](../gemstone/src/services/stake/mod.rs) | [`GemStakeStore`](../gemstone/src/services/stake/store.rs) | [Swift](../../ios/Packages/ChainServices/StakeService/GemstoneStakeStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/stake/GemstoneStakeStore.kt) | Done | Validators and delegations sync |
| [`GemStreamService`](../gemstone/src/services/stream/mod.rs) | — | — | — | Done | Dispatches WebSocket events to the Core services; typing state stays app-side |
| [`GemSubscriptionService`](../gemstone/src/services/subscription/mod.rs) | [`GemWalletStore`](../gemstone/src/services/subscription/store.rs) | [Swift](../../ios/Packages/FeatureServices/DeviceService/GemstoneWalletStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/wallets/GemstoneWalletStore.kt) | Done | Wallet subscription changes |
| [`GemSupportService`](../gemstone/src/services/support/mod.rs) | [`GemSupportStore`](../gemstone/src/services/support/store.rs) | [Swift](../../ios/Packages/FeatureServices/SupportChatService/GemstoneSupportStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/support/GemstoneSupportStore.kt) | Done | Message sync, pending/failed delivery of text and images |
| [`GemTransactionStateService`](../gemstone/src/services/transaction_state/mod.rs) | [`GemTransactionStateStore`](../gemstone/src/services/transaction_state/store.rs) | [Swift](../../ios/Packages/FeatureServices/TransactionStateService/GemstoneTransactionStateStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/transactions/GemstoneTransactionStateStore.kt) | Done | Pending transaction status updates |
| [`GemTransactionsService`](../gemstone/src/services/transactions/mod.rs) | [`GemTransactionStore`](../gemstone/src/services/transactions/store.rs) | [Swift](../../ios/Packages/FeatureServices/TransactionsService/GemstoneTransactionStore.swift) | [Kotlin](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/transactions/GemstoneTransactionStore.kt) | Done | Transaction history sync |
| [`GemWalletConfigurationService`](../gemstone/src/services/wallet_configuration/mod.rs) | [`GemWalletConfigurationStore`](../gemstone/src/services/wallet_configuration/store.rs) | [Swift](../../ios/Packages/FeatureServices/AppService/GemstoneWalletConfigurationStore.swift) | [Kotlin](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/wallet_import/GemstoneWalletConfigurationStore.kt) | Done | Initial wallet configuration sync, multi-signature banners |
| [`GemAppUpdateService`](../gemstone/src/services/app_update/mod.rs) | — | — | — | Done | Release for the store, version compare, skipped version via `GemPreferencesService` |
| [`GemFiatService`](../gemstone/src/services/fiat/mod.rs) | [`GemFiatStore`](../gemstone/src/services/fiat/store.rs) | [Swift](../../ios/Packages/FeatureServices/FiatService/GemstoneFiatStore.swift) | [Kotlin](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/fiat/GemstoneFiatStore.kt) | Done | Fiat quotes, transaction sync with asset prefetch |
| [`GemAuthService`](../gemstone/src/services/auth/mod.rs) | — | — | — | Done | Wallet auth payloads |
| [`GemChartService`](../gemstone/src/services/chart/mod.rs) | — | — | — | Done | Price charts |
| [`GemConfigService`](../gemstone/src/services/config/mod.rs) | — | — | — | Done | Remote config, cached via `GemPreferencesService` |
| [`GemPortfolioService`](../gemstone/src/services/portfolio/mod.rs) | — | — | — | Done | Portfolio |
| [`GemRewardsService`](../gemstone/src/services/rewards/mod.rs) | — | — | — | Done | Rewards and referrals |
| [`GemScanService`](../gemstone/src/services/scan/mod.rs) | — | — | — | Done | Transaction scanning |

## App services (iOS is the reference)

Every app service is listed so nothing is missed; "Review" rows are the remaining migration work.

| App service | Core service | Status | Notes |
| --- | --- | --- | --- |
| [`ActivityService`](../../ios/Packages/FeatureServices/ActivityService) | — | Review | |
| [`AddressNameService`](../../ios/Packages/FeatureServices/AddressNameService) | `GemNameService` | Done | |
| [`AppService/OnstartService`](../../ios/Packages/FeatureServices/AppService/OnstartService.swift) | — | Review | Startup migrations, asset import |
| [`AppService/OnstartAsyncService`](../../ios/Packages/FeatureServices/AppService/OnstartAsyncService.swift) | — | Done | Runs config update, availability sync, banner setup; Android: [`SyncService`](../../android/app/src/main/kotlin/com/gemwallet/android/services/SyncService.kt) |
| [`AppService/OnstartWalletService`](../../ios/Packages/FeatureServices/AppService/OnstartWalletService.swift) | `GemWalletConfigurationService` | Review | Configuration sync in Core; banner seeding and push permissions still app-side |
| [`AppService/ConfigService`](../../ios/Packages/FeatureServices/AppService/ConfigService.swift) | `GemConfigService` | Done | Thin actor that dedupes concurrent updates |
| [`AppService/ReleaseAlertService`](../../ios/Packages/FeatureServices/AppService/ReleaseAlertService.swift) | `GemAppUpdateService` | Done | Android: [`AppUpdateCoordinator`](../../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/update/AppUpdateCoordinator.kt) |
| [`AppService/RateService`](../../ios/Packages/FeatureServices/AppService/RateService.swift) | — | App-only | App Store review prompt |
| [`AppService/AppLifecycleService`](../../ios/Packages/FeatureServices/AppService/AppLifecycleService.swift) | — | Review | |
| [`AssetsService`](../../ios/Packages/FeatureServices/AssetsService) | `GemAssetsService` | Review | Sync, details, availability in Core; `ImportAssetsService.migrate` seeds bundled assets app-side |
| [`AuthService`](../../ios/Packages/FeatureServices/AuthService) | `GemAuthService` | Done | |
| [`AvatarService`](../../ios/Packages/FeatureServices/AvatarService) | — | App-only | Image files |
| [`BalanceService`](../../ios/Packages/FeatureServices/BalanceService) | `GemBalanceService` | Done | |
| [`BannerService`](../../ios/Packages/FeatureServices/BannerService) | `GemBannerService` | Done | |
| [`ConnectionsService`](../../ios/Packages/FeatureServices/ConnectionsService) | — | Review | WalletConnect sessions |
| [`ConnectionStatusService`](../../ios/Packages/FeatureServices/ConnectionStatusService) | — | App-only | Connectivity |
| [`ContactService`](../../ios/Packages/FeatureServices/ContactService) | — | Review | |
| [`DeviceService`](../../ios/Packages/FeatureServices/DeviceService) | `GemDeviceService` | Done | |
| [`DiscoverAssetsService`](../../ios/Packages/FeatureServices/DiscoverAssetsService) | `GemAssetDiscoveryService` | Done | |
| [`EarnService`](../../ios/Packages/FeatureServices/EarnService) | — | Planned | Android has no earn feature yet |
| [`FiatService`](../../ios/Packages/FeatureServices/FiatService) | `GemFiatService` | Done | Android: `SyncFiatTransactionsImpl`, `GetBuyQuotesImpl` |
| [`NFTService`](../../ios/Packages/FeatureServices/NFTService) | `GemNftService` | Done | |
| [`NotificationService`](../../ios/Packages/FeatureServices/NotificationService) | `GemNotificationService` | Done | |
| [`PerpetualService`](../../ios/Packages/FeatureServices/PerpetualService) | `GemPerpetualService` | Done | |
| [`PriceAlertService`](../../ios/Packages/FeatureServices/PriceAlertService) | `GemPriceAlertService` | Done | |
| [`PriceService`](../../ios/Packages/FeatureServices/PriceService) | `GemPriceService` | Done | |
| [`RewardsService`](../../ios/Packages/FeatureServices/RewardsService) | `GemRewardsService` | Done | |
| [`ServiceStatusService`](../../ios/Packages/FeatureServices/ServiceStatusService) | `GemServiceStatus` | Done | Thin wrapper over the Core client |
| [`StreamService`](../../ios/Packages/FeatureServices/StreamService) | `GemStreamService` | Done | Event handling in Core; socket connection and subscriptions app-side, see [DEVICE_WEBSOCKETS.md](DEVICE_WEBSOCKETS.md); Android: [`StreamEventHandler`](../../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/stream/StreamEventHandler.kt) |
| [`SupportChatService`](../../ios/Packages/FeatureServices/SupportChatService) | `GemSupportService` | Done | Typing state and image files stay app-side |
| [`SwapService`](../../ios/Packages/FeatureServices/SwapService) | — | Review | Swapper lives in Core; app wrapper to review |
| [`TransactionsService`](../../ios/Packages/FeatureServices/TransactionsService) | `GemTransactionsService` | Done | |
| [`TransactionStateService`](../../ios/Packages/FeatureServices/TransactionStateService) | `GemTransactionStateService` | Done | |
| [`WalletService`](../../ios/Packages/FeatureServices/WalletService) | — | Review | Keystore |
| [`WalletSessionService`](../../ios/Packages/FeatureServices/WalletSessionService) | — | App-only | Current wallet session |
| [`ChainServices/ChainService`](../../ios/Packages/ChainServices/ChainService) | — | Review | |
| [`ChainServices/ExplorerService`](../../ios/Packages/ChainServices/ExplorerService) | — | Review | |
| [`ChainServices/NodeService`](../../ios/Packages/ChainServices/NodeService) | `GemNodeService` | Done | |
| [`ChainServices/StakeService`](../../ios/Packages/ChainServices/StakeService) | `GemStakeService` | Done | |
| [`ChainServices/WalletConnectorService`](../../ios/Packages/ChainServices/WalletConnectorService) | — | Review | |
| [`SystemServices`](../../ios/Packages/SystemServices) | — | App-only | Connectivity, image gallery, local store |

## Conventions

- Identifiers cross the FFI typed: `WalletId`, `AssetId`, `Chain`, `NFTAssetId`, `Currency`; store row ids stay `String`.
- Store methods: `get_*` reads, `set_*` preferences, `save_*` upserts, `update_<items>(…, items, delete_ids)` for reconcile writes, `delete_*` removals.
- Rules live in `rules.rs` with unit tests; `primitives` types stay policy-free.
