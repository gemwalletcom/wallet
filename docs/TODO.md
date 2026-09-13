# Open work

x Contracts are in [ARCHITECTURE.md](ARCHITECTURE.md). **Delete an item's line in the commit that lands it.**

One item, one commit, both apps built and tested. Core rule + test, regenerate bindings, delete the app code it replaced on both sides, clear that screen's view leak while you are in it.

## 0. Make "session" mean one thing

Blocking: three shapes answer to the word, so there is no reference to copy. [`fiat/session.rs`](../core/gemstone/src/services/fiat/session.rs) is the one to keep.

- **V1** **M** [`confirm/session.rs`](../core/gemstone/src/services/confirm/session.rs) — an Object with `async load`/`state`/`execute`. It is a service; rename it.
- **V2** **S** [`swap/session.rs`](../core/gemstone/src/services/swap/session.rs) — answers `quote`, `is_transfer_loading`, `refreshes_quotes`, `accepts_quotes` outside `view_state`. Fold them in.
- **V3** **S** Session mocks belong in the testkits; iOS builds `GemSwapSession.mock()` inside [one test file](../ios/Features/Swap/Tests/SwapTests/SwapButtonViewModelTests.swift).
- **V4** **S** Add a session column to the [screen services table](SERVICES.md#screen-services).

## 1. Lists get a row record

Copy: [`GemAssetRow`](../core/gemstone/src/services/assets/model.rs) → [iOS](../ios/Packages/PrimitivesComponents/Sources/ViewModels/ListAssetItemViewModel.swift), [Android](../android/gemcore/src/main/kotlin/com/gemwallet/android/domains/asset/aggregates/AssetInfoDataAggregate.kt).

- **R1** **S** Perpetual position — [iOS](../ios/Features/Perpetuals/Sources/ViewModels/PerpetualPositionItemViewModel.swift), [Android](../android/features/perpetual/presents/src/main/kotlin/com/gemwallet/android/features/perpetual/views/components/PerpetualPositionItem.kt). Title differs: `symbolText` vs `asset.symbol.ifEmpty { name }`.
- **R2** **S** Perpetual market — [iOS](../ios/Features/Perpetuals/Sources/ViewModels/PerpetualItemViewModel.swift), [Android](../android/features/perpetual/presents/src/main/kotlin/com/gemwallet/android/features/perpetual/views/components/PerpetualItem.kt). Android hides the price subtitle at null or zero, iOS does not.
- **R3** **S** In-app notification — [iOS](../ios/Features/InAppNotifications/Sources/ViewModels/InAppNotificationListItemViewModel.swift), [Android](../android/features/settings/in_app_notifications/presents/src/main/kotlin/com/gemwallet/android/features/settings/in_app_notifications/presents/components/NotificationItem.kt). Value is a subtitle vs a trailing; unread keys off `isRead` vs `readAt == null`; the 🎁 💎 🎉 ⚠️ table is written twice.
- **R4** **S** Contact — [iOS](../ios/Features/Contacts/Sources/ViewModels/ContactsViewModel.swift), [Android](../android/features/settings/contacts/presents/src/main/kotlin/com/gemwallet/android/features/settings/contacts/presents/ContactsNavScreen.kt). A blank description reserves a line on iOS only.
- **R5** **S** Contact address — [iOS](../ios/Features/Contacts/Sources/ViewModels/ManageContactViewModel.swift), [Android](../android/features/settings/contacts/presents/src/main/kotlin/com/gemwallet/android/features/settings/contacts/presents/ManageContactScene.kt). Both drop the memo independently.
- **R6** **S** Curated asset list — [iOS](../ios/Features/WalletTab/Sources/ViewModels/AssetListItemViewModel.swift), [Android](../android/features/assets/presents/src/main/kotlin/com/gemwallet/android/features/assets/views/WalletSearchScreen.kt). Count is a subtitle vs a trailing.
- **R7** **S** WalletConnect connection — [iOS](../ios/Features/WalletConnector/Sources/WalletConnector/ViewModels/WalletConnectionViewModel.swift), [Android](../android/features/bridge/presents/src/main/kotlin/com/gemwallet/android/features/bridge/views/ConnectionsScene.kt). Host hidden when empty on iOS only; `shortName` derived twice.
- **R9** **S** Service status endpoint — [iOS](../ios/Features/Settings/Sources/ChainSettings/ViewModels/ServiceStatusItemViewModel.swift), [Android](../android/features/settings/networks/presents/src/main/kotlin/com/gemwallet/android/features/settings/networks/presents/ServiceStatusItem.kt). The same rule byte for byte.
- **R10** **S** Rewards redemption option — [iOS](../ios/Features/Settings/Sources/Settings/ViewModels/RewardRedemptionOptionViewModel.swift), [Android](../android/features/referral/presents/src/main/kotlin/com/gemwallet/android/features/referral/views/components/ReferralInfo.kt). Gem emoji and confirmation copy composed twice.
- **R11** **M** NFT grid and list — iOS [grid](../ios/Features/NFT/Sources/Types/NFTGridPosterBuilder.swift) and [list](../ios/Features/NFT/Sources/Views/CollectionsPreviewView.swift), Android [`NftItemUIModel`](../android/ui-models/src/main/kotlin/com/gemwallet/android/ui/models/NftItemUIModel.kt), [`NftListItem`](../android/ui/src/main/kotlin/com/gemwallet/android/ui/components/list_item/NftListItem.kt), [`NFTItem`](../android/features/nft/presents/src/main/kotlin/com/gemwallet/android/features/nft/presents/components/NFTItem.kt). Count placement differs; the verified badge is dropped from both list variants.
- **R12** **M** Address and recipient destination property — [iOS](../ios/Packages/PrimitivesComponents/Sources/ViewModels/AddressListItemViewModel.swift), [Android](../android/features/confirm/presents/src/main/kotlin/com/gemwallet/android/features/confirm/presents/components/PropertyDestination.kt). iOS resolves name-or-domain-or-address three ways, Android does `domain ?: formatted`. Touches confirm, activity details and NFT details.

Rejected: transaction, transaction detail, delegation, validator, asset select/search, wallet, price alert, fiat quote, currency, fee rate, simulation warning, asset market, collectible detail and banner rows already have a record; network list, recents chips, earn APR, swap detail, price list and onboarding rows carry no choice; swap provider rows are iOS only.

## 2. Sections, actions, destinations and limits

Copy: [`GemPerpetualMarketCounts::sections`](../core/gemstone/src/services/perpetual/model.rs) → [iOS](../ios/Features/Perpetuals/Sources/ViewModels/PerpetualsSceneViewModel.swift), [Android](../android/features/perpetual/presents/src/main/kotlin/com/gemwallet/android/features/perpetual/views/market/PerpetualMarketScene.kt).

- **C1** **S** Wallet search limits — [`wallet_search_limits`](../core/gemstone/src/services/assets/rules.rs) returns counts, then [iOS](../ios/Features/WalletTab/Sources/ViewModels/WalletSearchSceneViewModel.swift) checks "has more" against the whole result and [Android](../android/features/assets/viewmodels/src/main/kotlin/com/gemwallet/android/features/assets/viewmodels/WalletSearchViewModel.kt) against pinned + unpinned. Return the preview and the decision together.
- **C2** **S** Network assets sections — [iOS](../ios/Features/WalletTab/Sources/ViewModels/NetworkAssetsSceneViewModel.swift), [Android](../android/features/assets/viewmodels/src/main/kotlin/com/gemwallet/android/features/assets/viewmodels/NetworkAssetsViewModel.kt). The native-asset exclusion and the emptiness booleans are written twice.
- **C3** **S** Select asset sections — [iOS `AssetsSection`](../ios/Packages/PrimitivesComponents/Sources/Types/AssetsSection.swift) excludes popular in the model, Android does it in the scene.
- **C4** **S** Recents sections — [iOS](../ios/Features/Recents/Sources/ViewModels/RecentsSceneViewModel.swift), [Android](../android/features/asset_select/viewmodels/src/main/kotlin/com/gemwallet/android/features/asset_select/viewmodels/RecentsSheetViewModel.kt). Both build an id set, re-filter, then compute their own empty and clear booleans.
- **C5** **M** Audit remaining screens for action lists assembled app-side; `GemStakeActionItem`, `GemHeaderButtonKind`, `GemAssetAction`, `GemFiatButtonAction` are the shape.

## 3. Screens get a session

Copy: [`fiat/session.rs`](../core/gemstone/src/services/fiat/session.rs) → [iOS](../ios/Features/FiatConnect/Sources/ViewModels/FiatSceneViewModel.swift), [Android](../android/features/buy/viewmodels/src/main/kotlin/com/gemwallet/android/features/buy/viewmodels/FiatViewModel.kt).

- **S1** **S** Add node — [iOS](../ios/Features/Settings/Sources/ChainSettings/ViewModels/AddNodeSceneViewModel.swift), [Android](../android/features/settings/networks/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/networks/viewmodels/AddNodeViewModel.kt). Drifted, take first: Android maps three `GemAddNodeException` cases, iOS shows one generic error.
- **S2** **S** Chart period and candles — [iOS](../ios/Features/MarketInsight/Sources/ViewModels/ChartSceneViewModel.swift), [Android](../android/features/asset/viewmodels/src/main/kotlin/com/gemwallet/android/features/asset/viewmodels/chart/viewmodels/ChartViewModel.kt).
- **S3** **M** Perpetual autoclose — [iOS](../ios/Features/Perpetuals/Sources/ViewModels/AutocloseSceneViewModel.swift), [Android](../android/features/perpetual/viewmodels/src/main/kotlin/com/gemwallet/android/features/perpetual/viewmodels/AutocloseViewModel.kt). The confirm-button timing is a [deliberate divergence](SERVICES.md#deliberate-divergences--do-not-fix-these); the session must express both.
- **S4** **M** Set price alert — [iOS](../ios/Features/PriceAlerts/Sources/ViewModels/SetPriceAlertViewModel.swift), [Android](../android/features/settings/price_alerts/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/price_alerts/viewmodels/PriceAlertTargetViewModel.kt).
- **S5** **M** Add asset — [iOS](../ios/Features/Assets/Sources/ViewModels/AddAssetSceneViewModel.swift), [Android](../android/features/add_asset/viewmodels/src/main/kotlin/com/gemwallet/android/features/add_asset/viewmodels/AddAssetViewModel.kt).
- **S6** **M** Wallet search — [iOS](../ios/Features/WalletTab/Sources/ViewModels/WalletSearchSceneViewModel.swift), [Android](../android/features/assets/viewmodels/src/main/kotlin/com/gemwallet/android/features/assets/viewmodels/WalletSearchViewModel.kt). Take the limits item in § 2 first.
- **S7** **M** Import wallet — [iOS](../ios/Features/Onboarding/Sources/ViewModels/ImportWalletSceneViewModel.swift), [Android](../android/features/import_wallet/viewmodels/src/main/kotlin/com/gemwallet/android/features/import_wallet/viewmodels/ImportViewModel.kt).
- **S8** **M** Transactions list and filter — [iOS](../ios/Features/Transactions/Sources/ViewModels/TransactionsViewModel.swift), [Android](../android/features/activities/viewmodels/src/main/kotlin/com/gemwallet/android/features/activities/viewmodels/TransactionsViewModel.kt).
- **S9** **M** Rewards and referral — [iOS](../ios/Features/Settings/Sources/Settings/ViewModels/RewardsViewModel.swift), [Android](../android/features/referral/viewmodels/src/main/kotlin/com/gemwallet/android/features/referral/viewmodels/ReferralViewModel.kt).
- **S10** **M** Chain settings — [iOS](../ios/Features/Settings/Sources/ChainSettings/ViewModels/ChainSettingsSceneViewModel.swift), [Android](../android/features/settings/networks/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/networks/viewmodels/NetworksViewModel.kt). Take the node row in § 1 first.
- **S11** **M** Recipient — [iOS](../ios/Features/Transfer/Sources/ViewModels/RecipientSceneViewModel.swift), [Android](../android/features/recipient/viewmodels/src/main/kotlin/com/gemwallet/android/features/recipient/viewmodel/RecipientViewModel.kt).
- **S12** **L** Select asset — [iOS](../ios/Features/Assets/Sources/ViewModels/SelectAssetViewModel.swift), [Android](../android/features/asset_select/viewmodels/src/main/kotlin/com/gemwallet/android/features/asset_select/viewmodels/BaseAssetSelectViewModel.kt) and its four subclasses. Take the sections item in § 2 first.
- **S13** **L** Manage contact — [iOS](../ios/Features/Contacts/Sources/ViewModels/ManageContactViewModel.swift), [Android](../android/features/settings/contacts/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/contacts/viewmodels/ManageContactViewModel.kt). Take the contact rows in § 1 first.
- **S14** **L** Amount — [iOS](../ios/Features/Transfer/Sources/ViewModels/AmountSceneViewModel.swift) and its four providers, [Android](../android/features/transfer_amount/viewmodels/src/main/kotlin/com/gemwallet/android/features/transfer_amount/viewmodels/AmountViewModel.kt) and its providers. [`services/amount`](../core/gemstone/src/services/amount/) has the rules; the sequencing, button state and error mapping are duplicated.

Rejected: confirm, swap and fiat already hold a session; wallet home already has a `view_state`; asset details, delegation, stake, earn, receive, NFT details, collections, contacts list, transaction details, wallets list and currency are read-only or single-selection; perpetual market has no shared derived set; support chat's only duplication is day grouping; security and developer are platform preferences; create-wallet and phrase verification are not symmetric.

## 4. Views stop naming Core types

Copy: [`FiatScene.swift`](../ios/Features/FiatConnect/Sources/Scenes/FiatScene.swift) names no Core type. Most of this is absorbed by § 1 to § 3.

- **B1** **M** iOS, 11 files. [`LockScreenScene`](../ios/Features/LockManager/Sources/Scenes/LockScreenScene.swift) constructs `GemSecurityService()` in the view; [`RecipientScene`](../ios/Features/Transfer/Sources/Scenes/RecipientScene.swift) takes a `GemRecipient` callback; [`TransactionsFilterScene`](../ios/Features/Transactions/Sources/Scenes/TransactionsFilterScene.swift) takes `SelectionResult<GemTransactionFilter>`; [`ChartScene`](../ios/Features/MarketInsight/Sources/Scenes/ChartScene.swift) and [`CollectibleScene`](../ios/Features/NFT/Sources/Scenes/CollectibleScene.swift) call `socialLinks` directly.
- **B2** **L** Android, 88 files under `presents/`, 31 branching on a Core enum inside the composable. Move the `when` into the view model. `DocsUrl` is read in nine composables; referral renders `Rewards`, `RewardStatus`, `ReferralQuota` directly.
- **B3** **S** Fiat has a session and still leaks: [`FiatUiState`](../android/features/buy/viewmodels/src/main/kotlin/com/gemwallet/android/features/buy/viewmodels/models/FiatUiState.kt) carries `GemFiatQuotePhase` and `GemFiatAmountCheck` into [`FiatNavScreen`](../android/features/buy/presents/src/main/kotlin/com/gemwallet/android/features/buy/views/FiatNavScreen.kt). One resolved `errorText` on the view model.

## 5. Numbers cross as a value and a style

The largest duplication left. Contract: [a number crosses as a value and a style](ARCHITECTURE.md#a-number-crosses-as-a-value-and-a-style-never-as-a-string-or-a-callback). Do F1 first; the rest depend on it.

- **F1** **M** Define `GemPrecision`, `GemNumberStyle` and `GemFormattedNumber` in Core beside [`number_formatter`](../core/crates/number_formatter/src/), and move the adaptive rule with its `0.99` and `1e-10` constants there, with tests.
- **F2** **M** Retire the precision ladder from [`Precision+Constants.swift`](../ios/Packages/Formatters/Sources/Precision+Constants.swift) and [`Precision.kt`](../android/gemcore/src/main/kotlin/com/gemwallet/android/model/Precision.kt); both keep only a renderer that applies a `GemPrecision`.
- **F3** **M** `ValueFormatter` — [iOS](../ios/Packages/Formatters/Sources/ValueFormatter.swift), [Android](../android/gemcore/src/main/kotlin/com/gemwallet/android/model/ValueFormatter.kt). Same abbreviation threshold `100_000`, small-amount `0.1`, dust `0.0001`, `<dust` rendering and Full/Short/Auto ladder on both. Core already has a partial [`value_formatter.rs`](../core/crates/number_formatter/src/value_formatter.rs) with no `Short` style and no uniffi export.
- **F4** **S** `CurrencyFormatter` type-to-precision policy — [iOS](../ios/Packages/Formatters/Sources/CurrencyFormatter.swift), [Android](../android/gemcore/src/main/kotlin/com/gemwallet/android/model/CurrencyFormatter.kt). Three types, `Fiat` pinned to two decimals, compact at `100_000`.
- **F5** **S** `PercentFormatter` — [iOS](../ios/Packages/Formatters/Sources/PercentFormatter.swift), [Android](../android/gemcore/src/main/kotlin/com/gemwallet/android/domains/percentage/PercentageFormatter.kt). Both scale by 1/100, cap at two fraction digits and pick a sign strategy.
- **F6** **M** Carry `GemFormattedNumber` in the row records and view states that carry a bare `f64` today, starting with `GemFiatQuoteRow` and `GemAssetRow`.
- **F7** **S** Then `FiatSceneViewModel` holds no `CurrencyFormatter` or `ValueFormatter`; the session's view state carries the numbers and the scene renders them.

Not in scope: `Formatters` and `Validators` on iOS still cannot import Gemstone, so the renderer that applies a `GemPrecision` must stay dependency-free. That is why this is a value-plus-style contract and not a foreign trait.

## 6. Platform parity

Core exports these and one app calls them. Establish whether it is a missing feature or a duplicated decision first.

- **P1** **S** `selection.customGasPrice()` — iOS [`NetworkFeeSceneViewModel`](../ios/Packages/PrimitivesComponents/Sources/ViewModels/NetworkFeeSceneViewModel.swift); Android decides custom-fee selection its own way in [`FeeDetailsModel`](../android/gemcore/src/main/kotlin/com/gemwallet/android/domains/confirm/FeeDetailsModel.kt).
- **P2** **S** `recommendedValidators` — iOS [`AmountStakeViewModel`](../ios/Features/Transfer/Sources/ViewModels/AmountStakeViewModel.swift); Android picks validators in its amount provider.
- **P3** **S** `applyBannerAction` — iOS [`AssetSceneViewModel`](../ios/Features/Assets/Sources/ViewModels/AssetSceneViewModel.swift); Android handles banner actions elsewhere.
- **P4** **S** `resetTransactionsTimestamp` — iOS developer screen only.
- **P5** **M** `transfer.prefilledAmount()` — iOS [`AmountTransferViewModel`](../ios/Features/Transfer/Sources/ViewModels/AmountTransferViewModel.swift); Android has no prefilled amount from a payment link.
- **P6** **M** `findInvalidWords`, `previewImport` — Android [`ImportViewModel`](../android/features/import_wallet/viewmodels/src/main/kotlin/com/gemwallet/android/features/import_wallet/viewmodels/ImportViewModel.kt); iOS neither highlights invalid mnemonic words nor previews an import.
- **P7** **M** `receiveAccounts` — Android `ReceiveNftChainsViewModel`; iOS has no NFT receive chain picker.

Legitimately one-sided, not gaps: `isVersionHigher` (Play update), `migrateToSharedPassword` (Android password store), `signWithKeystore` (iOS keystore), `isOriginRejected` in the auth flow (Android-only one-click auth; proposal and sign check it inside Core on both), `scanTransaction` (both scan through `GemConfirmService`).

## 7. Everything else

Product or security decisions, one question each:

- **D1** Biometric gate — iOS prompts on every secret read via the [Keychain](../ios/Packages/Keychain/Sources/Keychain.swift) ACL; Android prompts per call site and [`TinkGemPreferences`](../android/app/src/main/kotlin/com/gemwallet/android/data/password/TinkGemPreferences.kt) is unauthenticated, so a new caller bypasses it. Core should mark which operations need authentication.
- **D2** Notification permission — Android's [adapter](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/notifications/NotificationPermissions.kt) holds an application context and cannot tell never-asked from denied. Needs an activity-scoped requester.
- **D3** Privacy lock — iOS has [one](../ios/Features/LockManager/Sources/), Android none. Product call.
- **D4** WalletConnect one-click auth — Android only, rules live in [`WCAuthViewModel`](../android/features/bridge/viewmodels/src/main/kotlin/com/gemwallet/android/features/bridge/viewmodels/WCAuthViewModel.kt). Product call.
- **D5** Polling beside a live socket — Android polls nowhere; adopting Core's interval means new timers. Product call.

Platform items:

- **X1** **L** Android earn flow — no scene, amount params or confirm params. Core exports `sync_earn`, `get_earn_data`, `earn_validators` in [`stake/mod.rs`](../core/gemstone/src/services/stake/mod.rs); earn is currently off in the UI via `EARN_OFFERED` in [`config/stake.rs`](../core/gemstone/src/config/stake.rs).
- **X2** **L** iOS untyped `.map()` naming — Android names the direction. Generator pair in [`remote_mappers.rs`](../core/bin/generate/src/remote_mappers.rs), ~819 call sites.
- **X3** **M** Android dead `NOT NULL` columns in [`GemDatabase`](../android/data/services/store/src/main/kotlin/com/gemwallet/android/data/service/store/database/GemDatabase.kt) — asset `updatedAt`, swap amounts on transaction state, two legacy NFT image columns, price currency. minSdk 28 needs table recreation behind the foreign keys.
- **X4** **S** [`TransactionScene.swift:82`](../ios/Features/Transactions/Sources/Scenes/TransactionScene.swift) — the only `TODO` left in the app, an open iOS 26 styling question.
- **X5** Blocked on install-base data: Android's config-store auth fallback, and the two dated file migrations in [`OnstartService`](../ios/Packages/FeatureServices/AppService/OnstartService.swift) that move the keystore and database to application support.

Do not "fix" the [deliberate divergences](SERVICES.md#deliberate-divergences--do-not-fix-these).
