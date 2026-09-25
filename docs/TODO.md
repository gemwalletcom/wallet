# Open work

The goal is that Gemstone decides once and both clients read that decision. Shared rules, records, sessions and orchestration live in Core; rendering, observation, scheduling, OS ports and locale formatting live in the apps. A view model, UI model, aggregate, factory or provider that still decides, re-derives, composes text or formats a number is debt; a model that holds one Core record and maps it to platform values is the target shape. Contracts and worked examples are in [ARCHITECTURE.md](ARCHITECTURE.md).

Every open item carries a stable id and a size (**S**/**M**/**L**); ids are never reused. Namespaces: `VM` (consolidation into Gemstone), `AUD` (correctness found by review), `BD` (a behaviour difference), `D` (a decided product or security choice being built) and `X` (blocked upstream).

- **Closing an item:** delete its line and commit at once. If it settles an important user-facing rule someone might simplify away, write that rule on its area page under [product/](product/) in the same commit, following [PRODUCT.md § Writing a page](PRODUCT.md#writing-a-page); most items have none. Nothing else is recorded here: git history carries the detail.
- **Comparing the apps:** say what each does on its own line — **iOS:**, **Android:**, then **Expected:** (the agreed behaviour, or "needs a decision") — never in one sentence covering both. When the apps differ and no item decides otherwise, iOS is the reference behaviour.

## Working one item

Pick one item by its stable id and use the screen map to find its current owner. Source paths and line numbers are retrieval hints, not proof that an old finding still applies. Read the owner, both consumers and the linked architecture example before editing.

Keep the implementation plan in the task update: the existing owner to change, the app path to delete, any real prerequisite and the exact relevant checks. Do not create a new planning artifact, service, session, cache or framework unless the current contract requires it. If a pure projection or existing record answers the question, use it. Preserve examples in the docs and correct them in place when signatures change. Check related ids before changing a shared owner: reuse one contract change and its regression fixtures rather than implementing competing fixes. Items that name each other ("land with …") share a record; land them in one change or in dependency order.

Use [Task Workflow](../skills/task-workflow.md) for execution and [Quality Checks](../skills/quality-checks.md) for verification. Implement ready work autonomously; an item that says "needs a decision" waits for the owner, and [Blocked upstream](#blocked-upstream) records external prerequisites. Shared infrastructure work gates only the consumers that need it; unrelated ready items need not wait.

## Ready next

These need no further answer; work them in this order, one family per change.

1. **Delete first:** VM187 (Android).
2. **Small shared rules:** VM182, VM184, VM183, VM186, VM196, VM194.
3. **Screens:** VM189, VM191, VM190, VM192 with VM193.
4. **Models:** VM195.
5. **Sessions:** VM185.

Waiting on the owner: BD29 and BD50 (server), VM79, VM181. Waiting on a date or a release: X168, X163.

## Screen coverage and existing infrastructure

This map routes work to current owners. It groups existing ids rather than creating a new service or task for every view. Shared components and platform-only features keep their established boundaries; the service map in [ARCHITECTURE.md](ARCHITECTURE.md#screen-services) supplies the current app callers.

| Screens / entry points | Existing owner or infrastructure to extend | Open work |
|---|---|---|
| App start, foreground, wallet switch, deep links and pushes | `GemAppStartService`, `GemWalletSessionService`, `GemNavigationService`, `GemAppUpdateService`, native lifecycle hosts | VM79, VM182, VM185 |
| Create/import wallet, terms, phrase generation and verification | `GemWalletService`, `GemVerifyPhraseSession`, `phrase_suggestions`, keystore and native auth ports | VM183 |
| Wallet list/detail, rename, avatar, secret export | Wallet rows/details, existing export flow and NFT avatar selection | VM184 |
| Wallet home, header, network assets, banners | `GemWalletHomeService`, `GemBalanceService`, `GemBannerService`, shared asset rows and banner context | VM196 |
| Asset search/select, add token, recents | `GemAssetSelectionService`, `GemSelectAssetFlow`, `GemAddAssetService`, recent activity | VM182 |
| Asset details and asset actions | `GemAssetDetailsService`, shared rows, copy, info and load state | VM189 |
| Portfolio chart/statistics | `GemPortfolioService`, chart load rules, shared numbers and rows | — |
| Asset chart/market/alerts sections | `GemChartService`, `GemChartSession`, shared list renderer | VM189 |
| Receive, QR display, address details | `GemReceiveService`, `GemAddressDetailsService`, `GemCopy`, payment encoding | Retain existing native QR/share adapters |
| Scanner, payment links, deep links and pushes | Existing payment decoder, `GemPaymentService`, push/navigation preparation | VM185 |
| Amount entry, fiat equivalent, amount extras | `GemAmountService`, `GemAmountRequest`, `GemAmountEntry`, `GemAutocloseDraft` | VM192 |
| Confirmation, fees, simulation, acquisition | `GemConfirmTransferService`, `GemConfirmation`, `GemConfirmScreen`, shared headers/rows/info | VM190, VM196 |
| Swap, providers, slippage and swap details | `GemSwapQuoteService`, `GemSwapSession`, `GemSlippageSession` | VM193 |
| Activity, asset/position history, transaction details | `GemTransactionsService`, `GemTransactionDetailsService`, detail records, native indexed queries | VM186, VM194, VM196 |
| Buy/sell quotes, provider opening, fiat history | `GemFiatQuoteService`, `GemFiatSession`, existing fiat transaction owner | — |
| Perpetual market list/search/pins and balance | `GemPerpetualService`, market session/rows, native search indexes | — |
| Perpetual position/details/candles/activity | `GemPerpetualDetailsService`, `GemCandleSession`, position rows, chart load rules | — |
| Stake, validators, delegation and claim | `GemStakeService`, validator/delegation records, generated transfer input | Preserve exact atomic values |
| NFT root/collection/unverified, detail/report/avatar | `GemNftService`, `GemCollectibleService`, shared rich rows and avatar flow | VM196 |
| Price-alert list/target/auto-alert controls | `GemPriceAlertService`, alert session and existing notification port | VM191, VM193 |
| Rewards/create/use/redeem referral | `GemRewardsService`, rewards state, shared load and list records | — |
| Contacts/list/editor/address picker | `GemContactService`, `GemContactEditorService`, contact session/name component | — |
| Networks/node list/add/check | `GemChainSettingsService`, node sessions, shared rows | — |
| Settings/preferences/currency/language/appearance/about | `GemSettingsService`, `GemCurrencyService`, `GemAppUpdateService`, preference observation | VM189; retain native locale/theme application |
| Security/lock/biometry/recovery | `GemSecurityService`, existing keystore/auth ports and settings sections | VM189; retain platform-only privacy lock |
| Push settings, in-app notifications, support chat | Notification services, `GemSupportService`, permission and lifecycle ports | VM189 |
| WalletConnect list/detail/proposal/request/signing | `GemWalletConnectService` (sign messages scanned through `GemScanService`), `GemSignMessageService`, Reown adapters | retain Android-only one-click auth |
| Info sheets, docs links and shared display components | `GemInfoTopic`, `GemFormattedNumber`, shared rich/plain renderers, the two mapper files per app | — |
| Widgets | `GemWidgetService` (Android); the iOS widget stays off Gemstone by rule | retain native widget scheduling |
| Stores and persistence | `Gem*Store` traits and both adapters | VM187, VM195 |

An id belongs in this table only while its bullet exists below. The upstream items stay in their own section.

## Completion contract for every item

- Name the existing Core owner, native consumers and infrastructure being reused. A new type, service or session needs an actual missing responsibility; a pure projection or one-input screen is not a reason for a wrapper.
- Implement domain behavior and meaningful tests in Core; preserve exact values, wallet/request identity, atomic writes, failure outcomes, cancellation and auth. Native SQL may retain indexed filtering and sorting while implementing the same product query contract.
- Wire both apps, remove the replaced path, and keep platform mapping and observation thin. Use generated models, shared rows, copy, info, number and load infrastructure and the module localization and style mappers. Reuse native routes, OS ports, Room/GRDB transactions and test kits.
- Test the contract being moved: Core decisions and ordering; real adapters for transaction, mapping and query behavior; app tests for wiring and visible output. For async screens include stale success and failure, wallet/asset/currency changes, empty first load and refresh with existing content as applicable. Do not remove meaningful integration coverage when pruning a duplicated rule test.
- Run the applicable [Quality Checks](../skills/quality-checks.md), regenerate and build both apps when the boundary changes, and exercise reachable changed UI. Delete the item's line when complete and remove its id from the screen map. Do not claim that a static scan proves product parity.

## 1. Correctness and swallowed failures

Transaction-critical input, a user-visible outcome that a swallowed error hides, a write that runs when nothing changed, and a policy the two apps run on different triggers.


## 2. One view state per screen

A screen whose state changes is a session, and a screen that reads gets one record ([a screen whose state changes is a session](ARCHITECTURE.md#a-screen-whose-state-changes-is-a-session), [one phase enum](ARCHITECTURE.md#a-screens-state-is-one-phase-enum-never-a-bag-of-flags), [sections, actions and destinations are records](ARCHITECTURE.md#sections-actions-and-destinations-are-records-too)). Each item below is a screen that still makes several Core calls per render or emission, or rebuilds a phase from flags Core hands back separately.

- **VM190** **M** **The confirm screen's sections are assembled by each app.** Core hands back the pieces (`GemConfirmViewState`: notice, row contents, fee row, verification, simulation); the order and visibility are written twice.
  - **iOS:** `ConfirmTransferSceneViewModel.sections` orders header, notice, details, warnings, payload, balance changes, fee or verification, and error; it hides the load error while a notice shows. `ConfirmSimulationState` copies Core's simulation into default arrays.
  - **Android:** `ConfirmScreen` composes the same blocks in the same order; `ConfirmViewModel` hides the load error while a notice shows (`if (notice != null) return@combine null`) and picks verification over fee. `Simulation.kt` is the same twin as the iOS one.
  - **Expected:** the view state carries the ordered sections with their content; both apps render them, and both simulation twins go.
- **VM191** **S** **The price-alert editor keeps its state in the app and rebuilds the session from it.** `GemPriceAlertSession` exists, but neither app holds one.
  - **iOS:** `SetPriceAlertViewModel` keeps type, direction and input in `SetPriceAlertViewModelState` plus `isSaving`, and builds `newAlertSession(...).onType().onDirection().onInput().onPrice().onSaving()` on every read.
  - **Android:** `PriceAlertTargetViewModel` keeps the same four values in flows and rebuilds the same chain in a `combine`.
  - **Expected:** the view model holds one session and feeds each change to it, as swap does; the state struct and the flows go.
- **VM192** **M** **The amount entry is driven by each app around Core's entry calls.** Core computes the entry (`GemAmountType.entry`, `GemAmountInput.max_entry`, `prefill`); the input handling around it is written twice.
  - **iOS:** `AmountSceneViewModel` keeps the text and input type, clears the text when the input type is toggled, applies `prefill` and `maxEntry` by setting the input type and converting the value with `NumberInput.format().inputText`, and rebuilds the entry on every change. It also builds the `GemAmountRequest` from its route input (`switch input.type`).
  - **Android:** `AmountViewModel` does the same (`switchInputType`, `prefillAmount`, `onMaxAmount`, `maxAmountText`) and builds the request from its route parameters (`when (params)`).
  - **Expected:** an amount session owns the text and input type and answers toggle, prefill and max with the new text and entry; the apps only bind the field.


## 3. Decisions the apps still make

The same product rule written in both apps, or in one app while the other reads Core ([the app maps; it does not decide](ARCHITECTURE.md#5-the-app-maps-it-does-not-decide), [a view never names a Core type](ARCHITECTURE.md#a-view-never-names-a-core-type)). Where the two apps answer differently today the item says how.

- **VM194** **S** **Both apps turn Core's activity filters into their own request filter the same way.** Core returns `GemActivityFilters` (`activity_filters`); each app keeps the selected chains and types and converts them.
  - **iOS:** `TransactionsRequestFilter+Activity.swift` builds `activity`, `activityDefaults` and `pendingActivity` into the Store's `TransactionsRequestFilter` enum.
  - **Android:** the `TransactionsRequestFilter` companion (`gemcore/.../transactions/cases/TransactionsRequestFilter.kt`) builds the same three into its own enum.
  - **Expected:** the native activity queries read `GemActivityFilters` (plus the asset and state filters they add) directly; both enums and both conversions go.
- **VM182** **S** **Whether an asset opens the asset or the perpetual screen is decided in the apps.** Core already answers it for pushes and deep links: `GemNavigationTarget::Asset` and `::Transaction` carry `is_perpetual` (`core/gemstone/src/services/navigation/mod.rs`).
  - **iOS:** `NavigationRouter.open(target:)` drops that flag (`case let .asset(asset, walletId, _)`) and re-derives it from `asset.type` in `getPath(for:)` (twice). `NavigationStateManager.openAsset` branches on `asset.type == .perpetual` for every in-app open.
  - **Android:** `NavigationTargetRoutes.kt` reads `isPerpetual`, but `WalletSearchScreen.kt` branches on `AssetType.PERPETUAL` when a recent is opened.
  - **Expected:** one Core answer for both paths: iOS reads `is_perpetual` from the target, and an in-app open asks the navigation service for the same target instead of checking the type; all three type checks go.
- **VM183** **S** **Phrase suggestions are cut and applied by each app.** Core returns suggestions for one word (`phrase_suggestions`, `core/gemstone/src/mnemonic.rs`); the input handling around it is written twice.
  - **iOS:** `ImportWalletSceneViewModel` takes the last whitespace-separated word, offers suggestions only while the cursor is at the end, and applies a pick by dropping the last word and appending the word and a space.
  - **Android:** `ImportViewModel` does the same with its own `lastWord()`, cursor check and `selectSuggestion`.
  - **Expected:** Core owns both steps beside `phrase_suggestions`: one call takes the input and whether the cursor is at the end and returns the suggestions, one returns the input with the picked word applied.
- **VM184** **S** **How long a copied secret stays on the clipboard is hardcoded in each app.** `GemCopyKind::is_sensitive` (`core/gemstone/src/models/copy.rs`) says which copies are secret; the lifetime is not in Core ([security](../skills/security.md)).
  - **iOS:** `CopyTypeViewModel` gives a sensitive copy a local-only pasteboard item that expires after 60 seconds.
  - **Android:** `ClipboardExt.setClip` marks it sensitive and schedules `clearPrimaryClip()` after one minute, which also wipes anything the user copied after it.
  - **Expected:** the copy kind carries the lifetime next to `is_sensitive` and both apps read it; Android clears the clipboard only while it still holds that copy, like the iOS expiry.
- **VM185** **M** **What opening a link or a scanned code shows is decided in each app.** Core parses the code (`GemDeeplinkService.url_action`) and builds the target; the outcome around it is not shared.
  - **iOS:** `NavigationRouter.open(code:)` shows "Not supported" for any code without an action, shows a loading toast for a payment link, and shows every failure as an error alert.
  - **Android:** `PendingNavigationCoordinator.buildRoutes` holds the code until unlock and decides "handled" per source (an unmatched link from an intent is silent unless it is a payment; a scan is unhandled unless it has routes or is WalletConnect). The coordinator swallows every deep-link failure except `NoAccountForChain`, and `MainViewModel` shows a payment failure only while its loading state is up and a service failure only when the input had a code.
  - **Expected:** iOS behaviour, decided in Core: one call returns the outcome for a code (open this target, pair WalletConnect, show loading first, or show this unsupported/failure text), and the apps keep only the unlock wait and the native presentation.

## 4. Numbers and text the apps still format

[A number crosses as a value and a style](ARCHITECTURE.md#a-number-crosses-as-a-value-and-a-style-never-as-a-string-or-a-callback) and [the record carries the finished value](ARCHITECTURE.md#the-record-carries-the-finished-value-not-the-ingredients). Each item is a raw amount, price or seconds value that both apps format, convert or compose themselves.

- **VM196** **S** **Some Core enums are turned into text outside the two mapper files, in both apps.** The parity check reads only the mapper files, so these escape it.
  - **iOS:** `NetworkFeeCustomViewModel` (`GemCustomFeeCheck`), `BannerButtonViewModel` (`GemBannerButton`), `CollectibleViewModel` (`GemCollectibleAction` labels), `ChainsFilterTypeViewModel` and `TransactionsFilterTypeViewModel` (filter summaries).
  - **Android:** `NetworkFeeCustomViewModel` (`GemCustomFeeCheck`), `WelcomeBanner` (`GemBannerButton`), `TransactionsViewModel` (`GemChainsFilterSummary`, `GemTransactionsFilterSummary`).
  - **Expected:** each mapping moves into `Gemstone+Localized.swift` and `GemstoneText.kt`.
- **VM193** **S** **Values Core hands back are turned into input text by the apps.** Swap's "use minimum amount" (iOS `SwapSceneViewModel.setFromValue(minimum:)`, Android `SwapViewModel.setPayValue`) and the price-alert suggestions (iOS `PriceSuggestion.inputValue`, Android `PriceAlertTargetViewModel.suggestion`) each convert the value with `NumberInput`/`numberFormat().inputText` or `valueText`. The owning session returns the text (the swap session for its minimum, the alert session for its suggestions). Land with VM192, which does the same for amount prefill and max.
- **VM186** **S** **Both apps put "≈" in front of an estimated duration.** iOS `EstimatedConfirmationFormatter` (`GemstonePrimitives/Sources/DurationFormatters.swift`) and Android `formatEstimate` (`gemcore/.../domains/duration/DurationFormatter.kt`, used by `GemListRowUIModel` for estimate rows) each compose `"≈ " + duration`. The fiat quote row already composes its estimate in Core around the platform-formatted value (`GemFiatQuoteRow::crypto_estimate_text`); give the duration estimate the same method and delete both prefixes.


## 5. Twins, adapters, redundant models and dead code

[An app row model stores the row and nothing else](ARCHITECTURE.md#an-app-row-model-stores-the-row-and-nothing-else), [a details screen gets a details record](ARCHITECTURE.md#a-details-screen-gets-a-details-record-not-a-row-plus-the-object-it-came-from), and a type that only crosses the FFI is used as the generated type. Delete-first items are at the end.

- **VM195** **L** **Each app hand-writes its own asset read model and converts it for Core.** Core has `Asset`, `Price`, `AssetMetaData` and `GemAssetBalance`, but the composite a store read returns is written twice, with different shapes.
  - **iOS:** `AssetData`, `ChainAssetData`, `Balance`, `AssetValuePrice` and `RecentAsset` (`Primitives`), converted by `GemAssetBalance(_:assetId:isActive:)` and `AssetData.rowInput` (`ListAssetItemsViewModel.swift`).
  - **Android:** `AssetInfo`, `ChainAssetInfo`, `Balance`, `AssetBalance`, `AssetPriceValue` and `RecentAsset` (`gemcore/.../model`), converted by `AssetBalance.toGem` and `AssetInfo.rowInput` (`AssetInfoDataAggregate.kt`), which also drops non-finite prices where iOS does not.
  - **Expected:** one generated read model from Core primitives that both store adapters fill, taken directly by Core's row functions; both sets of twins, conversions and row-input builders go.
- **VM187** **S** **Android keeps helpers nothing calls.** `String.words()` (`gemcore/.../ext/StringExt.kt`, only its own test), `AssetsDao.insertBalance` (the plural is used), `BannersDao.observeBanner`, `TransactionsDao.deleteByState` and the three `toSearchRecord` overloads in `DbSearch.kt`. Delete them and the test that only covers `words()`.
- **VM181** **S** **Keystore secrets are exported only for the flows that need them.** `GemKeystore.create_store`, `export_private_key` and `export_recovery_phrase` are exported for app tests alone (iOS `LocalKeystore+Export.swift`, `LocalKeystore+Keystore.swift`; Android `MigrateV3KeystoreFilesTest`, `GemKeystoreBenchmarkTest`, `GemKeystoreConcurrencyTest`), while the apps import and export through the wallet service. Move those tests onto the production path, then make the three methods a plain `impl`, so no secret-exporting symbol exists that no flow uses ([security](../skills/security.md)). `check-ffi-surface.py` allows the three until then. Needs a decision: no production flow reaches these three, but `GemWalletService.export_secret` returns the same secrets in the same process, so dropping them narrows the binding rather than closing a path, and it costs rewriting the iOS keystore test kit (`LocalKeystore+Keystore.swift`, which most wallet tests use to create a wallet) and the keystore integration, benchmark and v3-migration tests on both apps onto `import_wallet` and `export_secret`. Drop them, or keep them for those tests?

## 6. Orchestration, services and stores

Core has no runtime, so scheduling, timers and OS callbacks stay in the apps; what moves is the decision — what to do, in what order, under what condition — returned as one call or one record. And [a store returns what Core reads](ARCHITECTURE.md#4-the-store-trait-is-the-apps-only-persistence-obligation), through one trait per responsibility.

- **VM79** **S** **The root scene stops reading the wallet store.** iOS [`RootSceneViewModel`](../ios/Gem/ViewModels/RootSceneViewModel.swift) reads `stores.walletStore.getWallet` directly; the session service answers the current wallet. Blocked on a synchronous answer: `GemWalletSessionService::get_current_wallet` is async (the wallet store port is async), and the root view needs the wallet on its first render or it flashes onboarding at every launch; either the port gains a synchronous read or the root keeps a stored wallet it can seed before first render.

## 7. Rows and taps

Taps on rows that already exist, not new row types.

- **VM189** **M** **Both apps map row titles to taps.** Core sends rows keyed by `GemListRowTitle`; each app keeps a table from title to destination for the same screens.
  - **iOS:** asset details (`AssetSceneViewModel.rowAction`, `balanceAction`), settings (`SettingsRowDestination`), preferences (`PreferencesRowDestination`), security (`SecurityViewModel`), notifications (`NotificationsScene`), chart (`ChartScene`).
  - **Android:** asset details (`AssetDetailRowAction.kt`), settings (`SettingsRowUIModel.settingsAction`), preferences (`PreferencesRowAction.kt`), security (`SecurityRowAction.kt`), notifications (`NotificationsScene`), chart (`AssetChartScene`).
  - **Expected:** each tappable row carries its tap as a Core record (destination, toggle or picker), and each app maps that record to its routes once; the title tables go.


## 8. Persistence and parity


## 9. Behavior differences

Differences between the apps, or between an app and the server, each with its decision.

### Same rule, different answers

- **BD29** **S** **Redemption options with unlimited stock are never listed.** `summary.rs:18` (`remaining.unwrap_or_default() > 0` drops `None`) vs storage `rewards_redemptions_repository.rs:48,96` and Core `rules.rs:146` (`None` = unlimited). Needs a decision: `test_available_redemption_options` pins a `None` stock as hidden, so confirm whether options stored without a stock are meant to be offered before changing the server.

### Freshness

- **BD50** **S** **Public `/chain/fee-estimates` can serve very old estimates.** `core/crates/services/src/chain/fee_estimates_client.rs:73-76` has no freshness check (TTL 5 years, `core/crates/cacher/src/keys.rs:155`); the per-chain route refreshes (`:55-70`). Used by the website, not the apps. **Needs a decision (2026-09-24):** either the public route drops entries past the one-hour fresh key (the website loses chains nobody requested lately) or it refreshes them (a public route then triggers node calls).

## Blocked upstream

- **X168** **S** `WalletConfiguration.multi_signature_accounts` ([`wallet_configuration.rs`](../core/crates/primitives/src/wallet_configuration.rs)) is the old name of `externally_controlled_accounts`, which also covers Solana accounts assigned to another program. The API fills both because shipped apps read only the old field. Delete the field, its fill in [`wallet_configuration.rs`](../core/crates/services/src/devices/wallet_configuration_client.rs) and the merge in [`externally_controlled_banners`](../core/gemstone/src/services/wallet_configuration/rules.rs) on 2026-12-18, three months after the release that reads `externally_controlled_accounts`.
- **X163** **M** iOS pins the `Gemstone` package to Swift 5 language mode. Re-tested on 2026-09-16 against uniffi 0.32.1: both `uniffiTraitInterfaceCallAsync` sites still fail with "passing closure as a 'sending' parameter" because the generated `Task { }` captures three `@escaping` non-`Sendable` parameters. Nothing to decide and nothing to do until a uniffi release changes that function; re-test then. Rechecked on 2026-09-22 with `cargo search uniffi`: 0.32.1 is still the latest uniffi release, and the fix is already merged upstream as uniffi #2929 (`694fda6a05`, 2026-07-15, marks the async trait-interface closures `@Sendable`) but is not in 0.32.1. Bump uniffi to the first release that contains #2929, regenerate, and move the `Gemstone` package to Swift 6 mode.
