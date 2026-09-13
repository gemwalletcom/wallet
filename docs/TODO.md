# Open work

Every open item in the repo, ordered so an agent can pick one up and finish it. The shape everything converges on is [ARCHITECTURE.md](ARCHITECTURE.md); how a service is built is [SERVICES.md](SERVICES.md).

**When you finish an item, delete its line here in the same commit.** A finished item leaves no note behind. If the work turns out to be bigger than the line says, rewrite the line before starting rather than leaving it stale.

## How to take an item

One item, one commit, both apps verified before pushing.

1. Read both apps' code for that screen or row and write down the decision each makes. A difference between them is the item's real content.
2. Put the decision in Core with a unit test that fails if the rule flips.
3. Regenerate: `cd core/gemstone && just bindgen-kotlin && just build-android`; `cd ios && just generate-stone`.
4. Delete the app code it replaced on **both** apps, in the same commit. Adding a Core type without removing app code is a layer, not a migration.
5. Clear that screen's [view leak](#4-views-stop-naming-core-types) while you are in it.
6. Verify: `cd core && just lint && cargo test -p gemstone --lib`; `cd ios && just build && just test <Target>`; `cd android && ./gradlew :<module>:testDebugUnitTest`, plus `assembleGoogleDebug` when DI changed.

Sections 0 to 4 are the architecture migration. Section 0 blocks the rest; 1 and 2 are independent of each other; 3 is the most expensive per item; 4 is mostly absorbed by 1 to 3. Section 5 is everything else.

## 0. Make the vocabulary true

Blocking: three shapes answer to the word "session", so there is no reference to copy.

- [`confirm/session.rs`](../core/gemstone/src/services/confirm/session.rs) — `GemConfirmSession` is a `uniffi::Object` with `async load`, `state` and `execute`. It is the confirm screen's service. Rename it so "session" only names the pure record.
- [`swap/session.rs`](../core/gemstone/src/services/swap/session.rs) — `GemSwapSession` answers `quote`, `is_transfer_loading`, `refreshes_quotes` and `accepts_quotes` outside `view_state`. Fold each in, or record why it must be its own crossing.
- Session mocks live nowhere shared; iOS builds `GemSwapSession.mock()` inside [`SwapButtonViewModelTests.swift`](../ios/Features/Swap/Tests/SwapTests/SwapButtonViewModelTests.swift). A session is a value, so its mock belongs in the owning testkit.
- Add a session column to the [screen services table](SERVICES.md#screen-services) so a screen with a service and no session is visible.
- [`fiat/session.rs`](../core/gemstone/src/services/fiat/session.rs) is the reference. Keep it one: nothing added to it needs `await`, a store or the clock.

## 1. Lists get a row record

**Copy this:** [`GemAssetRow`](../core/gemstone/src/services/assets/model.rs) in Core, [`ListAssetItemViewModel.swift`](../ios/Packages/PrimitivesComponents/Sources/ViewModels/ListAssetItemViewModel.swift) on iOS, [`AssetInfoDataAggregate.kt`](../android/gemcore/src/main/kotlin/com/gemwallet/android/domains/asset/aggregates/AssetInfoDataAggregate.kt) on Android. The record is four cases; each app maps a case to a widget and formats nothing in Core.

Twelve lists still decide in app code which field is the title, what the subtitle says, what trails the row or whether a badge shows. Nine already disagree between the apps, so the record is a bug fix. Seven are settings screens. Ordered drifted-and-small first.

- **Perpetual position** — [iOS](../ios/Features/Perpetuals/Sources/ViewModels/PerpetualPositionItemViewModel.swift), [Android](../android/features/perpetual/presents/src/main/kotlin/com/gemwallet/android/features/perpetual/views/components/PerpetualPositionItem.kt). Title is `symbolText` on iOS, `asset.symbol.ifEmpty { name }` on Android.
- **Perpetual market** — [iOS](../ios/Features/Perpetuals/Sources/ViewModels/PerpetualItemViewModel.swift), [Android](../android/features/perpetual/presents/src/main/kotlin/com/gemwallet/android/features/perpetual/views/components/PerpetualItem.kt). Android hides the price subtitle when the price is null or zero; iOS always shows it.
- **In-app notification** — [iOS](../ios/Features/InAppNotifications/Sources/ViewModels/InAppNotificationListItemViewModel.swift), [Android](../android/features/settings/in_app_notifications/presents/src/main/kotlin/com/gemwallet/android/features/settings/in_app_notifications/presents/components/NotificationItem.kt). Value is a leading subtitle on iOS and a trailing with conditional chevron on Android; unread keys off `isRead` vs `readAt == null`. The 🎁 💎 🎉 ⚠️ table is written out twice and moves with it.
- **Contact** — [iOS](../ios/Features/Contacts/Sources/ViewModels/ContactsViewModel.swift), [Android](../android/features/settings/contacts/presents/src/main/kotlin/com/gemwallet/android/features/settings/contacts/presents/ContactsNavScreen.kt). A blank description reserves a line on iOS, is hidden on Android; iOS models a navigate-vs-select action Android does not.
- **Contact address** — [iOS](../ios/Features/Contacts/Sources/ViewModels/ManageContactViewModel.swift), [Android](../android/features/settings/contacts/presents/src/main/kotlin/com/gemwallet/android/features/settings/contacts/presents/ManageContactScene.kt). Both drop the memo independently.
- **Curated asset list** (wallet search "Lists") — [iOS](../ios/Features/WalletTab/Sources/ViewModels/AssetListItemViewModel.swift), [Android](../android/features/assets/presents/src/main/kotlin/com/gemwallet/android/features/assets/views/WalletSearchScreen.kt). Count is a subtitle on iOS, a trailing on Android.
- **WalletConnect connection** — [iOS](../ios/Features/WalletConnector/Sources/WalletConnector/ViewModels/WalletConnectionViewModel.swift), [Android](../android/features/bridge/presents/src/main/kotlin/com/gemwallet/android/features/bridge/views/ConnectionsScene.kt). Host hidden when empty on iOS only; `metadata.shortName` is derived separately in each app and moves with it. Home: [`services/wallet_connect`](../core/gemstone/src/services/wallet_connect/).
- **NFT grid and list** — iOS [grid](../ios/Features/NFT/Sources/Types/NFTGridPosterBuilder.swift) and [list](../ios/Features/NFT/Sources/Views/CollectionsPreviewView.swift), Android [`NftItemUIModel`](../android/ui-models/src/main/kotlin/com/gemwallet/android/ui/models/NftItemUIModel.kt), [`NftListItem`](../android/ui/src/main/kotlin/com/gemwallet/android/ui/components/list_item/NftListItem.kt), [`NFTItem`](../android/features/nft/presents/src/main/kotlin/com/gemwallet/android/features/nft/presents/components/NFTItem.kt). Count is a subtitle on iOS and a trailing on Android; the verified badge is dropped from both list variants. [`GemNftItem`](../core/gemstone/src/services/nft/model.rs) already crosses and carries data only.
- **Address and recipient destination property** — [iOS](../ios/Packages/PrimitivesComponents/Sources/ViewModels/AddressListItemViewModel.swift), Android [`PropertyDestination`](../android/features/confirm/presents/src/main/kotlin/com/gemwallet/android/features/confirm/presents/components/PropertyDestination.kt) and `DestinationPropertyItem`. iOS has a three-way name-or-domain-or-address rule; Android does `domain ?: formatted`. Touches confirm, activity details and NFT details on both apps.
- **Node row** (Settings › Networks) — [iOS](../ios/Features/Settings/Sources/ChainSettings/ViewModels/ChainNodeViewModel.swift), [Android](../android/features/settings/networks/presents/src/main/kotlin/com/gemwallet/android/features/settings/networks/presents/NodeItem.kt). `GemNodeSelection` and `GemNodeStatusState` already cross; the branch structure is written twice.
- **Service status endpoint** (Settings › Networks) — [iOS](../ios/Features/Settings/Sources/ChainSettings/ViewModels/ServiceStatusItemViewModel.swift), [Android](../android/features/settings/networks/presents/src/main/kotlin/com/gemwallet/android/features/settings/networks/presents/ServiceStatusItem.kt). The same rule byte for byte.
- **Rewards redemption option** (Settings › Rewards) — [iOS](../ios/Features/Settings/Sources/Settings/ViewModels/RewardRedemptionOptionViewModel.swift), [Android](../android/features/referral/presents/src/main/kotlin/com/gemwallet/android/features/referral/views/components/ReferralInfo.kt). The gem emoji and the confirmation message are composed twice.

Audited and rejected, do not re-audit: transaction, transaction detail, delegation, validator, asset select and search, wallet, price alert, fiat quote, currency, fee rate, simulation warning, asset market, collectible detail and banner rows already have a Core record; network list rows, recent activity chips, earn APR rows, swap detail rows, price list items and onboarding term rows carry no per-row choice; swap provider rows exist on iOS only.

## 2. Sections, actions, destinations and limits

**Copy this:** [`GemPerpetualMarketCounts::sections`](../core/gemstone/src/services/perpetual/model.rs) in Core, [`PerpetualsSceneViewModel.swift`](../ios/Features/Perpetuals/Sources/ViewModels/PerpetualsSceneViewModel.swift) on iOS, [`PerpetualMarketScene.kt`](../android/features/perpetual/presents/src/main/kotlin/com/gemwallet/android/features/perpetual/views/market/PerpetualMarketScene.kt) on Android. Counts in, one record of section booleans out.

The other three shapes in [§ 3](ARCHITECTURE.md#sections-actions-and-destinations-are-records-too). Small each.

- **Wallet search limits answer half a question** — [`assets/rules.rs`](../core/gemstone/src/services/assets/rules.rs) `wallet_search_limits` returns counts, then [iOS](../ios/Features/WalletTab/Sources/ViewModels/WalletSearchSceneViewModel.swift) checks "is there more" against the whole result and [Android](../android/features/assets/viewmodels/src/main/kotlin/com/gemwallet/android/features/assets/viewmodels/WalletSearchViewModel.kt) against pinned plus unpinned. Return the preview and the has-more decision together.
- **Network assets sections** — [iOS](../ios/Features/WalletTab/Sources/ViewModels/NetworkAssetsSceneViewModel.swift) computes active/pinned/unpinned/hidden plus four emptiness booleans, [Android](../android/features/assets/viewmodels/src/main/kotlin/com/gemwallet/android/features/assets/viewmodels/NetworkAssetsViewModel.kt) the same split plus `isEmpty`. The native-asset exclusion is written twice.
- **Select asset sections** — [iOS `AssetsSection`](../ios/Packages/PrimitivesComponents/Sources/Types/AssetsSection.swift) excludes popular from the plain bucket in the model; Android does it in `AssetSelectScene`.
- **Recents sections** — [iOS](../ios/Features/Recents/Sources/ViewModels/RecentsSceneViewModel.swift) and [Android](../android/features/asset_select/viewmodels/src/main/kotlin/com/gemwallet/android/features/asset_select/viewmodels/RecentsSheetViewModel.kt) both call Core matching, build an id set, re-filter, then compute their own empty and clear booleans.
- Audit remaining screens for action lists assembled app-side. `GemStakeActionItem`, `GemHeaderButtonKind`, `GemAssetAction` and `GemFiatButtonAction` are the shape to match.

## 3. Screens get a session

**Copy this:** [`fiat/session.rs`](../core/gemstone/src/services/fiat/session.rs) in Core, [`FiatSceneViewModel.swift`](../ios/Features/FiatConnect/Sources/ViewModels/FiatSceneViewModel.swift) on iOS, [`FiatViewModel.kt`](../android/features/buy/viewmodels/src/main/kotlin/com/gemwallet/android/features/buy/viewmodels/FiatViewModel.kt) on Android. A state record, `on_*` events returning a new session, one `view_state`; the app owns the task, the debounce and the navigation.

Both apps run the same state machine in their own view model. The drifted one first (a bug fix, small), then small to large. Amount last.

- **Add node** — [iOS](../ios/Features/Settings/Sources/ChainSettings/ViewModels/AddNodeSceneViewModel.swift), [Android](../android/features/settings/networks/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/networks/viewmodels/AddNodeViewModel.kt). **Drifted, take first:** Android maps `GemAddNodeException` to invalid-url, invalid-network-id and generic; iOS shows one generic error.
- **Perpetual autoclose** — [iOS](../ios/Features/Perpetuals/Sources/ViewModels/AutocloseSceneViewModel.swift), [Android](../android/features/perpetual/viewmodels/src/main/kotlin/com/gemwallet/android/features/perpetual/viewmodels/AutocloseViewModel.kt). The confirm-button timing difference is a [deliberate divergence](SERVICES.md#deliberate-divergences--do-not-fix-these), so the session must express both: Core owns the outcome, the app owns when errors appear.
- **Chart period and candles** — [iOS](../ios/Features/MarketInsight/Sources/ViewModels/ChartSceneViewModel.swift) and `PerpetualChartModel`, [Android](../android/features/asset/viewmodels/src/main/kotlin/com/gemwallet/android/features/asset/viewmodels/chart/viewmodels/ChartViewModel.kt) and `PerpetualDetailsViewModel`. Period persistence, load phase, resubscribe key.
- **Set price alert** — [iOS](../ios/Features/PriceAlerts/Sources/ViewModels/SetPriceAlertViewModel.swift), [Android](../android/features/settings/price_alerts/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/price_alerts/viewmodels/PriceAlertTargetViewModel.kt). Direction resolution, suggestion lists, confirmation copy, alert assembly.
- **Add asset** — [iOS](../ios/Features/Assets/Sources/ViewModels/AddAssetSceneViewModel.swift), [Android](../android/features/add_asset/viewmodels/src/main/kotlin/com/gemwallet/android/features/add_asset/viewmodels/AddAssetViewModel.kt). Lookup phase, which panel shows, debounce vs immediate.
- **Wallet search** — [iOS](../ios/Features/WalletTab/Sources/ViewModels/WalletSearchSceneViewModel.swift), [Android](../android/features/assets/viewmodels/src/main/kotlin/com/gemwallet/android/features/assets/viewmodels/WalletSearchViewModel.kt). Take the limits item in § 2 first.
- **Import wallet** — [iOS](../ios/Features/Onboarding/Sources/ViewModels/ImportWalletSceneViewModel.swift), [Android](../android/features/import_wallet/viewmodels/src/main/kotlin/com/gemwallet/android/features/import_wallet/viewmodels/ImportViewModel.kt). Import kind tabs, name resolution, view-only warning, new-vs-existing routing.
- **Transactions list and filter** — [iOS](../ios/Features/Transactions/Sources/ViewModels/TransactionsViewModel.swift) and `TransactionsFilterViewModel`, [Android](../android/features/activities/viewmodels/src/main/kotlin/com/gemwallet/android/features/activities/viewmodels/TransactionsViewModel.kt). Filter assembly with the all-types fallback, the filter badge, reset on wallet change.
- **Rewards and referral** — [iOS](../ios/Features/Settings/Sources/Settings/ViewModels/RewardsViewModel.swift), [Android](../android/features/referral/viewmodels/src/main/kotlin/com/gemwallet/android/features/referral/viewmodels/ReferralViewModel.kt). Sync phase, wallet selector visibility, pending-activation copy.
- **Chain settings** — [iOS](../ios/Features/Settings/Sources/ChainSettings/ViewModels/ChainSettingsSceneViewModel.swift), [Android](../android/features/settings/networks/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/networks/viewmodels/NetworksViewModel.kt). Per-node status map, stale-response discard, explorer selection. Take the node row in § 1 first.
- **Recipient** — [iOS](../ios/Features/Transfer/Sources/ViewModels/RecipientSceneViewModel.swift), [Android](../android/features/recipient/viewmodels/src/main/kotlin/com/gemwallet/android/features/recipient/viewmodel/RecipientViewModel.kt). Name-lookup state, memo visibility, section assembly, scan and next routing.
- **Select asset** — [iOS](../ios/Features/Assets/Sources/ViewModels/SelectAssetViewModel.swift) and `AssetsFilterViewModel`, [Android](../android/features/asset_select/viewmodels/src/main/kotlin/com/gemwallet/android/features/asset_select/viewmodels/BaseAssetSelectViewModel.kt) and its four subclasses. Take the sections item in § 2 first.
- **Manage contact** — [iOS](../ios/Features/Contacts/Sources/ViewModels/ManageContactViewModel.swift) and `ManageContactAddressViewModel`, [Android](../android/features/settings/contacts/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/contacts/viewmodels/ManageContactViewModel.kt). Sub-page state, avatar state, address form, save gating. Take the contact rows in § 1 first.
- **Amount** — [iOS](../ios/Features/Transfer/Sources/ViewModels/AmountSceneViewModel.swift) and its transfer/stake/perpetual/earn providers, [Android](../android/features/transfer_amount/viewmodels/src/main/kotlin/com/gemwallet/android/features/transfer_amount/viewmodels/AmountViewModel.kt) and `providers/Amount*Provider.kt`. [`services/amount`](../core/gemstone/src/services/amount/) already has the rules; the sequencing, button state and error mapping are duplicated.

Audited and rejected, do not re-audit: confirm, swap and fiat already hold a session; wallet home already has a Core `view_state`; asset details, delegation details, stake, earn, receive, NFT details, collections, contacts list, transaction details, wallets list and currency settings are read-only or single-selection with their decisions already in Core rules; perpetual market has no shared derived set to lift; support chat's only duplication is day grouping, which is a formatter; security and developer settings are platform preferences; create-wallet and phrase verification are not symmetric.

## 4. Views stop naming Core types

**Copy this:** [`FiatScene.swift`](../ios/Features/FiatConnect/Sources/Scenes/FiatScene.swift) reads only platform values from its model and names no Core type.

Two greps find the set: `Sources/Scenes/` and `Sources/Views/` on iOS, `presents/` on Android. Most is absorbed by § 1 to § 3 — clear a screen's leak in the commit that gives it a row or session. The leftovers:

- **iOS, 11 files.** [`LockScreenScene`](../ios/Features/LockManager/Sources/Scenes/LockScreenScene.swift) constructs `GemSecurityService()` in the view; [`RecipientScene`](../ios/Features/Transfer/Sources/Scenes/RecipientScene.swift) takes a `GemRecipient` callback; [`TransactionsFilterScene`](../ios/Features/Transactions/Sources/Scenes/TransactionsFilterScene.swift) takes `SelectionResult<GemTransactionFilter>`; [`ChartScene`](../ios/Features/MarketInsight/Sources/Scenes/ChartScene.swift) and [`CollectibleScene`](../ios/Features/NFT/Sources/Scenes/CollectibleScene.swift) call `Gemstone.socialLinks` directly.
- **Android, 88 files, 31 branching on a Core enum inside the composable.** The enum branch is the bulk: move the `when` into the view model and expose the resolved string. `DocsUrl` is read in nine composables. The referral feature renders `Rewards`, `RewardStatus` and `ReferralQuota` straight into the UI.
- A session does not clear the leak: fiat has one and still passes `GemFiatQuotePhase` and `GemFiatAmountCheck` through [`FiatUiState`](../android/features/buy/viewmodels/src/main/kotlin/com/gemwallet/android/features/buy/viewmodels/models/FiatUiState.kt) into [`FiatNavScreen`](../android/features/buy/presents/src/main/kotlin/com/gemwallet/android/features/buy/views/FiatNavScreen.kt), which switches on both. One resolved `errorText` on the view model fixes it.

### Rules for every item in § 0 to § 4

- A session is constructed where the screen is: [`ViewModelFactory.swift`](../ios/Gem/Services/ViewModelFactory.swift) on iOS, the screen's Hilt module on Android. Never at file scope.
- Name the pair `Gem<Feature>Session` and `Gem<Feature>ViewState`. No `Scene` in a Gemstone name.
- One session per screen, the same rule as one Core service per screen.
- A row record carries choices, never formatted text — Core's value formatter is not locale-aware.
- Localization stays app-side: move the branch, not the string.

## 5. Everything else

### Decisions someone has to make

Each is one product or security question, none blocked on investigation.

- **Biometric gate.** iOS gates at the Keychain ACL, so every secret read prompts ([`Keychain`](../ios/Packages/Keychain/Sources/Keychain.swift)). Android prompts at each call site and [`TinkGemPreferences`](../android/app/src/main/kotlin/com/gemwallet/android/data/password/TinkGemPreferences.kt) is unauthenticated, so a new caller bypasses the gate. Recommendation: Core marks which operations require authentication ([`services/auth`](../core/gemstone/src/services/auth/)) and the adapter enforces it.
- **Notification permission.** Core owns granted / denied / never-asked, but [`GemstoneNotificationPermissions`](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/notifications/NotificationPermissions.kt) holds an application context and cannot tell never-asked from denied, so it sends a first-time user to Settings. Core keeps the three-state decision; Android needs an activity-scoped requester.
- **Privacy lock.** iOS has an app-lock setting with a cover-screen rule and an overlay window ([`LockManager`](../ios/Features/LockManager/Sources/)); Android has none. Product call — the cover predicate is Core's, the overlay is platform.
- **WalletConnect one-click auth.** Android only, and its rules — including what the user is asked to sign — live in [`WCAuthViewModel`](../android/features/bridge/viewmodels/src/main/kotlin/com/gemwallet/android/features/bridge/viewmodels/WCAuthViewModel.kt). Product call; whoever takes it moves the rules to Core first.
- **Polling beside a live socket.** Android polls nowhere, so adopting Core's refresh interval means adding timers to the activity, asset and perpetual screens — new background work, not a consolidation. Product call.

### Android

- **Earn flow.** No Earn surface exists: no earn provider reader, amount params or confirm params, and Core's deposit delegation action maps to nothing. Core already exports `sync_earn`, `get_earn_data` and `earn_validators` in [`stake/mod.rs`](../core/gemstone/src/services/stake/mod.rs); [`features/earn`](../android/features/earn/) has only `stake` and `delegation`. Note earn is currently switched off in the UI on both apps via `EARN_OFFERED` in [`config/stake.rs`](../core/gemstone/src/config/stake.rs).
- **Dead `NOT NULL` columns** with no iOS counterpart in [`GemDatabase`](../android/data/services/store/src/main/kotlin/com/gemwallet/android/data/service/store/database/GemDatabase.kt): an asset `updatedAt` stamp, swap amounts on transaction state, two legacy NFT image columns, and the price currency column. minSdk 28 has no `ALTER TABLE DROP COLUMN`, so removing them means recreating the tables behind their foreign keys in one migration.
- **Secure auth fallback.** The config-store fallback for the auth value can go once enough installs have written the secure one. An install-base call, not a code call.

### iOS

- **Untyped `.map()` naming.** Android names the direction (`toPrimitives()`); iOS does not. The generator pair is in [`remote_mappers.rs`](../core/bin/generate/src/remote_mappers.rs); roughly 819 call sites follow.
- **Transaction scene corner radius** — [`TransactionScene.swift:82`](../ios/Features/Transactions/Sources/Scenes/TransactionScene.swift) carries the only `TODO` comment left in the app. An open iOS 26 styling question that marks a real gap.
- **Two dated file migrations** in [`OnstartService`](../ios/Packages/FeatureServices/AppService/OnstartService.swift) move the keystore and database from documents to application support at launch. Deleting them strands anyone who has not opened the app since the move, losing their keystore; needs install-base data.

### Deliberate divergences — do not "fix" these

They are listed in [SERVICES.md](SERVICES.md#deliberate-divergences--do-not-fix-these): the 30-second stream reconnect cap, the autoclose confirm-button timing, the iOS copy of typed-number reading (the price widget links the formatter package without the Rust library, and the validators package may not import Gemstone), and a flow parent holding several private Core services.
