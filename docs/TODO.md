# Open work

The goal is that Gemstone decides once and both clients read that decision. Shared rules, records, sessions and orchestration live in Core; rendering, observation, scheduling, OS ports and locale formatting live in the apps. A view model, UI model, aggregate, factory or provider that still decides, re-derives, composes text or formats a number is debt; a model that holds one Core record and maps it to platform values is the target shape. Contracts and worked examples are in [ARCHITECTURE.md](ARCHITECTURE.md).

Every open item carries a stable id and a size (**S**/**M**/**L**); ids are never reused. Namespaces: `VM` (consolidation into Gemstone), `AUD` (correctness found by review), `BD` (a behaviour difference), `D` (a decided product or security choice being built), `GEN` (code `just generate-models` writes instead of hand-written test mocks and mappers), `MOD` (feature modules named and grouped the same way on both apps), `CLN` (a cleanup sweep across every platform) and `X` (blocked upstream).

- **Closing an item:** delete its line and commit at once. If it settles an important user-facing rule someone might simplify away, write that rule on its area page under [product/](product/) in the same commit, following [PRODUCT.md § Writing a page](PRODUCT.md#writing-a-page); most items have none. Nothing else is recorded here: git history carries the detail.
- **Comparing the apps:** say what each does on its own line — **iOS:**, **Android:**, then **Expected:** (the agreed behaviour, or "needs a decision") — never in one sentence covering both. When the apps differ and no item decides otherwise, iOS is the reference behaviour.

## Working one item

Pick one item by its stable id and use the screen map to find its current owner. Source paths and line numbers are retrieval hints, not proof that an old finding still applies. Read the owner, both consumers and the linked architecture example before editing.

Keep the implementation plan in the task update: the existing owner to change, the app path to delete, any real prerequisite and the exact relevant checks. Do not create a new planning artifact, service, session, cache or framework unless the current contract requires it. If a pure projection or existing record answers the question, use it. Preserve examples in the docs and correct them in place when signatures change. Check related ids before changing a shared owner: reuse one contract change and its regression fixtures rather than implementing competing fixes. Items that name each other ("land with …") share a record; land them in one change or in dependency order.

Use [Task Workflow](../skills/task-workflow.md) for execution and [Quality Checks](../skills/quality-checks.md) for verification. Implement ready work autonomously; an item that says "needs a decision" waits for the owner, and [Blocked upstream](#blocked-upstream) records external prerequisites. Shared infrastructure work gates only the consumers that need it; unrelated ready items need not wait.

## Ready next

These need no further answer; work them in this order, one family per change.

1. **Generated mocks:** GEN297 type by type, then GEN298.
2. **Small shared rules:** VM182, VM184, VM183, VM186, VM196, VM194.
3. **Screens:** VM189, VM191, VM190, VM192 with VM193.
4. **Models:** VM195.
5. **Sessions:** VM185.
6. **App models to Core records:** VM197 to VM208 (shared components, which later items reuse), then VM209 to VM260 area by area as grouped in section 5, then VM261 to VM289 (second round) and VM290 to VM295 (scenes) in the same way.
7. **Generated mappers:** BD299, then GEN300.
8. **Module layout:** MOD301, then the renames MOD302 to MOD306, then MOD307 before MOD308, MOD309 with MOD310, MOD311, MOD312, MOD313, then MOD314 to MOD317; MOD320 area by area, with MOD321 in its next change.
9. **Unused code:** CLN318.
10. **Unit test review, last:** CLN319, after every other ready item, so it reviews the tests that remain once rules have moved into Core.

Waiting on the owner: BD29 and BD50 (server), VM79, VM181, D175 (on hold). Waiting on a date or a release: X168, X163.

## Screen coverage and existing infrastructure

This map routes work to current owners. It groups existing ids rather than creating a new service or task for every view. Shared components and platform-only features keep their established boundaries; the service map in [ARCHITECTURE.md](ARCHITECTURE.md#screen-services) supplies the current app callers.

| Screens / entry points | Existing owner or infrastructure to extend | Open work |
|---|---|---|
| App start, foreground, wallet switch, deep links and pushes | `GemAppStartService`, `GemWalletSessionService`, `GemNavigationService`, `GemAppUpdateService`, native lifecycle hosts | VM79, VM182, VM185, VM275 |
| Create/import wallet, terms, phrase generation and verification | `GemWalletService`, `GemVerifyPhraseSession`, `phrase_suggestions`, keystore and native auth ports | VM183, VM241, VM242, VM294 |
| Wallet list/detail, rename, avatar, secret export | Wallet rows/details, existing export flow and NFT avatar selection | VM184, VM237, VM238, VM239, VM240, VM294 |
| Wallet home, header, network assets, banners | `GemWalletHomeService`, `GemBalanceService`, `GemBannerService`, shared asset rows and banner context | VM196, VM200, VM202, VM204, VM264, VM269 |
| Asset search/select, add token, recents | `GemAssetSelectionService`, `GemSelectAssetFlow`, `GemAddAssetService`, recent activity | VM182, VM250, VM251, VM252, VM254, VM262, VM263, VM286 |
| Asset details and asset actions | `GemAssetDetailsService`, shared rows, copy, info and load state | VM189, VM253, VM261, VM285 |
| Portfolio chart/statistics | `GemPortfolioService`, chart load rules, shared numbers and rows | — |
| Asset chart/market/alerts sections | `GemChartService`, `GemChartSession`, shared list renderer | VM189, VM232 |
| Receive, QR display, address details | `GemReceiveService`, `GemAddressDetailsService`, `GemCopy`, payment encoding | Retain existing native QR/share adapters |
| Scanner, payment links, deep links and pushes | Existing payment decoder, `GemPaymentService`, push/navigation preparation | VM185, VM284 |
| Amount entry, fiat equivalent, amount extras | `GemAmountService`, `GemAmountRequest`, `GemAmountEntry`, `GemAutocloseDraft` | VM192, VM222, VM223, VM272, VM287 |
| Confirmation, fees, simulation, acquisition | `GemConfirmTransferService`, `GemConfirmation`, `GemConfirmScreen`, shared headers/rows/info | VM190, VM196, VM201, VM214, VM215, VM216, VM217, VM218, VM219, VM220, VM221, VM267 |
| Swap, providers, slippage and swap details | `GemSwapQuoteService`, `GemSwapSession`, `GemSlippageSession` | VM193, VM224, VM227, VM228, VM295 |
| Activity, asset/position history, transaction details | `GemTransactionsService`, `GemTransactionDetailsService`, detail records, native indexed queries | VM186, VM194, VM196, VM207, VM209, VM210, VM211, VM212, VM213, VM276, VM280, VM290 |
| Buy/sell quotes, provider opening, fiat history | `GemFiatQuoteService`, `GemFiatSession`, existing fiat transaction owner | VM225, VM226 |
| Perpetual market list/search/pins and balance | `GemPerpetualService`, market session/rows, native search indexes | VM282, VM289 |
| Perpetual position/details/candles/activity | `GemPerpetualDetailsService`, `GemCandleSession`, position rows, chart load rules | VM231, VM233, VM234, VM235 |
| Stake, validators, delegation and claim | `GemStakeService`, validator/delegation records, generated transfer input | VM229, VM230, VM278, VM279; preserve exact atomic values |
| NFT root/collection/unverified, detail/report/avatar | `GemNftService`, `GemCollectibleService`, shared rich rows and avatar flow | VM196, VM255, VM256 |
| Price-alert list/target/auto-alert controls | `GemPriceAlertService`, alert session and existing notification port | VM191, VM193, VM248, VM249, VM271, VM277 |
| Rewards/create/use/redeem referral | `GemRewardsService`, rewards state, shared load and list records | VM257, VM258, VM274, VM291, VM292 |
| Contacts/list/editor/address picker | `GemContactService`, `GemContactEditorService`, contact session/name component | VM208, VM246 |
| Networks/node list/add/check | `GemChainSettingsService`, node sessions, shared rows | VM199, VM244, VM245, VM265, VM270 |
| Settings/preferences/currency/language/appearance/about | `GemSettingsService`, `GemCurrencyService`, `GemAppUpdateService`, preference observation | VM189, VM243, VM283, VM290; retain native locale/theme application |
| Security/lock/biometry/recovery | `GemSecurityService`, existing keystore/auth ports and settings sections | VM189; retain platform-only privacy lock |
| Push settings, in-app notifications, support chat | Notification services, `GemSupportService`, permission and lifecycle ports | VM189, VM247, VM259 |
| WalletConnect list/detail/proposal/request/signing | `GemWalletConnectService` (sign messages scanned through `GemScanService`), `GemSignMessageService`, Reown adapters | VM260, VM281, VM291; retain Android-only one-click auth |
| Info sheets, docs links and shared display components | `GemInfoTopic`, `GemFormattedNumber`, shared rich/plain renderers, the two mapper files per app | VM197, VM198, VM203, VM205, VM206, VM266 |
| Widgets | `GemWidgetService` (Android); the iOS widget stays off Gemstone by rule | retain native widget scheduling |
| Stores and persistence | `Gem*Store` traits and both adapters | D175, VM195, VM288, BD299, GEN300 |
| Test mocks | `core/bin/generate` with `remote_types.yml`, iOS `TestKit` targets, Android `testFixtures` | GEN297, GEN298 |
| Unused code and unit tests, every platform | `scripts/check-ffi-surface.py`, `cargo machete`, each module's test target | CLN318, CLN319 |

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

- **VM196** **M** **Core values are turned into text, icons and styles outside the two mapper files, in both apps.** The parity check reads only the mapper files, so these escape it.
  - **iOS:** text in `NetworkFeeCustomViewModel` (`GemCustomFeeCheck`), `BannerButtonViewModel` (`GemBannerButton`), `CollectibleViewModel` (`GemCollectibleAction` labels), `ChainsFilterTypeViewModel` and `TransactionsFilterTypeViewModel` (filter summaries); icons and styles in `SwapProviderType+Gemstone`, `FiatProviderName+`, `GemSocialLink+`, `GemLatencyStatus+`, `GemButtonState+`, `GemTransactionHeader+`, `GemHeaderActions+`, `GemAvatar+` and `GemLoadState+StateViewType` (`PrimitivesComponents/Sources/Extensions`).
  - **Android:** text in `NetworkFeeCustomViewModel` (`GemCustomFeeCheck`), `WelcomeBanner` (`GemBannerButton`), `TransactionsViewModel` (`GemChainsFilterSummary`, `GemTransactionsFilterSummary`); icons and labels in `VerificationStatusItem.kt` and `PinnedAssetsHeaderItem.kt`.
  - **Expected:** each mapping moves into `Gemstone+Localized.swift`/`Gemstone+Style.swift` and `GemstoneText.kt`/`GemstoneStyle.kt`. `GemCopy+`, `GemWalletRow+` and `GemSecretPhraseRow+` go with VM206, VM237 and VM294; Android `ChainIcon.kt` stays where `android/scripts/convert-icons.sh` generates it.
- **VM193** **S** **Values Core hands back are turned into input text by the apps.** Swap's "use minimum amount" (iOS `SwapSceneViewModel.setFromValue(minimum:)`, Android `SwapViewModel.setPayValue`) and the price-alert suggestions (iOS `PriceSuggestion.inputValue`, Android `PriceAlertTargetViewModel.suggestion`) each convert the value with `NumberInput`/`numberFormat().inputText` or `valueText`. The owning session returns the text (the swap session for its minimum, the alert session for its suggestions). Land with VM192, which does the same for amount prefill and max.
- **VM186** **S** **Both apps put "≈" in front of an estimated duration.** iOS `EstimatedConfirmationFormatter` (`GemstonePrimitives/Sources/DurationFormatters.swift`) and Android `formatEstimate` (`gemcore/.../domains/duration/DurationFormatter.kt`, used by `GemListRowUIModel` for estimate rows) each compose `"≈ " + duration`. The fiat quote row already composes its estimate in Core around the platform-formatted value (`GemFiatQuoteRow::crypto_estimate_text`); give the duration estimate the same method and delete both prefixes.


## 5. Twins, adapters, redundant models and dead code

[An app row model stores the row and nothing else](ARCHITECTURE.md#an-app-row-model-stores-the-row-and-nothing-else), [a details screen gets a details record](ARCHITECTURE.md#a-details-screen-gets-a-details-record-not-a-row-plus-the-object-it-came-from), and a type that only crosses the FFI is used as the generated type. Delete-first items are at the end.

The target for every item below: a model that only renames or regroups a Core record is deleted and the view takes the record; the two mapper files turn titles, tones and icon kinds into platform values; anything the model decides (a text, an icon choice, a visibility, a grouping) moves into the Core row, list or session that feeds it.

### One generated type system

- **D175** **L** **The apps use only the UniFFI-generated types.** On hold by the owner (2026-09-25). TypeShare keeps generating TypeScript for the web; its Swift and Kotlin models (`Primitives/Sources/Generated`, `gemcore/.../primitives/generated`, 161 types) and the generated mappers between the two systems (`RemoteTypeMappers.swift`, `RemoteTypeMappers.kt`) go.
  - **Today:** 141 of the 161 TypeShare types already have a UniFFI twin (`core/bin/generate/remote_types.yml`). 936 iOS files and 638 Android files import the TypeShare models, and about 210 files per app convert with `toPrimitives()`/`toGem()`. `Chain` crosses as its string code, and `AssetId`, `WalletId`, `TransactionId`, `NFTAssetId`, `NFTCollectionId` and `PerpetualId` cross as their stored strings. iOS `Store` depends on `Primitives` but not on `Gemstone`.
  - **Phase 1, coverage:** declare the 20 TypeShare types the bindings lack, or delete their app use: `AssetPriceInfo`, `AssetSubtype`, `BitcoinChain`, `ChartValue`, `ContactData`, `CosmosChain`, `Device`, `EVMChain`, `FiatAssets`, `PerpetualAccountSummary`, `PerpetualPortfolio`, `PerpetualPositionsSummary`, `PriceData`, `QRScanType`, `ScanReceiveMode`, `StakeChain`, `TransactionNFTTransferMetadata`, `TransactionPerpetualMetadata`, `TransactionSwapMetadata`, `WCPairingProposal`.
  - **Phase 2, enums and identifiers:** `Chain` becomes a generated UniFFI enum and the identifiers generated records (`AssetId` carries chain and token id). `just generate-models` emits their stored-string conversions on both apps, so no app parses an identifier by hand.
  - **Phase 3, storage and routes:** `just generate-models` emits `Codable` and `Hashable` conformances (iOS) and kotlinx serializers (Android) for the generated types that routes and stored JSON carry: iOS `Scenes`, Android route arguments, and three GRDB JSON columns. iOS `Store` gains the `Gemstone` dependency.
  - **Phase 4, the apps, module by module:** replace TypeShare imports with the generated types and delete each mapper once its last caller goes. Order: store adapters and `Store`, then shared components, then features. VM195 (asset read model), VM286 (`SelectAssetType`), VM288 (iOS service wrappers) and VM289 (Android aggregates) land inside this phase.
  - **Phase 5, removal:** stop generating Swift and Kotlin from TypeShare, delete both `RemoteTypeMappers` and the mapper sections of the generator, and fold the hand-written rest of the iOS `Primitives` package into `GemstonePrimitives`.
  - **Widget:** the iOS widget links neither `Gemstone` nor `GemstonePrimitives` and decodes API JSON with TypeShare `Codable` models. Default: it keeps a small widget-local model for the fields it shows, so the no-Gemstone rule stands.

### Shared components

- **VM197** **S** **`AssetIdViewModel` exists only to fetch and map an icon.**
  - **iOS:** `AssetIdViewModel` (27 call sites) calls `GemAssetConfigService.assetIcon` on every render and holds `AssetImage(icon:)`; `networkAssetImage` maps a chain.
  - **Android:** `AssetId.icon()`, `iconChain()`, `supportIconChain()` (`gemcore/.../domains/asset/IconUrlGeneration.kt`) and `ListItemImage.Asset(assetId)` do the same lookup per render.
  - **Expected:** every Core row that shows an asset carries its `GemAssetIcon`; `AssetImage(icon:)` and the Android image mapping move into the style mapper files; `AssetIdViewModel` and the `AssetId` icon helpers go.
- **VM198** **S** **`AssetViewModel` re-asks Core for asset text on every read.**
  - **iOS:** `AssetViewModel` calls `assetText` per asset for title, network name and subtitle symbol.
  - **Android:** `Asset.networkFullName` and `Asset.subtitleSymbol` (`AssetExt.kt`) call `assetText` on each access.
  - **Expected:** the rows that show these texts carry them; both wrappers go.
- **VM199** **S** **Chain rows are built by the apps.**
  - **iOS:** `ChainViewModel` builds title, image and list item from a `Chain` (network name, `standard` suffix).
  - **Android:** `ChainItem`, `NetworkScene`, `AddNodeScene` and the contact chain rows build the same row from `Chain.networkName()` and the chain icon.
  - **Expected:** `GemChainService` returns chain rows (title, subtitle, icon) and the shared renderer draws them; `ChainViewModel` goes.
- **VM200** **M** **Value headers are composed in the apps from several Core records.**
  - **iOS:** `ValueHeader` extensions build headers from `GemWalletHomeViewState`, `GemSimulationValue` (formats the value with a full-style `ValueFormatter`) and `GemPerpetualBalanceHeader` (composes "Available balance: X"); `AmountDisplay`, `NumericViewModel` and `SymbolViewModel` wrap the same parts.
  - **Android:** `AmountListHead` takes the same parts as parameters; `SimulationHeaderUIModel` formats the simulation value with `ValueFormatter(style = FULL)`; `AmountUiState` carries title, symbol and equivalent.
  - **Expected:** one Core header record (title, subtitle, tone, icon, buttons) that every header screen returns; the wrappers and header extensions go.
- **VM201** **M** **Transaction and confirm headers are picked by the apps.**
  - **iOS:** `TransactionHeaderType` (amount, swap, nft, asset, assetValue) is built from `GemConfirmHeader` and transaction details, and decides `showsClearHeader`.
  - **Android:** `ConfirmHeaderUIModel` (Simulation, Swap, Nft, Symbol, Amount, Placeholder, ReservedSpace) is built by `confirmHeader` from the same record.
  - **Expected:** one Core header enum with finished values that both headers render; `TransactionHeaderType` and `ConfirmHeaderUIModel` go.
- **VM202** **S** **Header buttons are mapped to callbacks and titles in the apps.**
  - **iOS:** `HeaderButton` and `HeaderButtonViewType` carry `GemHeaderButtonKind` with title and image outside the mapper.
  - **Android:** `HeadActionsUIModel` maps each kind to a callback, a string, an icon and a test tag (`assetBuy`).
  - **Expected:** header buttons carry their tap as a Core record (as in VM189); titles and icons come from the mapper files; both wrappers go.
- **VM203** **S** **Empty states are assembled from app callbacks.**
  - **iOS:** `EmptyContentType` and `EmptyContentTypeViewModel` build `GemEmptyStateInput` from the keys of an app callback map, then join Core's actions back to the callbacks.
  - **Android:** `EmptyStateUIModel` does the same and styles the first action as primary.
  - **Expected:** Core returns the empty state with its actions and their styles for the screen; the apps bind taps; both builders go.
- **VM204** **S** **Banner rows are wrapped and styled outside the mappers.**
  - **iOS:** `BannerViewModel`, `BannerButtonViewModel` and `BannerAction` wrap `GemBannerRow`; icon size and corner radius are chosen per icon, button titles and styles per button.
  - **Android:** `BannerRowUIModel` and `BannerItemUIModel` copy the same fields; `WelcomeBanner` picks button titles.
  - **Expected:** views take `GemBannerRow`; sizes, radii, titles and button styles live in the mapper files; the wrappers go.
- **VM205** **S** **Shared-row texts and menus are composed by the renderers.**
  - **iOS:** `GemListRowItem` writes the titles of app, wallet, memo and explorer rows itself, composes "View on X", the " #rank " tag and "Enable Face ID" for the authentication toggle, and builds the explorer, wallet and memo menus; `GemListRowView` titles the provider row "Contract".
  - **Android:** `GemListRowUIModel` composes the explorer title and the same labels, builds the website, copy-address, view-on and copy-memo menus, picks the copy title by copy kind, and exposes an address only when the title is `CONTRACT`.
  - **Expected:** Core sends every row's title, text and menu entries (copy or open, by kind); the renderers only map.
- **VM206** **S** **`CopyTypeViewModel` wraps a copy record.**
  - **iOS:** `CopyTypeViewModel` holds `GemCopy`, builds the copied message and the pasteboard options.
  - **Android:** `GemCopyExt.kt` and `ClipboardExt.kt` do the same around `GemCopy`.
  - **Expected:** a clipboard port per app takes `GemCopy` (with VM184's lifetime) and the message comes from the mapper; the wrapper goes.
- **VM207** **S** **`AddressListItemViewModel` decides how an address row reads.**
  - **iOS:** `AddressListItemViewModel` decides `canToggleAddress` (a name that is not the address), picks name or short address for the subtitle, and builds "View on X".
  - **Android:** the recipient and participant rows in `TransactionDetailsRowUIModel` and `ConfirmRowUIModel` build the same row.
  - **Expected:** one Core address row (name, short and full address, whether it toggles, explorer link, contact actions) rendered by one shared component per app.
- **VM208** **S** **The name-resolve indicator is mapped in two places per app.**
  - **iOS:** `NameRecordViewModel` drives the lookup and maps the state to an image (`resolveImage`).
  - **Android:** `NameRecordController` and `NameResolveIndicatorUIModel` do the same.
  - **Expected:** the name state carries its indicator kind and the mapper supplies the icon; the indicator models go.

### Activity

- **VM209** **M** **`TransactionViewModel` decides what a transaction row shows.**
  - **iOS:** `TransactionViewModel` chooses NFT image or asset icon, picks the overlay badge from direction, shows progress from status, and calls `transactionRows` for a single transaction.
  - **Android:** `TransactionRowUIModel` decides the badge text (`status.showsBadge`), copies every field, and `TransactionItem` picks the image.
  - **Expected:** `GemTransactionRow` carries the final image and badge and the list screen gets rows only; both wrappers go.
- **VM210** **M** **Every dated list is grouped by day in the apps.**
  - **iOS:** `DateSectionBuilder` (transactions, recents, notifications, fiat transactions) and `SupportChatDayBuilder` group by local day and sort.
  - **Android:** `rememberDateSections`, `dateSectionLabels` and `SupportChatModels` (`groupBy` on local date) do the same, relying on pre-sorted input.
  - **Expected:** Core returns sectioned lists given the device's day boundaries (`GemDayBoundaries`), labels included; the builders go.
- **VM211** **S** **Transaction participants are assembled by the apps.**
  - **iOS:** `TransactionParticipantViewModel` builds a `SimpleAccount` and an address row from `GemTransactionParticipant`, deciding contact and select actions.
  - **Android:** `TransactionDetailsRowUIModel` builds the same row (`GemTransactionParticipant.address`).
  - **Expected:** details sections carry finished participant rows (with VM207's address row); both models go.
- **VM212** **S** **Swap progress steps are composed in the apps.**
  - **iOS:** `TransactionSwapProgressItemModel` derives tag, marker, spinner and colours from `GemSwapProgressState`.
  - **Android:** `SwapProgressUIModel` composes the transfer subtitle as "amount (network)", activates the connector when the transfer step completed and shows the estimate only with a spinner.
  - **Expected:** the Core swap-progress record carries titles, subtitles and those flags; both models go.
- **VM213** **S** **Transaction filter screens keep their options and state in the apps.**
  - **iOS:** `TransactionsFilterViewModel`, `TransactionTypesFilterViewModel`, `TransactionTypesSelectorViewModel`, `TransactionsFilterTypeViewModel` and `ChainsFilterViewModel` hold the selections and titles.
  - **Android:** `TransactionFilterUIModel` builds options from `GemConstants.transactionFilters`; `TransactionsViewModel` holds the selections.
  - **Expected:** a Core filter session holds the selections and returns option rows and summaries (land with VM194).

### Transfer and confirm

- **VM214** **S** **Fee rate rows are rebuilt with app emojis.**
  - **iOS:** `FeeRateViewModel` wraps priority, value and fee, and picks the emoji per priority.
  - **Android:** `FeeRateUIModel` hardcodes the emojis, `FeeRateRowUIModel` builds the list item and `customFeeRowUIModel` adds the custom row with its own emoji.
  - **Expected:** `GemFeeRateRows` returns every row including custom, with title, emoji, value, fiat and selection; the three models go.
- **VM215** **M** **The network fee screen is modelled twice.**
  - **iOS:** `NetworkFeeSceneViewModel` and `NetworkFeeCustomViewModel` hold fee, rates, assets and custom checks.
  - **Android:** `FeeDetailsModel`, `FeeUIModel` (a twin of the fee amount record) and `NetworkFeeCustomViewModel` do the same.
  - **Expected:** one Core fee screen record (rows, custom field state, fee assets); the models go.
- **VM216** **S** **The confirm fee row decides its own text.**
  - **iOS:** `ConfirmNetworkFeeViewModel` shows fiat else value, hides the symbol when unavailable and decides selectability.
  - **Android:** `ConfirmViewModel` builds the fee list item and `FeeUIModel.Unavailable` the same way.
  - **Expected:** `GemConfirmFeeRow` carries the final texts and whether it opens details.
- **VM217** **S** **Confirm rows are assembled by the apps.**
  - **iOS:** `ConfirmRowViewModel` builds the recipient row from eight fields and titles the payment row "Pay with".
  - **Android:** `ConfirmRowUIModel` builds the same rows.
  - **Expected:** `GemConfirmRowContent` rows are complete (with VM207's address row); both models go.
- **VM218** **S** **The confirm details block is chosen by input type in the apps.**
  - **iOS:** `ConfirmDetailsViewModel` switches on `TransactionInputType` and calls `swapQuoteDetails` or `perpetualConfirmDetails` itself.
  - **Android:** `ConfirmViewModel.swapDetails` does the same for swaps.
  - **Expected:** the confirm view state carries the details record; both paths go.
- **VM219** **S** **The confirm button's icon is decided in the app.**
  - **iOS:** `ConfirmButtonViewModel` shows the biometric or passcode icon only for an enabled confirm button.
  - **Android:** the confirm button shows no authentication icon.
  - **Expected:** `GemConfirmButton` carries the icon kind and both apps show it (the iOS behaviour); the mapper supplies the image.
- **VM220** **S** **Simulated balance changes are wrapped.**
  - **iOS:** `ConfirmBalanceChangeViewModel` builds the list item from `GemSimulationBalanceChange`.
  - **Android:** `ConfirmViewModel.balanceChangeRows` maps each change with `listItem()`.
  - **Expected:** the shared renderer draws the Core row; both wrappers go.
- **VM221** **S** **The "get this asset" options are written in the apps.**
  - **iOS:** `GetAssetAction` (buy, swap, receive) and its sheet list the options.
  - **Android:** `AcquireOptionUIModel` builds Buy, Swap and Receive with their subtitles and icons.
  - **Expected:** `GemAcquireAsset` returns the option rows; both lists go.
- **VM222** **S** **The amount field's symbol and placement are decided in the apps.**
  - **iOS:** `AmountInputConfig` puts the asset symbol trailing and the currency leading, and picks the keyboard from `usesWholeAmounts`.
  - **Android:** `AmountSymbolUIModel` and `AmountUiState` make the same choices.
  - **Expected:** the amount view state carries symbol, placement and keyboard kind (land with VM192).
- **VM223** **S** **Amount extras are chosen by the apps.**
  - **iOS:** `AmountStakeViewModel` and `AmountPerpetualViewModel` supply validator, resources, leverage and autoclose rows.
  - **Android:** `AmountExtrasUIModel`, `AmountStakeProvider` and `AmountPerpetualProvider` do the same, and `AmountViewModel` picks `EarnProvider` for an earn type.
  - **Expected:** the amount session returns the extras record (land with VM192).

### Swap and fiat

- **VM224** **S** **Provider rows are wrapped twice per app.**
  - **iOS:** `SwapProviderItem` and `FiatQuoteViewModel` each build a provider list item with a selected badge.
  - **Android:** `BuyFiatProviderUIModel` and the provider rows of the swap details model do the same.
  - **Expected:** one Core provider row (name, amounts, icon kind, selected) rendered by one shared component; the wrappers go.
- **VM225** **S** **Fiat transaction rows are rebuilt by the apps.**
  - **iOS:** `FiatTransactionViewModel` builds title from quote type, badge, amounts and provider image.
  - **Android:** `FiatTransactionRowUIModel` does the same.
  - **Expected:** the shared renderer draws `GemFiatTransactionRow`; both models go.
- **VM226** **S** **The fiat screen state is twinned.**
  - **iOS:** `FiatProvidersViewModel` and `FiatCurrencyInputConfig` wrap view-state fields.
  - **Android:** `FiatUiState` copies `GemFiatViewState` and derives `retries` from the button action.
  - **Expected:** the view state is used directly; the twins go.
- **VM227** **S** **Swap side and button models decide small rules.**
  - **iOS:** `SwapTokenViewModel` picks "Select asset" or the symbol and an empty or zero placeholder; `SwapButtonViewModel` hides the button while the input is empty.
  - **Android:** `SwapUiState` copies `GemSwapViewState` field by field and `SwapItem` makes the same choices.
  - **Expected:** `GemSwapSideState` and `GemSwapViewState` carry those answers; the models go.
- **VM228** **S** **Swap details and slippage state are wrapped.**
  - **iOS:** `SwapDetailsViewModel` reads rate text, price-impact rows and the provider; `SwapSlippageViewModel` mirrors `GemSlippageViewState`.
  - **Android:** `SwapViewModel.swapDetails` (`details.uiModel(providers, allowsProviderSelection)`) and `SlippageStateUIModel` do the same.
  - **Expected:** the views read `GemSwapDetails` rows and the slippage view state directly; the wrappers go.

### Stake and perpetuals

- **VM229** **S** **Validator and delegation rows are built by the apps.**
  - **iOS:** `ValidatorViewModel` picks the provider logo or a text placeholder; `DelegationViewModel` builds the delegation row and header.
  - **Android:** `DelegationRowUIModel`, `DelegationProperties` and the validator rows do the same.
  - **Expected:** Core validator and delegation rows are drawn by the shared renderer, provider logos from the style mapper; the models go.
- **VM230** **S** **Stake sections are wrapped.**
  - **iOS:** `StakeSceneViewModel` maps sections and actions.
  - **Android:** `StakeSectionUIModel` and `StakeActionUIModel` wrap `GemStakeSection` and `GemStakeActionItem`.
  - **Expected:** the views read the Core sections; the wrappers go.
- **VM231** **S** **Perpetual details sections and buttons are wrapped.**
  - **iOS:** `PerpetualButtonViewModel` maps tone to a style; `PerpetualSceneViewModel` builds sections.
  - **Android:** `PerpetualDetailsUIModel`, `PerpetualButtonUIModel` and `PerpetualPositionRowUIModel` copy `GemPerpetualDetails` and tag autoclose rows by kind.
  - **Expected:** views read the Core sections and button rows; styles from the mapper; the models go.
- **VM232** **M** **Chart selection and scales are computed in the apps.**
  - **iOS:** `ChartValuesViewModel` pads the x-scale by 2% and resolves the header at a selection.
  - **Android:** `ChartUIModel` resolves the header at a selection and picks the date style.
  - **Expected:** `GemChartSession` answers selection (header and date style) and bounds; both models go.
- **VM233** **M** **The candle chart is modelled twice.**
  - **iOS:** `CandlestickChartViewModel`, `PerpetualChartModel` and `ChartLineViewModel` (label "kind | price") build the chart.
  - **Android:** `PerpetualChartUIModel` computes the header from the first and selected close; `CandlestickChartUIModel` builds lines and labels.
  - **Expected:** `GemCandleSession` returns the chart record with line labels and the selection header; the models go.
- **VM234** **S** **Candle tooltips are wrapped.**
  - **iOS:** `CandleTooltipViewModel` maps each `GemCandleTooltipCell`.
  - **Android:** `CandlestickTooltipUIModel` does the same.
  - **Expected:** the tooltip view reads the cells directly with mapper styles; both go.
- **VM235** **S** **Autoclose fields are held in the apps.**
  - **iOS:** `AutocloseInput` keeps both texts and turns empty into nil (`AutocloseSelection`), `AutocloseViewModel` picks "Expected profit" or "Expected loss".
  - **Android:** `AutocloseViewModel` keeps the same texts and titles.
  - **Expected:** `GemAutocloseSession` holds the field texts and returns titles; the input structs go.

### Wallets and onboarding

- **VM237** **S** **Wallet rows are rebuilt and imaged by the apps.**
  - **iOS:** `WalletEntry` pairs a wallet with `walletRow(wallet:)`; `GemWalletRow` extensions choose image, placeholder and watch badge; `WalletBarViewViewModel` copies the wallet name and avatar for the wallet bar.
  - **Android:** `WalletRowUIModel`, `WalletSectionUIModel`, `walletListItemImage` and `supportIcon` do the same; the wallet bar takes name and icon separately from `WalletRowUIModel`.
  - **Expected:** views, the wallet bar included, take `GemWalletRow`/`GemWalletSection`; image choice moves to the mapper; the wrappers and `WalletBarViewViewModel` go.
- **VM238** **S** **The wallets list splits sections in the app.**
  - **iOS:** `WalletsSceneViewModel` sorts through `walletService.sorted`; `SelectWalletViewModel` maps `GemWalletSection` kinds to titles.
  - **Android:** `WalletsUIState` splits pinned and unpinned and marks the current wallet.
  - **Expected:** Core returns the wallet sections with the current flag; both splits go.
- **VM239** **S** **Wallet details are twinned.**
  - **iOS:** `WalletIDetailViewModel` and `WalletImageViewModel` read and reshape wallet details; `WalletIDetailViewModel` also composes "Show {secret}" and the address row.
  - **Android:** `WalletDetailsUIModel` and `WalletAvatarUIModel` copy `GemWalletDetails`; `WalletViewModel` composes "Show {secret}".
  - **Expected:** `GemWalletDetails` returns its rows, texts included, and the views read it; the twins and both row builders go.
- **VM240** **S** **The secret screen decides its own title and warning.**
  - **iOS:** `SecretDataViewModel` titles words "Secret phrase" or "New wallet" depending on a continue action, picks the callout, and builds rows and copy.
  - **Android:** `WalletSecretContentUIModel` and `WalletSecretUIModel` build rows and copy.
  - **Expected:** a Core secret-screen record (title, warning kind, rows, copy); both go.
- **VM241** **S** **Terms acceptance is tracked in the apps.**
  - **iOS:** `AcceptTermsViewModel` and `TermItemViewModel` list the terms and track which are confirmed.
  - **Android:** `AcceptTermRowUIModel` lists `GemConstants.acceptTermsItems` and the screen tracks confirmation.
  - **Expected:** a terms session returns rows and whether all are accepted; both models go.
- **VM242** **S** **Import types are enumerated in the apps.**
  - **iOS:** `ImportWalletTypeViewModel` and the `ImportWalletType` enum twin the import kinds.
  - **Android:** `SelectImportTypeViewModel` and `ImportType` do the same.
  - **Expected:** Core returns the import type rows; the enums go.

### Settings

- **VM243** **S** **Currency rows are wrapped.**
  - **iOS:** `CurrencySceneViewModel` maps sections and rows.
  - **Android:** `CurrencyRowUIModel` copies `GemCurrencyRow` into a list item.
  - **Expected:** views read `GemCurrencySection`; the wrapper goes.
- **VM244** **S** **Network settings sections are fixed in the apps.**
  - **iOS:** `ChainSettingsSectionViewModel.Kind` (nodes, explorer) and `ChainNodeViewModel` build the sections.
  - **Android:** `NetworksUIState`, `NetworkSectionUIModel`, `NodeRowUIModel` and `ExplorerRowUIModel` do the same with fixed titles.
  - **Expected:** the chain settings view state returns sections with titles and rows; the models go.
- **VM245** **S** **Add-node results are shaped in the apps.**
  - **iOS:** `AddNodeSceneViewModel` shows the warning only once checks exist.
  - **Android:** `AddNodeUIModel` derives error text and checks from the phase, shows the warning once checks exist and maps sync state to a flag.
  - **Expected:** the add-node view state carries error, check rows and warning; both models go.
- **VM246** **S** **Contact address rows are built in the apps.**
  - **iOS:** `ContactEditorViewModel.listItemModel(for:)` builds network name, short address and image.
  - **Android:** `ContactAddressRowUIModel` and `addAddressListItem` do the same.
  - **Expected:** the contact session returns address rows; both builders go.
- **VM247** **S** **Notification rows decide their tag and icon.**
  - **iOS:** `InAppNotificationListItemViewModel` shows "New" when unread and maps the icon.
  - **Android:** `NotificationRowUIModel` does the same.
  - **Expected:** the shared renderer draws `GemNotificationRow`; both models go.
- **VM248** **S** **Price alert items are twinned.**
  - **iOS:** `PriceAlertItem` copies `GemPriceAlertItem` fields.
  - **Android:** `PriceAlertItemUIModel` does the same.
  - **Expected:** views read `GemPriceAlertItem`; both go.
- **VM249** **S** **The price-alert input is configured in the apps.**
  - **iOS:** `SetPriceAlertType` and `SetPriceAlertCurrencyInputConfig` pick placeholder "5", the "%" symbol, direction images and compose "Current price X".
  - **Android:** `PriceAlertTargetScene` shows "%" for percent alerts and composes "Current price **X**".
  - **Expected:** the alert session returns placeholder, symbol and the current-price text (land with VM191).

### Assets, NFT, rewards, support

- **VM250** **S** **Asset filters are held in the apps.**
  - **iOS:** `AssetsFilterViewModel` holds the balance toggle and chains and decides `isAnyFilterSpecified`.
  - **Android:** `BaseAssetSelectViewModel` holds the same filters.
  - **Expected:** a filter session in Core (as for activity) returns the state; both go.
- **VM251** **S** **Asset context menus are assembled in the apps.**
  - **iOS:** `AssetContextMenu` builds `GemAssetMenuInput` from which callbacks exist and maps actions to items.
  - **Android:** `assetContextMenuItems` does the same and picks the pin icon from state.
  - **Expected:** Core returns the menu rows with icon kinds; both builders go.
- **VM252** **S** **Asset-list rows are composed in the apps.**
  - **iOS:** `WalletSearchSceneViewModel` builds list rows with the count as subtitle.
  - **Android:** `AssetListRowUIModel` builds the same row and the list icon URL.
  - **Expected:** search returns list rows; both builders go.
- **VM253** **S** **Asset details sections are twinned.**
  - **iOS:** `AssetDetailRowItem`, `AssetDetailSectionItem` and `AssetDetailRowAction` copy `GemAssetDetails` sections.
  - **Android:** `AssetInfoUIModel`, `AssetInfoUIModelFactory`, `PriceAlertMenuUIModel` and `EmptyTransactionsUIModel` do the same.
  - **Expected:** views read the Core sections (taps from VM189); the twins go.
- **VM254** **S** **Recents are wrapped and grouped in the apps.**
  - **iOS:** `RecentAssetsModel`, `RecentsSceneViewModel` and `RecentAsset` hold and group recents.
  - **Android:** `RecentsSheetUIModel` and `RecentAsset` do the same and sort by date in the view.
  - **Expected:** Core returns the recents list sectioned (with VM210); the models go.
- **VM255** **S** **The collections grid is built in the app.**
  - **iOS:** `NFTGridPosterBuilder`, `GridPosterViewItem` and `CollectionsContent` map `GemNftEntry` to grid items and destinations.
  - **Android:** `NftListViewModels` builds the same grid.
  - **Expected:** views read the Core grid rows; the builders go.
- **VM256** **S** **Collectible details are reshaped in the apps.**
  - **iOS:** `CollectibleViewModel` decides which actions go to the menu or the buttons and labels them.
  - **Android:** `NftDetailsUIModel` copies sections and formats attribute dates.
  - **Expected:** a Core details record with placed actions; the models go.
- **VM257** **S** **Redemption rows compose the confirmation text.**
  - **iOS:** `RewardRedemptionOptionViewModel` composes the confirmation with `Localized.Rewards.confirmRedeem(value, points)`.
  - **Android:** `RewardRedemptionUIModel` composes it with `R.string.rewards_confirm_redeem`.
  - **Expected:** `GemRewardsRedemption` carries the confirmation text; both models go.
- **VM258** **S** **The incoming referral code is twinned on Android.**
  - **iOS:** `RewardsViewModel` switches on `GemIncomingCode` directly.
  - **Android:** `IncomingCodeUIModel` splits it into two nullable strings.
  - **Expected:** Android reads `GemIncomingCode` like iOS; the twin goes.
- **VM259** **S** **Support bubbles decide their side and flags.**
  - **iOS:** `SupportMessageBubbleViewModel` picks palette and alignment by sender and derives sending, failed and content flags.
  - **Android:** `SupportMessageBubble` does the same (`isUser` from the sender).
  - **Expected:** the Core bubble row carries side and outcome; palette from the style mapper; the models go.
- **VM260** **S** **WalletConnect connection rows are built in the apps.**
  - **iOS:** `ConnectionViewModel` and `ConnectionSceneViewModel` build the row and title.
  - **Android:** `ConnectionRowUIModel` picks icon or initial placeholder.
  - **Expected:** the shared renderer draws the Core connection row; both go.

### Shared components, second round

- **VM261** **S** **Asset screen options menus are composed in the apps.**
  - **iOS:** `AssetSceneViewModel.menuItems` builds "View address on X", "View token on X" and Share from the details' links.
  - **Android:** `AssetDetailsMenu.kt` builds the same three entries.
  - **Expected:** `GemAssetDetails` returns the menu rows (kind and link); the mapper supplies titles and icons; both builders go.
- **VM262** **M** **Screens count their own lists so Core can pick the phase.**
  - **iOS:** `SelectAssetViewModel` (`GemAssetSectionCounts`), `WalletSearchSceneViewModel` and `AssetsResultsSceneViewModel` (`GemWalletSearchCounts`), `PerpetualsSceneViewModel` (`GemPerpetualMarketCounts`), and `loadError(state:hasRows:)` in the asset, transactions, fiat transactions, price alerts, asset price alerts, notifications and support screens.
  - **Android:** `BaseAssetSelectViewModel`, `WalletSearchViewModel`, `AssetsResultsViewModel`, `PerpetualMarketViewModel`, `RecentsSheetUIModel` (`GemRecentsCounts`) and `loadError(..., candles.isNotEmpty())` in `PerpetualDetailsViewModel`.
  - **Expected:** the Core call that builds the screen reads the list sizes through the store port it already has and returns sections and phase together; the counting and the `hasRows` arguments go.
- **VM263** **S** **Asset list section titles are chosen in the apps.**
  - **iOS:** `SelectAssetViewModel` titles the popular section and `SelectAssetScene` shows popular, pinned and other sections by emptiness.
  - **Android:** `PinnedAssetsHeaderItem` picks "Popular" or "Pinned" and the icon by `AssetsGroupType`.
  - **Expected:** asset sections from Core carry their titles and kinds; the header choices go.
- **VM264** **S** **Search result sections are split in the apps.**
  - **iOS:** `WalletSearchSections.from` splits pinned and other assets (`AssetsSections.from`) and calls `perpetualMarketSections`; `WalletHomeState` re-assembles the home sections, header and flags.
  - **Android:** `WalletSearchViewModel` and `AssetsViewModel` split the same way; `WalletSummary` re-assembles the home state.
  - **Expected:** the search and home view states carry finished sections; the splitting types go.
- **VM265** **S** **Chain pickers each search chains themselves.**
  - **iOS:** `ChainListSettingsViewModel`, `ContactAddressEditorViewModel`, `ImportWalletTypeViewModel` (with its own empty-query branch) and `NetworkSelectorViewModel` (filters its list by matching ids) call the chain service separately.
  - **Android:** `ContactChainSelectViewModel`, `NetworksViewModel`, `SelectImportTypeViewModel`, `AddAssetViewModel` and `SelectFilterChain` do the same.
  - **Expected:** one Core chain list (rows plus search) that every picker uses, with VM199's chain rows.
- **VM266** **S** **Info sheets are twinned and their button rule is written twice.**
  - **iOS:** `InfoSheetModel` and `InfoSheetButton` copy `GemInfoSheet` and show the button for learn-more or when a handler exists.
  - **Android:** `InfoSheetEntity` and `InfoBottomSheet` apply the same rule.
  - **Expected:** the sheet views take `GemInfoSheet`; only the handler binding stays in the apps.
- **VM267** **S** **Simulation payload fields are mapped twice.**
  - **iOS:** `SimulationPayloadFieldViewModel` (with `SimulationPayloadFieldKind` and `models(for:)`) maps text, address and timestamp values and wires address taps.
  - **Android:** `SimulationPayloadFieldsContent` does the same per value case.
  - **Expected:** payload rows render through the shared row renderer (address rows from VM207); both go.
- **VM269** **S** **Banner destinations are routed per screen.**
  - **iOS:** `WalletSceneViewModel` opens only URL banners and ignores stake, activate and perpetual destinations; `AssetSceneViewModel` handles all four; both map banner buttons to header actions.
  - **Android:** `AssetsScreen` ignores the same three; `BannerItem` handles all four.
  - **Expected:** Core emits only destinations the screen can open and each app keeps one banner routing function.

### Texts composed in the apps

- **VM270** **S** **Delete confirmations are composed in the apps.**
  - **iOS:** `ChainSettingsSceneViewModel`, `WalletDetailScene` and `WalletsScene` compose "Delete X?".
  - **Android:** `NetworkScene` and `ConfirmWalletDeleteDialog` compose the same.
  - **Expected:** Core returns the prompt as a `GemLocalizedText` with the name; the mapper renders it.
- **VM271** **S** **Toast texts are composed in the apps.**
  - **iOS:** `ToastMessage+PrimitivesComponents` composes pinned/unpinned asset, price alerts enabled/disabled for an asset, and copied value.
  - **Android:** `AssetToasts` composes pinned/unpinned; `PriceAlertsNavScreen` composes "Price alerts enabled for X".
  - **Expected:** the Core call that changes the state returns its toast as `GemLocalizedText`; both builders go.
- **VM272** **S** **Amount screen texts are composed in the apps.**
  - **iOS:** `AmountSceneViewModel` composes "Balance: X" and "Reserved fees X".
  - **Android:** `PropertyAssetInfoItem` composes the balance and `AmountScene` the reserved fees.
  - **Expected:** the amount view state carries both texts (land with VM192).
- **VM274** **S** **Rewards invite and share texts are composed in the apps.**
  - **iOS:** `RewardsViewModel` composes the invite description with bold points and the share text with the link.
  - **Android:** `ReferralHead` bolds the points and `ReferralScene` composes the share text.
  - **Expected:** the rewards state carries both texts.
- **VM275** **S** **The app update prompt is composed in the apps.**
  - **iOS:** `RootSceneViewModel` builds the alert title, the version description and skip/update actions.
  - **Android:** `WalletApp` and `InAppUpdateBanner` compose the description and "version · …"; `AppViewModel` decides dismissal from `canSkip`.
  - **Expected:** `GemAppUpdateService` returns the prompt record (texts and which actions exist).
- **VM276** **S** **The "Today, 10:30" row date is composed in both formatters.**
  - **iOS:** `TransactionDateFormatter.row` joins the day label and the time with ", ".
  - **Android:** `SectionDateFormatter.row` does the same.
  - **Expected:** Core returns the day label with a localized template for label plus time; only the time formatting stays native.
- **VM277** **S** **The price-alerts toggle label is composed in the apps.**
  - **iOS:** `PriceAlertsSceneViewModel` composes "Enable price alerts".
  - **Android:** `PriceAlertScene` composes the same.
  - **Expected:** the price-alerts screen returns a toggle row from Core.

### Screen composition

- **VM278** **S** **Validator picker sections are titled in the apps.**
  - **iOS:** `ValidatorSelectSceneViewModel` titles "Recommended" and "Active".
  - **Android:** `ValidatorsScene` does the same.
  - **Expected:** `GemStakeValidatorOptions` returns titled sections.
- **VM279** **S** **The earn screen's rows are built in the apps.**
  - **iOS:** `EarnSceneViewModel` builds the "No data" and "Deposit" rows, decides the empty state and shows "Positions" only when positions exist.
  - **Android:** `EarnViewModel.depositListItem` and `EarnScreen` build the same.
  - **Expected:** the earn view returns its sections and rows.
- **VM280** **M** **Transaction details are composed per row kind in the apps.**
  - **iOS:** `TransactionSceneViewModel` maps each `GemTransactionDetailRow` kind to an item, writes "Swap again", rebuilds the fee amount and routes header taps; `TransactionItemModel` lists the cases.
  - **Android:** `TransactionDetailsViewModel`, `TransactionDetailsRowUIModel` and `TransactionDetailsScene` do the same.
  - **Expected:** the details sections carry finished rows (with VM209, VM211, VM212); both item layers go.
- **VM281** **S** **WalletConnect proposal rows are hardcoded in the apps.**
  - **iOS:** `ConnectionProposalViewModel` builds wallet, connection, status and the two permission rows.
  - **Android:** `ProposalSceneViewModel` builds the same four.
  - **Expected:** a Core proposal record returns the rows.
- **VM282** **S** **The perpetual market hides its header while searching in the views.**
  - **iOS:** `PerpetualsScene` gates the balance header on `!model.isSearching`.
  - **Android:** `PerpetualMarketScene` gates the header and sections on `isSearching`.
  - **Expected:** `GemPerpetualMarketSession` sections include or omit the header.
- **VM283** **S** **Settings screens hand Core the values its preference store holds.**
  - **iOS:** `PreferencesViewModel` builds `GemPreferencesInput` and `GemPerpetualDefaults` from stored values; `SecurityViewModel` builds `GemSecurityInput`.
  - **Android:** `PreferencesViewModel` and `SecurityViewModel` build the same inputs.
  - **Expected:** the settings services read preferences through their store port and return sections; the inputs go.
- **VM284** **S** **Recipients are built from contacts in each app.**
  - **iOS:** `RecipientSceneViewModel.contactRecipients` flattens contacts into `GemRecipient`s.
  - **Android:** `RecipientViewModel` does the same.
  - **Expected:** `recipientSections` reads contacts through the contact store; both flattenings go.

### Models and adapters

- **VM285** **S** **The fee asset of an asset screen is the chain coin in both apps.**
  - **iOS:** `ChainAssetRequest` loads `assetId.chain.assetId` as the fee asset.
  - **Android:** `GetChainAssetInfoImpl` does the same for tokens.
  - **Expected:** Core answers the fee asset for an asset; both stores read it.
- **VM286** **S** **iOS twins `GemSelectAssetType`.**
  - **iOS:** `SelectAssetType` and `SelectAssetSwapType` mirror the generated enum and map back through `flowType`.
  - **Android:** uses `GemSelectAssetType` directly.
  - **Expected:** iOS uses the generated enum; the twin and `flowType` go.
- **VM287** **S** **Amount routes twin `GemAmountRequest`.**
  - **iOS:** `AmountType` and `AmountInput` are rebuilt into a request in `AmountSceneViewModel`.
  - **Android:** `AmountParams` (and `toAmountParams`) is rebuilt into a request in `AmountViewModel`.
  - **Expected:** routes carry `GemAmountRequest` (land with VM192).
- **VM288** **M** **iOS re-wraps Core service methods to take Primitives types.**
  - **iOS:** `GemstonePrimitives/Sources/Services/*.swift` (wallet, wallet session, WalletConnect, contacts, wallet home, perpetual details, swap quote, and others) and `GemConfirmMetadata+GemstonePrimitives` wrap Core calls with conversions.
  - **Android:** calls Core with `toGem()` at each call site.
  - **Expected:** callers use the generated types directly; the wrapper extensions go.
- **VM289** **S** **Android domain aggregates wrap Core rows.**
  - **iOS:** uses `GemPerpetualMarketItem` directly in views (its wallet-row twin `WalletEntry` is VM237).
  - **Android:** `PerpetualDataAggregate`, `PerpetualPositionDataAggregate(Impl)`, `WalletDataAggregate`, `WalletSummary`, `LeverageState` and `NftAssetDetailsData` (`gemcore/.../domains`) wrap them.
  - **Expected:** Android uses the records directly; the aggregates go.

### Scenes

- **VM290** **M** **Empty states are decided scene by scene.**
  - **iOS:** `FiatTransactionsScene`, `InAppNotificationsScene`, `PriceAlertsScene` and `TransactionsScene` show the empty view when the list is empty and there is no error; `ContactsScene`, `ConnectionsScene`, `ChainListSettingsScene`, `ValidatorSelectScene`, `WalletImageScene`, `CurrencyScene`, `ImportWalletTypeScene` and `CollectionsScene` check emptiness themselves; `AssetPriceAlertsViewModel.showsEmpty`, `EarnSceneViewModel.showsEmptyState` and `PerpetualsPreviewViewModel.hasNoPositions` decide it in the model.
  - **Android:** `TransactionsScene`, `ConnectionsScene`, `FiatTransactionsScene`, `EarnScreen`, `StakeScene`, `ContactsNavScreen`, `InAppNotificationsScene`, `NftListScene` (empty and no unverified row), `CurrenciesScene` (only while a query is typed, unlike iOS), `PerpetualMarketScene`, `ValidatorsScene`, `PriceAlertScene`, `NetworkAssetsScreen`, `SelectChain`, `WalletImageScene` and `PerpetualsPreviewSection` do the same.
  - **Expected:** each list screen's Core phase says it is empty and which empty kind to show, as `GemSelectAssetState` already does; the scenes render the phase (land with VM262).
- **VM291** **S** **Whether a wallet can be chosen is decided in the apps.**
  - **iOS:** `RewardsViewModel.showsWalletSelector` checks `wallets.count > 1`.
  - **Android:** `ReferralNavScreen` checks `availableWallets.size > 1`; `AuthRequestScene` does the same for WalletConnect authentication.
  - **Expected:** the rewards state and the WalletConnect request say whether a wallet can be chosen; the counts go.
- **VM292** **S** **The rewards screen's intro and action placement are written in the apps.**
  - **iOS:** `RewardsScene` lists the three intro features with their emojis and titles and places share or create-code, use-code and the pending referral.
  - **Android:** `ReferralHead` lists the same three features and `ReferralScene` picks the first share or create-code action, then use-code and the pending referral.
  - **Expected:** the rewards state returns the intro items and the placed actions.
- **VM294** **S** **Secret phrase rows are filled with words in the apps.**
  - **iOS:** `SecretPhraseRow` and `GemSecretPhraseRow+PrimitivesComponents` map Core's index rows to words.
  - **Android:** `PhraseWord.phraseRows` does the same.
  - **Expected:** Core returns the rows with their words; both mappings go.
- **VM295** **S** **Android re-checks the swap provider count.**
  - **iOS:** shows the provider picker when `allowSelectProvider` is set.
  - **Android:** `SwapDetailsComponents` lists providers only when `providers.size > 1` on top of `allowsProviderSelection`, which already requires more than one quote, and picks the section title itself.
  - **Expected:** Android relies on the Core flag and the title comes from the mapper.

### Twins and dead code

- **VM195** **L** **Each app hand-writes its own asset read model and converts it for Core.** Core has `Asset`, `Price`, `AssetMetaData` and `GemAssetBalance`, but the composite a store read returns is written twice, with different shapes.
  - **iOS:** `AssetData`, `ChainAssetData`, `Balance`, `AssetValuePrice` and `RecentAsset` (`Primitives`), converted by `GemAssetBalance(_:assetId:isActive:)` and `AssetData.rowInput` (`ListAssetItemsViewModel.swift`).
  - **Android:** `AssetInfo`, `ChainAssetInfo`, `Balance`, `AssetBalance`, `AssetPriceValue` and `RecentAsset` (`gemcore/.../model`), converted by `AssetBalance.toGem` and `AssetInfo.rowInput` (`AssetInfoDataAggregate.kt`), which also drops non-finite prices where iOS does not.
  - **Expected:** one generated read model from Core primitives that both store adapters fill, taken directly by Core's row functions; both sets of twins, conversions and row-input builders go.
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

- **BD299** **S** **Android loses an NFT's resource when it stores it.**
  - **iOS:** `NFTAssetRecord` stores `resourceUrl` and `resourceMimeType` and reads them back into `NFTAsset.resource` (`ios/Packages/Store/Sources/Models/NFTAssetRecord.swift`).
  - **Android:** `DbNFTAsset.toAssetModel` returns `resource = NFTResource("", "")` and the preview image with an empty mime type (`data/services/gemstone/.../nft/NftModels.kt`), so a stored NFT has no resource.
  - **Expected:** Android stores and returns the resource URL and mime type as iOS does (Room migration for the two columns); lands before GEN300 so one mapping spec serves both apps.


## 9. Behavior differences

Differences between the apps, or between an app and the server, each with its decision.

### Same rule, different answers

- **BD29** **S** **Redemption options with unlimited stock are never listed.** `summary.rs:18` (`remaining.unwrap_or_default() > 0` drops `None`) vs storage `rewards_redemptions_repository.rs:48,96` and Core `rules.rs:146` (`None` = unlimited). Needs a decision: `test_available_redemption_options` pins a `None` stock as hidden, so confirm whether options stored without a stock are meant to be offered before changing the server.

### Freshness

- **BD50** **S** **Public `/chain/fee-estimates` can serve very old estimates.** `core/crates/services/src/chain/fee_estimates_client.rs:73-76` has no freshness check (TTL 5 years, `core/crates/cacher/src/keys.rs:155`); the per-chain route refreshes (`:55-70`). Used by the website, not the apps. **Needs a decision (2026-09-24):** either the public route drops entries past the one-hour fresh key (the website loses chains nobody requested lately) or it refreshes them (a public route then triggers node calls).

## 10. Generated mocks and mappers

Test mocks for generated types are written by hand three times, once per language, and have drifted: `Asset.mock()` is Bitcoin on iOS and Android but Ethereum in Rust; `Transaction` is incoming with value `"0"` on iOS and outgoing with `"1"` on Android; `Account.address` is empty on iOS and `"wallet-address"` on Android; dates are `.now`, `Date()` or the epoch. `core/bin/generate` already reads every type in `remote_types.yml` from the primitives source into records, enums and fields, writes the Swift and Kotlin mappers from that model and checks them against golden files; the mocks come from the same model.

**The standard, for every item below.** A type gets a mock only when `remote_types.yml` lists it under `mocks:`; no type is mocked because it could be. Each listed type gets exactly one mock, the same on both apps: iOS `Type.mock(...)`, Android `mockType(...)` (each app keeps its existing call shape). All of a package's data mocks live in one file, never one file per type: the generated ones in `GeneratedMocks.swift`/`GeneratedMocks.kt`, and the few the generator cannot reach in one hand-written `Mocks.swift`/`Mocks.kt` beside it (distinct names, since Swift rejects two files of the same name in a target and Kotlin two `MocksKt` facades in a package). Behavioural doubles (service fakes, stores) are classes and keep their own files. Every field is a parameter with a default, in declaration order. Defaults follow one rule table, identical on both apps: `""` for a string, `0` for a number and big integer, `false`, `nil`/`null` for an optional, `[]` for a list, the first variant for an enum and for `Chain` (Bitcoin), the epoch for a date, the listed type's own mock for a nested record and the identifier's hand-written mock for an identifier. A field whose type has no mock and no rule fails generation and names the field, so the fix is to list its type or add a per-field override under `mocks:`; an override exists only where the rule would build an invalid value, never for readability, and applies to both apps. There are no named presets (`mockEthereum()`, `mockAssetSolanaUSDC()`, `mockWithChains()`): a test passes the values it asserts on, so every input it depends on is visible at the call site. Rust keeps its hand-written `mock()` and `Type { field, ..Type::mock() }` overrides (`core/skills/tests.md`); the generator does not write Rust mocks.

- **GEN297** **M** **Both apps' existing mocks move onto the generated ones and agree.** Work type by type, one family per change (assets, transactions, wallets and accounts, perpetuals, NFT, fiat, support, WalletConnect).
  - **Generated types:** add the type to `mocks:`, delete its hand-written mock on both apps, and rewrite the call sites whose shape changed. Android's `mockAsset(chain, tokenId, …)` becomes `mockAsset(id = mockAssetId(…), …)`. iOS mocks that hard-code or derive fields (`Account` derivation path, `Transaction` contract, block number and `feeAssetId` copied from `assetId`, `TransactionExtended` prices, `Wallet.isPinned`, `PerpetualPosition` direction from size, `WalletConnectionSession` id from session id, `Device` and `BlockExplorerLink` with no parameters) become the generated all-field mock.
  - **Presets:** delete every named preset and write its values at the call site: iOS 27 presets used at 377 call sites (`mockEthereum` 74, `mockWithChains` 50, `mockEthereumUSDT` 41, `mockAssets` 41, `mockBNB` 28, `mockTron` 24) and Android's asset presets (`mockAssetSolana`, `mockAssetEthereumUSDT`, `mockAssetHyperCoreUSDC`, …), `mockWalletMulticoin` and `mockAmountParamsTransfer`/`mockAmountParamsPerpetual`.
  - **Tests that relied on a default:** a test that asserts on a value it did not pass now passes it, e.g. `AssetViewModelTests.swift:11` ("BTC"), `TransactionSceneViewModelTests.swift:194` ("Bitcoin"), `RewardsViewModelTests.swift:48` ("test123"). Never add an override to keep such a test green.
  - **What stays hand-written, aligned across the apps:** identifiers (`AssetId`, `WalletId`, `TransactionId`, `NFTAssetId`, `NFTCollectionId`, `PerpetualId`, hand-written on both apps) keep one mock each with the same parameters and the iOS defaults (`AssetId.mock()` is Bitcoin native, which matches the rule table); hand-written app types (iOS `AssetData`, `Balance`, `AssetValuePrice`; Android `AssetInfo`, `AssetBalance`, `Session`) follow the standard until VM195 replaces them; behavioural doubles (service mocks implementing `Gem*ServiceProtocol`/`Gem*Interface`, stores, `PasswordStoreMock`) are not data mocks. Where two hand-written mocks for the same concept differ, iOS is the reference.
  - **One file:** the hand-written data mocks left in `Primitives/TestKit`, `GemstonePrimitives/TestKit` and `gemcore` `testFixtures` move into that package's single `Mocks.swift`/`Mocks.kt`, and the per-type files (`Type+PrimitivesTestKit.swift`, `Type+GemstonePrimitivesTestKit.swift`, `TypeMock.kt`) go.
  - **Done when:** no hand-written mock remains for a type `remote_types.yml` can list, each package has at most one generated and one hand-written data mock file, both apps' mocks of a type take the same parameters with the same defaults, and no `mock<Name>()` preset remains.
- **GEN298** **M** **Mocks for Gemstone's own records.** The generator reads only `crates/primitives`; the UniFFI-side mocks (iOS `GemstonePrimitives/TestKit`: 9 plain, 15 partial, 5 with presets; Android `gemcore` `testFixtures`: 49 functions for `uniffi.gemstone` types) are hand-written and disagree the same way. Extend the parser to the `#[derive(uniffi::Record)]` and `#[derive(uniffi::Enum)]` types in `core/gemstone/src` and write them with UniFFI naming (Kotlin unit-enum variants in upper case, data enums as sealed classes) into `ios/Packages/GemstonePrimitives/TestKit/GeneratedMocks.swift` and the Android `GeneratedMocks.kt` (Android keeps one generated file for both type systems, as `gemcore` `testFixtures` holds both), under the same standard and `mocks:` list. iOS mocks that take a `Primitives` value and convert it (`GemHeaderAmount.mock(asset: Asset)`, `PerpetualConfirmData.mock(baseAsset: Asset)`, `GemConfirmLoadMock.swift`) take the generated type instead, and their call sites change. Records that app tests build inline without a mock (`GemSimulationPayloadRow`, `GemFiatQuoteRequest`, `GemPerpetualButtonRow`, `GemBalanceRequirement` on iOS; `GemBalanceRecord`, `GemPerpetualChartLayout`, `GemStakeViewState` on Android) are listed when a second test needs them.
- **GEN300** **L** **Both apps' persistence mappers are generated from one spec.** Each app hand-writes the record ↔ model mapping for the same tables: iOS about 66 mapping members (about 780 lines) in `ios/Packages/Store/Sources/Models/*Record.swift`, Android about 49 mappers (about 650 lines) in `android/data/services/store/.../database/entities/Db*.kt`, about 60–65% of them plain field copies. They drift (BD299), and Android builds the same record in several places: four `DbAsset` builders (`AssetFull.toRecord`, `Asset.toRecord`, `AssetBasic.toRecord`, `AssetBasic.toUpdateRecord`), `DbPerpetual.toUpdate` duplicating `toDB`, and two `DbPrice.toAssetPrice` that disagree on a missing price (`stores/StoreModels.kt` returns 0, `DbTransactionExtended.kt` drops it).
  - **Expected:** a `records:` section in `remote_types.yml` names each record or entity, the model it maps and the few field rules a mapping needs (rename, flatten a nested record into prefixed columns, integer cast, parent key); the generator writes both directions per app (`Store/Sources/Generated/RecordMappers.swift`, `data/services/store/.../generated/EntityMappers.kt`), and the hand-written copies go. The record and entity declarations and schema stay hand-written. Start with the plain copies (`Account`, `AddressName`, `AssetLink`, `Contact`, `ContactAddress`, `SupportMessage`, `PriceAlert`, `Perpetual`, `PerpetualPosition`, `FiatRate`, `Banner`, `InAppNotification`), then the flattened ones (`Asset`, `NFTAsset`, `NFTCollection`, `FiatTransaction`, `WalletConnection`, `Balance`); mappings with real logic stay hand-written (`Transaction` id split, `AssetMarket` reassembly, `Price` positive check, `Node` status). Land after BD299.

## 11. Module layout

A feature module is one product area, and both apps give it the same name. iOS groups by product area and is the reference; Android splits many areas into one module per screen and names several differently (`activities`, `bridge`, `buy`), and its `assets` is the wallet tab while iOS `Assets` is the asset screens.

**The standard, for every item below.** Names and packages follow [Cross-Platform Awareness rule 7](../skills/cross-platform-awareness.md); on Android that means no `views`, `navigation` or `details` package roots and no singular `viewmodel`. A move renames the Gradle path in `settings.gradle.kts`, every `project(":features:…")` dependency and the imports, and changes no behaviour. One module per change. Verify with `cd android && ./gradlew assembleGoogleDebug test` for an Android move, `cd ios && just build` and `just test-package <Package>` for an iOS rename.

- **MOD301** **S** **Wallet list and wallet details are one module.**
  - **iOS:** `ManageWallets` holds `WalletsScene` and `WalletDetailScene`.
  - **Android:** `features/wallets` holds the list; `features/wallet-details` (the only hyphenated module, package `features.wallet`) holds the detail, secret data and wallet image screens, and `wallets:presents` depends on it.
  - **Expected:** one Android `wallets` module; iOS `ManageWallets` becomes `Wallets`.
- **MOD302** **S** **Android `activities` becomes `transactions`**, as iOS `Transactions` and [transactions.md](product/transactions.md).
- **MOD303** **S** **Android `bridge` becomes `wallet_connector`**, as iOS `WalletConnector`.
- **MOD304** **M** **Rewards is its own module on both apps.**
  - **iOS:** `RewardsScene`, its view models and `RewardsTests` live in the `Settings` package.
  - **Android:** `features/referral`, packages `referral.views` and `referral.viewmodels`.
  - **Expected:** iOS `Rewards` and Android `rewards`, as [rewards.md](product/rewards.md).
- **MOD305** **S** **Android `perpetual` becomes `perpetuals`**, as iOS `Perpetuals` and [perpetuals.md](product/perpetuals.md).
- **MOD306** **S** **Android `buy` becomes `fiat_connect`**, as iOS `FiatConnect`; it already holds sell and the fiat history.
- **MOD307** **S** **The wallet tab is `wallet_tab`.**
  - **iOS:** `WalletTab` holds `WalletScene`, `NetworkAssetsScene`, `WalletSearchScene`, `AssetsResultsScene` and `PortfolioScene`.
  - **Android:** `features/assets` holds the same screens except the portfolio, whose `PortfolioChartScene` is in `features/asset`.
  - **Expected:** Android `assets` becomes `wallet_tab` and takes `PortfolioChartScene`; the `update_app` module folds into it, its only consumer. Lands before MOD308, which reuses the name `assets`.
- **MOD308** **M** **The asset screens are one `assets` module.**
  - **iOS:** `Assets` holds `AssetScene`, `AddAssetScene`, `SelectAssetScene`, `AddressDetailsScene` and `AssetsFilterScene`.
  - **Android:** split over `features/asset`, `features/add_asset` and `features/asset_select`; `swap`, `perpetual`, `settings/price_alerts` and the wallet tab depend on `asset_select`.
  - **Expected:** one Android `assets` module; the dependents point at it.
- **MOD309** **M** **Send, receive and amount are one `transfer` module.** Lands with MOD310.
  - **iOS:** `Transfer` holds `RecipientScene`, `AmountScene`, `ReceiveScene`, `ConfirmTransferScene`, `PaymentVerificationScene` and `GetAssetScene`.
  - **Android:** split over `features/recipient` (package `recipient.viewmodel`), `features/transfer_amount`, `features/receive` and `features/confirm`; `transfer_amount` also holds `ValidatorsScene`.
  - **Expected:** Android `recipient`, `transfer_amount` and `receive` become one `transfer` module; `ValidatorsScene` moves to `stake`; `confirm` stays a module of its own (MOD314).
- **MOD310** **S** **Android `earn/stake` and `earn/delegation` become one `stake` module**, as iOS `Stake` (`EarnScene`, `StakeScene`, `DelegationScene`, `ValidatorSelectScene`). Their namespaces are `features.stake` and `features.earn.delegation`, which do not match the path; the `earn` level goes.
- **MOD311** **M** **Onboarding is one module.**
  - **iOS:** `Onboarding` holds `OnboardingScene`, `AcceptTermsScene`, the create, import and verify-phrase flows, `SecurityReminderScene`, `ShowSecretDataScene` and `WalletImageScene`.
  - **Android:** `features/create_wallet`, `features/import_wallet`, and `OnboardScreen` and `AcceptTermsScreen` in `app/features/onboarding`.
  - **Expected:** one Android `onboarding` module holding all of them except the secret data and wallet image screens, which belong to Wallets (MOD315).
- **MOD312** **M** **Settings is one module, and the areas with their own page leave it.**
  - **iOS:** `Settings` holds settings, preferences, security, notifications, appearance, currency, networks (`ChainSettings`), about us and developer; `Contacts`, `PriceAlerts`, `InAppNotifications` and `Support` are packages of their own.
  - **Android:** nine modules under `features/settings` (`aboutus`, `contacts`, `currency`, `develop`, `in_app_notifications`, `networks`, `price_alerts`, `security`, `settings`), with `SupportChatScene` inside `settings/settings`.
  - **Expected:** Android `aboutus`, `currency`, `develop`, `networks`, `security` and `settings` become one `settings` module; `contacts`, `price_alerts` and `in_app_notifications` move to the top level; `SupportChatScene` moves to a `support` module.
- **MOD313** **S** **Android names a screen's view-model-bound entry one way.** 33 entries end in `Screen` and 16 in `NavScreen` (`TransactionsNavScreen`, `FiatNavScreen`, `ContactsNavScreen`, `WalletNavScreen`, …) for the same role; the stateless composable is `XScene` on both apps. Rename the `NavScreen` ones to `Screen`, each with its module's move where it has one.
- **MOD314** **M** **Confirmation is its own `Confirm` module on both apps.** Transfer, swap, stake, perpetuals and WalletConnect all end on it, and it is where signing happens.
  - **iOS:** the 12 confirm files (`ConfirmTransferScene`, `ConfirmTransferSceneViewModel`, `Confirm*ViewModel`, `ConfirmTransferRequest`, `ConfirmTransferSection`, `ConfirmTransferState`, `ConfirmTransferSheetType`, `ConfirmSimulationState`) are in `Transfer`, which imports `WalletConnector` and `Swap` only for them; the app opens the screen (`ConfirmTransferNavigationView`).
  - **Android:** `features/confirm` is its own module, but `bridge` and `perpetual` build `ConfirmScreen` themselves, and `perpetual` uses `AcquireAssetAction` from it (iOS `GetAssetAction`).
  - **Expected:** iOS moves the confirm files into a `Confirm` package and `Transfer` drops the `WalletConnector` and `Swap` imports; Android `bridge` and `perpetual` stop importing `ConfirmScreen` and the app opens it, as on iOS.
- **MOD315** **M** **The secret data and wallet image screens belong to Wallets on both apps**, as [wallets.md](product/wallets.md).
  - **iOS:** `ShowSecretDataScene`, `ExportWalletNavigationStack` and `WalletAvatar` (`WalletImageScene`) are in `Onboarding`, so `ManageWallets` depends on `Onboarding`; the wallet image is used only from `WalletsNavigationStack`, and create-wallet reuses `ShowSecretDataScene` for the new phrase.
  - **Android:** `WalletSecretDataNavScreen` and `WalletImageScene` are in `wallet-details`; create-wallet draws the phrase with its own `WordChip` view.
  - **Expected:** iOS moves the export flow and `WalletAvatar` to `Wallets` and drops its `Onboarding` dependency. The grid of phrase words becomes one shared component (`PrimitivesComponents`, `ui`) that create-wallet and export use on both apps. Lands after MOD301.
- **MOD316** **S** **Recents belong to Assets, and the chart is a `Market` module on both apps.**
  - **iOS:** recents are the `Recents` package, used by `Assets`, `Perpetuals` and `WalletTab`; the chart is `ChartScene` in `MarketInsight`.
  - **Android:** `RecentsBottomSheet` is in `features/asset_select`, used by the same three areas; `AssetChartScene` and its view model are under `chart` in `features/asset`.
  - **Expected:** iOS folds `Recents` into `Assets`, and `Perpetuals` and `WalletTab` depend on `Assets`; Android keeps recents in `assets` (MOD308). iOS `MarketInsight` becomes `Market`; Android moves `asset/presents/chart` and `asset/viewmodels/chart` into a `market` module.
- **MOD317** **S** **The QR scanner is a feature on both apps; banners belong to Assets and the info sheet is shared UI.**
  - **iOS:** `QRScanner` and `InfoSheet` are feature packages; `BannerView` is in `PrimitivesComponents`, used by `AssetScene` and `WalletScene`.
  - **Android:** the QR scanner and the info sheet are in `ui/components`; banners are the `banner` feature, used by `asset` and the wallet tab.
  - **Expected:** Android moves `QRScanner.kt` into a `qr_scanner` feature; `BannerView` moves into iOS `Assets` and the Android `banner` module folds into `assets`, with `wallet_tab` depending on `assets`; iOS `InfoSheet` moves from `Features/` to `Packages/`.

- **MOD320** **L** **Android observed reads are requests, as on iOS** ([ARCHITECTURE § 5](ARCHITECTURE.md#android)). One area per change; each change deletes the case, its `Impl` and its Hilt binding, adds the request with an in-memory Room test beside [`PriceAlertsRequestTest`](../android/data/services/store/src/test/kotlin/com/gemwallet/android/data/services/store/integration/PriceAlertsRequestTest.kt), and matches the iOS request's query; where the Android read differs, iOS is the reference and the change says so.
  - **iOS:** each observed read is one `Store/Requests` type observed through `ObservableQuery`.
  - **Android:** about 75 interfaces in `gemcore/application/<area>/cases/` with 58 implementations in `data/coordinators/`. Classify each while migrating: an observed read (`GetWallets`, `GetTransactions`, `GetWalletAssets`, `GetSession`, `GetContacts`, …) becomes a request; a command that only forwards a Core call (`SetCurrentCurrency`, `DeleteWallet`, `PairWalletConnect`, …) is deleted and the view model calls the service; a platform port (`PasswordStore`, `ShowSystemNotification`, `NotificationPermissionRequests`, `WalletConnectClient`) stays an interface. The feature modules that reach `:data:services:gemstone` directly move in their area's change: `GemstoneAssetStore` in `NetworkAssetsViewModel` (a store held by a view model), `RecentAssetsService` (its writes go through `GemRecentActivityService`), `AssetsSearchService`, `UserConfig` and `ConnectionStatusObserver`.
  - **Expected:** a feature view model holds one Core service, the requests it observes and platform ports; `data/coordinators` holds only port implementations and Core composition.
- **MOD321** **S** **`check-boundaries.py` keeps Android features off the data internals.** No `features/**` Gradle file depends on `:data:services:gemstone` or `:data:coordinators`. The rule starts from the modules that still do, each named, and fails when a module joins the list or a listed module no longer needs its entry. Lands with the next MOD320 change.

## 12. Cleanup sweeps

Two passes over the whole repository, Core, iOS and Android, run after the items above have moved their rules and deleted their models. Each goes one platform and one module family per change, builds and tests that module before moving on, and says in the commit what it removed. Neither pass removes something an open item already names (that item deletes it with its replacement), and neither touches a public contract: API routes and fields shipped apps or the website read (see X168), stored formats, database and keystore migrations, and deep link URLs stay until their own item retires them. Changes near key material, signing or transaction construction follow [security](../skills/security.md).

- **CLN318** **L** **Unused and redundant code is removed on every platform.** Dead code is still compiled, generated, reviewed and copied by the next change that reads nearby code.
  - **Unused:** a declaration nothing in production reaches. One reached only by its own test, a mock or a preview counts as unused, and its test goes with it (as `String.words()` went). Find candidates per platform, then confirm each by search and a build:
    - **Core:** `just core unused` (cargo machete) for dependencies; `pub` items with no caller outside their crate, which rustc does not flag; `scripts/check-ffi-surface.py` for exports no app calls, shrinking its `ALLOWED` list to entries that name an open item; TypeShare types neither app nor the web reads; unused `CacheKey` variants, config fields and features.
    - **iOS:** a Periphery scan of the workspace for unused declarations, protocols and conformances; package dependencies in `Package.swift` no source imports; `Style` image assets nothing references.
    - **Android:** Android Lint `UnusedResources`; top-level functions, extensions, DAO queries and classes with no caller; Gradle dependencies no module source uses. Keep what the manifest, Hilt, WorkManager or reflection reaches.
    - **Both apps:** Fluent keys in `localization/` that neither app reads (a key is unused only when both apps drop it), then `just localize`.
  - **Redundant:** two helpers, formatters, extensions or fixtures in one platform that do the same thing keep one, and callers move to it. The twin of a Core record belongs to its `VM` item, not here.
  - **Done when:** each tool reports nothing, or every remaining hit is listed in the commit with the reason it stays.
- **CLN319** **L** **Every unit test is reviewed, and the ones that protect no contract go.** Last in the order: before it, the `VM` items move rules into Core and the `GEN` items replace mocks, which changes which app tests still mean anything. Unit tests only; integration tests and Maestro flows keep their own rules ([testing-maestro](../skills/testing-maestro.md)).
  - **Delete a test that:** asserts a constant, static table or 1:1 enum mapping back at itself; tests generated code (TypeShare models, `RemoteTypeMappers`, generated mocks), which the generator's golden tests cover; re-tests in an app a rule Core owns and tests (the app keeps a test only for wiring and visible output); repeats another test's case in the same platform with no new boundary; covers a getter, `copy`, equality or framework behaviour (SwiftUI, Compose, GRDB, Room) rather than ours; mocks the very code it claims to check; or still passes when the rule it names is inverted.
  - **Keep, and fix rather than delete:** a weak test that guards a real contract gets the assertion the contract needs. Signing, keystore, derivation, transaction construction, amounts and decimals, address validation, wire formats and migrations are never deleted without an equivalent test in the same change, whether in Core or the app.
  - **Check:** for each test kept on a business rule, invert the rule, run the test, confirm it fails, and restore ([engineering principles](../skills/engineering-principles.md#tests)). A contract found with no test gets the smallest one at its owner.
  - **Run:** `cd core && just test <CRATE>`, `cd ios && just test-package <Package>` (confirm the target is still in the test plan and the expected tests ran), `cd android && ./gradlew :<module>:testDebugUnitTest`. Record the test count per module before and after in the commit.
  - **Done when:** every Core crate, iOS package and Android module has been reviewed once.

## Blocked upstream

- **X168** **S** `WalletConfiguration.multi_signature_accounts` ([`wallet_configuration.rs`](../core/crates/primitives/src/wallet_configuration.rs)) is the old name of `externally_controlled_accounts`, which also covers Solana accounts assigned to another program. The API fills both because shipped apps read only the old field. Delete the field, its fill in [`wallet_configuration.rs`](../core/crates/services/src/devices/wallet_configuration_client.rs) and the merge in [`externally_controlled_banners`](../core/gemstone/src/services/wallet_configuration/rules.rs) on 2026-12-18, three months after the release that reads `externally_controlled_accounts`.
- **X163** **M** iOS pins the `Gemstone` package to Swift 5 language mode. Re-tested on 2026-09-16 against uniffi 0.32.1: both `uniffiTraitInterfaceCallAsync` sites still fail with "passing closure as a 'sending' parameter" because the generated `Task { }` captures three `@escaping` non-`Sendable` parameters. Nothing to decide and nothing to do until a uniffi release changes that function; re-test then. Rechecked on 2026-09-22 with `cargo search uniffi`: 0.32.1 is still the latest uniffi release, and the fix is already merged upstream as uniffi #2929 (`694fda6a05`, 2026-07-15, marks the async trait-interface closures `@Sendable`) but is not in 0.32.1. Bump uniffi to the first release that contains #2929, regenerate, and move the `Gemstone` package to Swift 6 mode.
