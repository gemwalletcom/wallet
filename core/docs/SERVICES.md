# Gemstone Services

Core-owned services live in `core/gemstone/src/services/<name>/` (`mod.rs`, `model.rs`, `rules.rs`, `store.rs`, `error.rs` — only the files a service needs). A service owns the flow (API + rules); the app implements the `Gem*Store` trait over its database or preferences. Both apps construct the service in their DI (`ServicesFactory.swift`, Hilt modules).

Status: **Done** = flow in Core and both apps use it · **In progress** = being migrated · **Planned** = still app-side.

| Service | Store trait | Status | Notes |
| --- | --- | --- | --- |
| `GemAssetDiscoveryService` | `GemAssetDiscoveryStore` | Done | Discovers wallet assets, enables them, prefetches metadata |
| `GemAssetsService` | `GemAssetStore` | Done | Asset details, search, prefetch, missing balances |
| `GemBalanceService` | `GemBalanceStore` | Done | Coin/token/stake/earn balance updates via gateway |
| `GemBannerService` | `GemBannerStore` | Done | Banner state rules |
| `GemDeviceService` | `GemDeviceStore` | Done | Device registration and subscription sync |
| `GemNameService` | `GemAddressStore` | Done | Name resolution, address names |
| `GemNftService` | `GemNftStore` | Done | NFT sync, asset refresh, reporting |
| `GemNodeService` | `GemNodeStore` | Done | Node selection and custom nodes |
| `GemNotificationService` | `GemNotificationStore` | Done | In-app notifications sync |
| `GemPerpetualService` | `GemPerpetualStore` | Done | Perpetual markets and positions |
| `GemPriceService` | `GemPriceStore` | Done | Prices, rates, currency change, market data |
| `GemPriceAlertService` | `GemPriceAlertStore` | In progress | Alerts sync done; enabled flag moving to `GemPreferencesService` |
| `GemStakeService` | `GemStakeStore` | Done | Validators and delegations sync |
| `GemSubscriptionService` | `GemWalletStore` | Done | Wallet subscription changes |
| `GemTransactionStateService` | `GemTransactionStateStore` | Done | Pending transaction status updates |
| `GemTransactionsService` | `GemTransactionStore` | Done | Transaction history sync |
| `GemPreferencesService` | `GemPreferencesStore` | In progress | Typed app preferences over a key-value store |
| `GemAuthService` | — | Done | Wallet auth payloads |
| `GemChartService` | — | Done | Price charts |
| `GemConfigService` | — | Done | Remote config |
| `GemFiatService` | — | Done | Fiat quotes and transactions |
| `GemPortfolioService` | — | Done | Portfolio |
| `GemRewardsService` | — | Done | Rewards and referrals |
| `GemScanService` | — | Done | Transaction scanning |
| `GemSupportService` | — | Done | Support chat |
| `GemWalletConfigurationService` | — | Done | Wallet configuration |
| Earn positions | — | Planned | Android has no earn feature yet |

## Conventions

- Identifiers cross the FFI typed: `WalletId`, `AssetId`, `Chain`, `NFTAssetId`, `Currency`; store row ids stay `String`.
- Store methods: `get_*` reads, `set_*` preferences, `save_*` upserts, `update_<items>(…, items, delete_ids)` for reconcile writes, `delete_*` removals.
- Rules live in `rules.rs` with unit tests; `primitives` types stay policy-free.
