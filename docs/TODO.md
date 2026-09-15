# Open work

Every open item carries a stable id (V vocabulary, R rows, C composition, S sessions, B view boundary, F formatting, P parity, D decisions, O ownership, X platform, G guidance) and a size (**S**/**M**/**L**). Contracts are in [ARCHITECTURE.md](ARCHITECTURE.md). **Delete an item's line in the commit that lands it** — ids are never reused.

The goal is that Gemstone decides once and both clients read that decision. An item belongs on this list when the same choice — a label, an order, a row's shape, a number's style, a threshold — is made in two places.

One item, one commit, both apps built and tested. Core rule + test, regenerate bindings, delete the app code it replaced on both sides, clear that screen's view leak while you are in it.

## 1. Lists get a row record

Copy: [`GemAssetRow`](../core/gemstone/src/services/assets/model.rs) → [iOS](../ios/Packages/PrimitivesComponents/Sources/ViewModels/ListAssetItemViewModel.swift), [Android](../android/gemcore/src/main/kotlin/com/gemwallet/android/domains/asset/aggregates/AssetInfoDataAggregate.kt).


- **R16** **S** `GemSwapProgressStep` is switched on inside the view on both apps to pick a marker glyph and a tone — [iOS](../ios/Features/Transactions/Sources/Views/TransactionSwapProgressView.swift) plus [its extension](../ios/Features/Transactions/Sources/ItemModels/TransactionSwapProgressItemModel.swift), [Android](../android/features/activities/presents/src/main/kotlin/com/gemwallet/android/features/activities/presents/details/components/SwapProgressItem.kt) — and the two have drifted: `refunded` reads as an arrow-swap in orange on iOS and as a red close icon on Android, and `waiting` is an ellipsis glyph against three hand-drawn dots. Copy [`GemTransactionStateTone`](../core/gemstone/src/services/transactions/model.rs): the step answers a tone and a marker kind, each app maps those to its own palette and icon set. Live divergence, not drift prevention.

- **R17** **L** Settings has no Core service at all. Both apps show the same nine rows in the same order — wallets, security, notifications, preferences, WalletConnect, support, rewards, about, developer — so there is no drift today; the row set, its order, the icons and the visibility conditions are simply written twice — [iOS `SettingsViewModel`](../ios/Features/Settings/Sources/Settings/ViewModels/SettingsViewModel.swift) as `xTitle`/`xImage` pairs, [Android `SettingsScene`](../android/features/settings/settings/presents/src/main/kotlin/com/gemwallet/android/features/settings/settings/presents/views/SettingsScene.kt) inline in the composable. Add a `settings` service with a row record carrying the key, the icon key and the destination, and let each app map the key to its own image and string. `PreferencesViewModel`, `SecurityViewModel` and `AboutUsViewModel` are the same screen family and go with it.
- **R18** **S** The NFT collection screen takes its title as a navigation string: [`CollectionViewModel`](../ios/Features/NFT/Sources/ViewModels/CollectionViewModel.swift) stores `collectionName` passed in by the caller rather than reading it from the collection the screen already loads. The name belongs to the collection record; a navigation value carries the id.
- **R19** **M** WalletConnect connections — [`ConnectionsViewModel`](../ios/Features/WalletConnector/Sources/WalletConnector/ViewModels/ConnectionsViewModel.swift) and `ConnectionSceneViewModel` hold no Core record and build the row and the detail fields themselves; Android's bridge screens do the same. `GemConnectionRow` already exists and answers part of it.
- **R20** **M** Chain settings and nodes — `ChainSettingsSceneViewModel`, `ChainNodeViewModel` and `ServiceStatusItemViewModel` on iOS against the Android networks screens. `GemNodeSelection` and `GemServiceEndpoint` cross, but the section titles, the node subtitle and the explorer row are each app's.
- **R21** **M** Stake and delegation — `StakeSceneViewModel`, `DelegationSceneViewModel` and `DelegationViewModel` build nine section and field titles app-side against the Android earn screens.
- **R22** **S** Contacts list and support chat rows carry no Core record on either app (`ContactsViewModel`, `SupportChatSceneViewModel` and their Android counterparts).
- **R23** **S** In-app notifications and the fiat transaction list build their rows app-side on both apps.

- **R24** **L** The shared row models in [`PrimitivesComponents`](../ios/Packages/PrimitivesComponents/Sources/ViewModels) are the largest unmigrated group: 29 models, 57 user-facing strings, none holding a Core record — `AssetViewModel`, `AddressListItemViewModel`, `AssetDataViewModel`, `NetworkSelectorViewModel`, `WalletHeaderViewModel`, `MarketValueViewModel`. Android mirrors each in `ui-models` and `gemcore/domains`. Take them one row at a time; `GemAssetRow` is the exemplar and several already have a Core record they do not hold.
- **R25** **M** Perpetuals — `PerpetualSceneViewModel` alone decides nine section and button titles, with `AutocloseViewModel`, `PerpetualsHeaderViewModel` and `ChartLineViewModel` behind it, against the Android perpetual screens.
- **R26** **M** Onboarding — ten models across import, setup, phrase verification and secret display decide their own titles and footer text against the Android `import_wallet` and `create_wallet` screens.
- **R27** **M** Transfer — `AmountSceneViewModel`, `ReceiveViewModel` and `AmountPerpetualViewModel` against the Android `transfer_amount` and `receive` screens.
- **R28** **S** Transaction filters — `TransactionTypesSelectorViewModel` and `TransactionsFilterViewModel` decide the selector titles on both apps.
- **R29** **S** Add asset — `AddAssetViewModel` decides five field titles against the Android `add_asset` screen.
- **R30** **S** Swap details — `SwapProvidersViewModel` and `PriceImpactViewModel` decide the provider and impact titles on both apps.
- **R31** **S** Market value and QR scanner errors — `MarketValueViewModel` (5 strings) and `QRScannerErrorViewModel` against their Android counterparts.

Rejected: transaction, transaction detail, delegation, validator, asset select/search, wallet, price alert, fiat quote, currency, fee rate, simulation warning, asset market, collectible detail and banner rows already have a record; network list, recents chips, earn APR, swap detail, price list and onboarding rows carry no choice; swap provider rows and the QR scan-type hint table are iOS only; swap price impact already crosses as `impactType`/`isHigh`/`showsInSummary`; the delegation completion countdown is computed twice but belongs to the delegation record if anywhere.

- **R32** **M** Row models that store a domain object beside the Core row, against [§ 3](ARCHITECTURE.md#a-row-model-conforms-to-a-ui-protocol-it-stores-nothing-but-the-row). iOS: `ListAssetItemViewModel` holds `GemAssetRow` and an `AssetDataViewModel`, `InAppNotificationListItemViewModel` a `GemNotificationRow` and a `CoreListItem`, `ServiceStatusItemViewModel` two Core records and a loose `name`. Android: `TransactionDetailsAggregateImpl` and `TransactionDataAggregateImpl` keep a `TransactionExtended`, `WalletDetailsAggregateImpl` a `Wallet`, `BuyFiatProviderUIModel` an asset and a currency. Each pair answers the same question twice; move the field Core is missing onto its row and drop the second object.
- **R33** **M** Row models with no Core row at all: iOS `PerpetualItemViewModel`, `PerpetualPositionItemViewModel`, `OpenPositionItemViewModel` and `AssetListItemViewModel` decide their own name, symbol, image, subtitle and right view. Give each the row its screen needs, in the shape of `GemPriceAlertRow`.
- **R34** **L** `ListAssetItemViewable` itself is a parity question: its `subtitleView` and `rightView` are a fixed set of shapes (price, balance, toggle, copy, none) that both apps re-derive per screen. Decide whether Core names the shape — a `GemListItemSubtitle`/`GemListItemAccessory` enum every row carries — so a row's layout is chosen once. Weigh it against [§ 3 Keep the crossings few](ARCHITECTURE.md#keep-the-crossings-few) before starting; this is the largest of the row items and should follow R32 and R33.


- **O12** **M** `WalletId` and `Chain` still cross as bare strings, so every call unwraps one by hand on both platforms. `WalletId` is a struct in the apps and a `String` across the FFI — that is why [`GemWalletService.swift`](../ios/Packages/GemstonePrimitives/Sources/Services/GemWalletService.swift) is sixty lines of `wallet.id.id` and `.toPrimitives()`, a bridge with no decisions in it. `Chain` is declared under `codes` in [`remote_types.yml`](../core/bin/generate/remote_types.yml) and crosses the same way, so `chain.rawValue` and `Chain(core:)` litter both apps and every new Core parameter tempts the next caller to type it `String`. Make them cross as the records the apps already hold, the way `AssetId` and `Currency` do, and the bridges go with them. Check first what persists a wallet id as a raw string.
- **R35** **S** The chart header's colours are decided twice and the two apps disagree. iOS colours the headline from the value only when the type is `PriceChange` and always colours the change from the percentage; Android derives one direction — the value for `PriceChange`, the percentage otherwise — and applies it to the whole `PriceInfo`. Both read the same two numbers and reach different screens. Decide which is right, then let the row carry the tone so neither app derives it.

## 2. Sections, actions, destinations and limits

Per-variant labels: a primitives enum both apps map to a string themselves is a decision written twice. Both languages force a `switch`/`when` over a Core enum to be exhaustive, so these sets cannot silently drift — every one checked below maps the same variants to the same meaning. That makes the V series maintenance cost and a place for drift to start, not a live bug; the exception is a catch-all branch, which **X20** covers. The migrated shape is a Core text key each app resolves once in its own `Gemstone+Localized.swift` / `GemstoneText.kt` — `GemTransactionTitle`, `GemBannerTitle` and `GemWalletSubtitle` already work that way. These do not:

- **V3** **S** `Appearance` — [iOS](../ios/Features/Settings/Sources/Settings/Types/Appearance+Title.swift), Android inline in `PreferencesScene`.
- **V4** **S** `ChartPeriod` — 12 cases across [iOS](../ios/Packages/PrimitivesComponents/Sources/Extensions/ChartPeriod+PrimitivesComponents.swift) and Android `PeriodsPanel`.
- **V5** **S** `LinkType` — 14 cases, checked case by case: both apps map the same set to the same labels, so this is duplication with no drift. [iOS `AssetLinkViewModel`](../ios/Packages/PrimitivesComponents/Sources/ViewModels/AssetLinkViewModel.swift), Android `SocialLink`.
- **V7** **S** `PerpetualMarginType` — cross vs isolated, four cases.
- **V8** **S** `Resource` — Tron bandwidth and energy, four cases.
- **V9** **S** `ReportReason` — [iOS `ReportReasonViewModel`](../ios/Features/NFT/Sources/ViewModels/ReportReasonViewModel.swift) and Android `NftDetailsScene.titleRes` map the same five reasons, and the order comes from each language's enum declaration rather than from Core.
- **V10** **S** QR scan type — 8 cases, [iOS](../ios/Features/QRScanner/Sources/ViewModels/QRScannerSceneViewModel.swift), Android `QRScanner.kt`.
- **V11** **S** Simulation payload field kind — 7 cases, `SimulationPayloadFieldViewModel` against `SimulationPayloadFieldsContent.kt`.
- **V12** **S** Portfolio statistic — 7 cases, `PortfolioSceneViewModel` against `PortfolioStatistics.kt`.
- **V13** **S** Transaction row subtitle kind — 6 cases, `TransactionViewModel` against `TransactionDataAggregateExt.kt`.
- **V14** **S** Transaction participant role — 6 cases, `TransactionParticipantViewModel` against `DestinationPropertyItem.kt`.
- **V15** **S** Transaction state title — 6 cases, `TransactionStateViewModel` against `TransactionStateExt.kt`. Core answers the tone already; the title is still each app's.
- **V16** **S** Price alert kind — 5 cases, `PriceAlertItemViewModel` against `PriceAlertListItem.kt`. Lands with **F12**.
- **V17** **S** Perpetual position action — 5 cases, `ToastMessage+PrimitivesComponents` against `PerpetualConfirmDetailsComponents.kt`.
- **V18** **S** Swap button action — 5 cases, `SwapButtonViewModel` against `SwapUiState.kt`.
- **V19** **S** Asset context menu action — 4 cases, `AssetContextMenu` on both apps.
- **V20** **S** Collectible row kind — 4 cases, `CollectibleViewModel` against `NftDetailsScene.kt`.
- **V21** **S** Chart scene section — 4 cases, `ChartScene` against `AssetChartScene.kt`.
- **V22** **S** `GemFiatAmountCheck` — a Core enum both apps map to text themselves, `FiatSceneViewModel` against `FiatViewModel.kt`. Lands with **F9**'s shape.
- **V23** **S** Verification status — 3 cases, mapped twice on iOS (`VerificationStatusViewModel` and `ConnectionProposalViewModel`) and once on Android.
- **V24** **S** Connection status — 3 cases, `ConnectionStatusViewModel` against `MainActivity.kt`.
- **V25** **S** Fee unit type — gwei, native, sat/vB, `FeeUnitViewModel` against `FeeDetails.kt`.
- **V26** **S** Price alert direction — up, down, none, `SetPriceAlertViewModel` against `PriceAlertTargetNavScreen.kt`.
- **V27** **S** Perpetual direction — long and short, `PerpetualDirectionViewModel` against `PerpetualDirectionValue.kt`.
- **V28** **S** Fiat quote type — buy and sell, mapped in `FiatTransactionViewModel` and `FiatSceneViewModel` against `FiatNavScreen.kt`.
- **V29** **S** Fee priority — fast and normal, `FeeRateViewModel` against `FeePriorityExt.kt`.
- **V30** **S** Approval value — exact and unlimited, `AssetValueHeaderViewModel` against `AssetValueListHead.kt`.
- **V32** **L** Info sheets, and this is a product gap before it is a duplication: iOS `InfoSheetType` has 30 cases, Android's [`InfoSheetEntity`](../android/ui/src/main/kotlin/com/gemwallet/android/ui/components/InfoBottomSheet.kt) has 14. Every Android sheet exists on iOS, so nothing is Android-only; **sixteen explanations are iOS-only** — `assetStatus`, `autoclose`, `circulatingSupply`, `fullyDilutedValuation`, `fundingApr`, `fundingPayments`, `liquidationPrice`, `maliciousTransaction`, `maxSupply`, `memoRequired`, `minimumAmount`, `noQuote`, `openInterest`, `priceImpact`, `slippage`, `totalSupply` and `watchWallet`. `maliciousTransaction` is a security explanation an Android user never sees. Decide the set in Core, then each app renders it.
- **V33** **M** Empty states — [iOS `EmptyContentTypeViewModel`](../ios/Packages/PrimitivesComponents/Sources/ViewModels/EmptyContentTypeViewModel.swift) against Android `EmptyStateView` and `EmptyContentView`: the title, the description and the action for every empty list are decided twice.
- **V34** **M** The confirm screen's row set — `ConfirmTransferScene.itemModel` maps 15 cases on iOS against the Android confirm screen.
- **V35** **S** Select-asset presentation — 11 cases of title and empty text on [iOS](../ios/Features/Assets/Sources/Types/SelectAssetPresentation.swift) against the Android select screens.
- **V36** **S** Asset details info rows — 9 cases in `AssetDetailsInfoViewModel` against the Android asset info section.
- **V37** **S** iOS keeps a local `HeaderButtonType` with 9 label cases while `GemHeaderButtonKind` already crosses and both apps use it; the local enum is the leftover.
- **V38** **M** Error descriptions — [`Gem/Types/Errors.swift`](../ios/Gem/Types/Errors.swift) maps seven separate Core error enums to text, against Android's `Throwable.serviceMessage()` and its per-module `GemstoneText.kt`. Check each enum: some already have a Core text key and some do not.
- **V39** **S** Stake amount action — 7 cases in `AmountStakeViewModel` against the Android amount providers.

- **V31** **S** Scan/receive mode, portfolio type (wallet and perpetuals), autoclose type (take profit and stop loss), wallet source (create and import), wallet secret kind (phrase and private key), fiat button action and the swap select side (pay and receive) — seven two-case maps, each duplicated, each its own commit.


Copy: [`GemPerpetualMarketCounts::sections`](../core/gemstone/src/services/perpetual/model.rs) → [iOS](../ios/Features/Perpetuals/Sources/ViewModels/PerpetualsSceneViewModel.swift), [Android](../android/features/perpetual/presents/src/main/kotlin/com/gemwallet/android/features/perpetual/views/market/PerpetualMarketScene.kt).


## 3. Screens get a session

Copy: [`fiat/session.rs`](../core/gemstone/src/services/fiat/session.rs) → [iOS](../ios/Features/FiatConnect/Sources/ViewModels/FiatSceneViewModel.swift), [Android](../android/features/buy/viewmodels/src/main/kotlin/com/gemwallet/android/features/buy/viewmodels/FiatViewModel.kt). A smaller one to copy first: [`chart/session.rs`](../core/gemstone/src/services/chart/session.rs) → [iOS](../ios/Features/MarketInsight/Sources/ViewModels/ChartSceneViewModel.swift), [Android](../android/features/asset/viewmodels/src/main/kotlin/com/gemwallet/android/features/asset/viewmodels/chart/viewmodels/ChartViewModel.kt).

A session is a plain Record: the screen's service vends it, `on_*` events return a new one, and `view_state()` derives what the screen shows. It performs no I/O — the app awaits its own service and hands the result back through an event. Do not let the flow that collects a session also write to it; drive loads from the period or trigger that changed.


Rejected: confirm, swap and fiat already hold a session; wallet home already has a `view_state`; asset details, delegation, stake, earn, receive, NFT details, collections, contacts list, transaction details, wallets list and currency are read-only or single-selection; perpetual market has no shared derived set; support chat's only duplication is day grouping (C6) and platform image encoding; security and developer are platform preferences; create-wallet and phrase verification are not symmetric; the perpetuals preview gates on balance on iOS and on user config on Android, which is a § 6 question, not a session.

## 4. Views stop naming Core types

Copy: [`FiatScene.swift`](../ios/Features/FiatConnect/Sources/Scenes/FiatScene.swift) names no Core type. Most of this is absorbed by § 1 to § 3.


## 5. Numbers cross as a value and a style

Contract: [a number crosses as a value and a style](ARCHITECTURE.md#a-number-crosses-as-a-value-and-a-style-never-as-a-string-or-a-callback). The precision rules, the value ladder, the abbreviation threshold and the dust cut are Core's, and `GemFormattedNumber` carries a number with its resolved display. What is left is the records that still hand the apps a bare `f64` and let each pick a style. Copy [`GemFiatQuoteRow`](../core/gemstone/src/services/fiat/model.rs) and [`GemAmountError::display`](../core/gemstone/src/services/amount/model.rs).



Not in scope: counts a screen uses to build sections (`GemWalletSearchCounts`, `GemPerpetualMarketCounts`, `GemNetworkAssetCounts`, `GemRecentsCounts`), decimals, bps, indices, timeouts and chart geometry stay numbers — the contract is about numbers the app renders as text.

Not in scope: `Formatters` and `Validators` on iOS still cannot import Gemstone, so the renderer that applies a `GemPrecision` must stay dependency-free. That is why this is a value-plus-style contract and not a foreign trait.

## 6. Platform parity

Core exports these and one app calls them. Establish whether it is a missing feature or a duplicated decision first. Exports that belong to a screen already listed above are noted on that item (V2, S3, S5, S9, S16, F8).


Legitimately one-sided, not gaps: `isVersionHigher` (Play update), `migrateToSharedPassword` (Android password store), `set_price_alerts_enabled` (Android one-off migration), `signWithKeystore` (iOS keystore), `isOriginRejected`, `authentication_chain_ids`, `authentication_accounts`, `authentication_methods` in the auth flow (Android-only one-click auth, D4; proposal and sign check the origin inside Core on both), `scanTransaction` (both scan through `GemConfirmService`).

## 7. Everything else

Product or security decisions, one question each:


Ownership, injection and threads:

- **O10** **S** [`ImportWalletViewModel`](../ios/Features/Onboarding/Sources/ViewModels/ImportWalletViewModel.swift) is down to the wallet and name services after the avatar moved behind `GemWalletService`. The name service is the last one it only passes down, to `ImportWalletSceneViewModel` for recipient name resolution. Decide whether resolving a name while importing is a wallet-service answer — if it is, the parent holds one service and the threading is gone.
- **O9** **M** Six Android view models hold two or three Core services where iOS composes one screen service: [`TransactionsViewModel`](../android/features/activities/viewmodels/src/main/kotlin/com/gemwallet/android/features/activities/viewmodels/TransactionsViewModel.kt), `RecentsSheetViewModel`, `WCAuthViewModel` (three), `WCRequestViewModel`, `PerpetualMarketViewModel` and `SettingsViewModel`. Copy the screen-service shape in [SERVICES.md](SERVICES.md#the-screen-service-map); a launch host or a flow parent vending child models is [allowed to hold several](ARCHITECTURE.md#7-at-most-one-core-service-on-ios-narrow-cases-on-android) and these are neither.



Platform items:


- **X19** **S** Dead code, second pass. Removed: the Android `Precision` leftovers, iOS `String.numberOfOccurrencesOf` and nine unread `Locale` identifiers. Kept, with reasons a future sweep must respect — `recoverPubKey` satisfies reown's `CryptoProvider`, `deviceInfo` and `signTransaction` are `GemDevicePlatform` and `GemWalletConnectSigner` foreign-trait callbacks Rust invokes, `AuthenticationPolicy` mirrors Apple's `SecAccessControlCreateFlags` as a whole, and the three `Localized` keys no app reads are translated entries in the shared Fluent source that other clients draw from. What is left is only that last group, which needs the translation workflow's agreement, not a code change.






Guides:


Do not "fix" the [deliberate divergences](SERVICES.md#deliberate-divergences--do-not-fix-these).
