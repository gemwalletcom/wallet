# Open work

The goal is that Gemstone decides once and both clients read that decision. Shared rules, records, sessions and orchestration live in Core; rendering, observation, scheduling, OS ports and locale formatting live in the apps. A view model, UI model, aggregate, factory or provider that still decides, re-derives, composes text or formats a number is debt; a model that holds one Core record and maps it to platform values is the target shape. Contracts and worked examples are in [ARCHITECTURE.md](ARCHITECTURE.md).

Every open item carries a stable id and a size (**S**/**M**/**L**); ids are never reused. Namespaces: `VM` (consolidation into Gemstone), `AUD` (correctness found by review), `BD` (a behaviour difference), `D` (a decided product or security choice being built), `GEN` (code `just generate-models` writes instead of hand-written test mocks and mappers), `MOD` (feature modules named and grouped the same way on both apps), `NAM` (types inside a feature named the same way on both apps), `CLN` (a cleanup sweep across every platform) and `X` (blocked upstream).

- **Closing an item:** delete its line and commit at once. If it settles an important user-facing rule someone might simplify away, write that rule on its area page under [product/](product/) in the same commit, following [PRODUCT.md § Writing a page](PRODUCT.md#writing-a-page); most items have none. Nothing else is recorded here: git history carries the detail.
- **Comparing the apps:** say what each does on its own line — **iOS:**, **Android:**, then **Expected:** (the agreed behaviour, or "needs a decision") — never in one sentence covering both. When the apps differ and no item decides otherwise, iOS is the reference behaviour.

## Working one item

Pick one item by its stable id and use the screen map to find its current owner. Source paths and line numbers are retrieval hints, not proof that an old finding still applies. Read the owner, both consumers and the linked architecture example before editing.

Keep the implementation plan in the task update: the existing owner to change, the app path to delete, any real prerequisite and the exact relevant checks. Do not create a new planning artifact, service, session, cache or framework unless the current contract requires it. If a pure projection or existing record answers the question, use it. Preserve examples in the docs and correct them in place when signatures change. Check related ids before changing a shared owner: reuse one contract change and its regression fixtures rather than implementing competing fixes. Items that name each other ("land with …") share a record; land them in one change or in dependency order.

Use [Task Workflow](../skills/task-workflow.md) for execution and [Quality Checks](../skills/quality-checks.md) for verification. Implement ready work autonomously; an item that says "needs a decision" waits for the owner, and [Blocked upstream](#blocked-upstream) records external prerequisites. Shared infrastructure work gates only the consumers that need it; unrelated ready items need not wait.

## Ready next

These need no further answer; work them in this order, one family per change.

1. **App models to Core records:** VM262 to VM289 (second round) area by area as grouped in section 5, then VM290 to VM295 (scenes) in the same way.
2. **Generated mappers:** BD299, then GEN300.
3. **Unused code:** CLN318.
4. **Parity:** BD373 to BD376.
5. **Unit test review, last:** CLN319, after every other ready item, so it reviews the tests that remain once rules have moved into Core.

Waiting on the owner: BD29 and BD50 (server), VM79, VM181, VM183, D175 (on hold). Waiting on a date or a release: X168, X163.

## Screen coverage and existing infrastructure

This map routes work to current owners. It groups existing ids rather than creating a new service or task for every view. Shared components and platform-only features keep their established boundaries; the service map in [ARCHITECTURE.md](ARCHITECTURE.md#screen-services) supplies the current app callers.

| Screens / entry points | Existing owner or infrastructure to extend | Open work |
|---|---|---|
| App start, foreground, wallet switch, deep links and pushes | `GemAppStartService`, `GemWalletSessionService`, `GemNavigationService`, `GemAppUpdateService`, native lifecycle hosts | VM79 |
| Create/import wallet, terms, phrase generation and verification | `GemWalletService`, `GemVerifyPhraseSession`, `phrase_suggestions`, keystore and native auth ports | VM183, VM294 |
| Wallet list/detail, rename, avatar, secret export | Wallet rows/details, existing export flow and NFT avatar selection | VM294 |
| Wallet home, header, network assets, banners | `GemWalletHomeService`, `GemBalanceService`, `GemBannerService`, shared asset rows and banner context | VM264, VM269 |
| Asset search/select, add token, recents | `GemAssetSelectionService`, `GemSelectAssetFlow`, `GemAddAssetService`, recent activity | VM262, VM286 |
| Asset details and asset actions | `GemAssetDetailsService`, shared rows, copy, info and load state | VM285 |
| Portfolio chart/statistics | `GemPortfolioService`, chart load rules, shared numbers and rows | — |
| Asset chart/market/alerts sections | `GemChartService`, `GemChartSession`, shared list renderer | — |
| Receive, QR display, address details | `GemReceiveService`, `GemAddressDetailsService`, `GemCopy`, payment encoding | Retain existing native QR/share adapters |
| Scanner, payment links, deep links and pushes | Existing payment decoder, `GemPaymentService`, push/navigation preparation | VM284 |
| Amount entry, fiat equivalent, amount extras | `GemAmountService`, `GemAmountRequest`, `GemAmountEntry`, `GemAutocloseDraft` | VM287 |
| Confirmation, fees, simulation, acquisition | `GemConfirmTransferService`, `GemConfirmation`, `GemConfirmScreen`, shared headers/rows/info | VM267 |
| Swap, providers, slippage and swap details | `GemSwapQuoteService`, `GemSwapSession`, `GemSlippageSession` | VM295 |
| Activity, asset/position history, transaction details | `GemTransactionsService`, `GemTransactionDetailsService`, detail records, native indexed queries | VM280, VM290 |
| Buy/sell quotes, provider opening, fiat history | `GemFiatQuoteService`, `GemFiatSession`, existing fiat transaction owner | — |
| Perpetual market list/search/pins and balance | `GemPerpetualService`, market session/rows, native search indexes | VM282, VM289 |
| Perpetual position/details/candles/activity | `GemPerpetualDetailsService`, `GemCandleSession`, position rows, chart load rules | — |
| Stake, validators, delegation and claim | `GemStakeService`, validator/delegation records, generated transfer input | VM279; preserve exact atomic values |
| NFT root/collection/unverified, detail/report/avatar | `GemNftService`, `GemCollectibleService`, shared rich rows and avatar flow | — |
| Price-alert list/target/auto-alert controls | `GemPriceAlertService`, alert session and existing notification port | — |
| Rewards/create/use/redeem referral | `GemRewardsService`, rewards state, shared load and list records | VM291, VM292 |
| Contacts/list/editor/address picker | `GemContactService`, `GemContactEditorService`, contact session/name component | — |
| Networks/node list/add/check | `GemChainSettingsService`, node sessions, shared rows | — |
| Settings/preferences/currency/language/appearance/about | `GemSettingsService`, `GemCurrencyService`, `GemAppUpdateService`, preference observation | VM283, VM290; retain native locale/theme application |
| Security/lock/biometry/recovery | `GemSecurityService`, existing keystore/auth ports and settings sections | retain platform-only privacy lock |
| Push settings, in-app notifications, support chat | Notification services, `GemSupportService`, permission and lifecycle ports | — |
| WalletConnect list/detail/proposal/request/signing | `GemWalletConnectService` (sign messages scanned through `GemScanService`), `GemSignMessageService`, Reown adapters | VM281, VM291; retain Android-only one-click auth |
| Info sheets, docs links and shared display components | `GemInfoTopic`, `GemFormattedNumber`, shared rich/plain renderers, the two mapper files per app | — |
| Widgets | `GemWidgetService` (Android); the iOS widget stays off Gemstone by rule | retain native widget scheduling |
| Stores and persistence | `Gem*Store` traits and both adapters | D175, VM288, BD299, GEN300 |
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



## 3. Decisions the apps still make

The same product rule written in both apps, or in one app while the other reads Core ([the app maps; it does not decide](ARCHITECTURE.md#5-the-app-maps-it-does-not-decide), [a view never names a Core type](ARCHITECTURE.md#a-view-never-names-a-core-type)). Where the two apps answer differently today the item says how.

- **VM183** **S** **Phrase suggestions are cut and applied by each app.** Core returns suggestions for one word (`phrase_suggestions`, `core/gemstone/src/mnemonic.rs`); the input handling around it is written twice.
  - **iOS:** `ImportWalletSceneViewModel` takes the last whitespace-separated word, offers suggestions only while the cursor is at the end, and applies a pick by dropping the last word and appending the word and a space.
  - **Android:** `ImportWalletViewModel` does the same with its own `lastWord()`, cursor check and `selectSuggestion`.
  - **Expected:** Core owns both steps beside `phrase_suggestions`: one call takes the input and whether the cursor is at the end and returns the suggestions, one returns the input with the picked word applied.
  - **Needs a decision:** [security](../skills/security.md) keeps phrase editing in the app and gives Core only the word being typed, while both calls above hand Core the whole phrase on every keystroke; either the rule or this design changes.

## 4. Numbers and text the apps still format

[A number crosses as a value and a style](ARCHITECTURE.md#a-number-crosses-as-a-value-and-a-style-never-as-a-string-or-a-callback) and [the record carries the finished value](ARCHITECTURE.md#the-record-carries-the-finished-value-not-the-ingredients). Each item is a raw amount, price or seconds value that both apps format, convert or compose themselves.



## 5. Twins, adapters, redundant models and dead code

[An app row model stores the row and nothing else](ARCHITECTURE.md#an-app-row-model-stores-the-row-and-nothing-else), [a details screen gets a details record](ARCHITECTURE.md#a-details-screen-gets-a-details-record-not-a-row-plus-the-object-it-came-from), and a type that only crosses the FFI is used as the generated type. Delete-first items are at the end.

The target for every item below: a model that only renames or regroups a Core record is deleted and the view takes the record; the two mapper files turn titles, tones and icon kinds into platform values; anything the model decides (a text, an icon choice, a visibility, a grouping) moves into the Core row, list or session that feeds it.

### One generated type system

- **D175** **L** **The apps use only the UniFFI-generated types.** On hold by the owner (2026-09-25). The generated Swift and Kotlin models (`Primitives/Sources/Generated`, `gemcore/.../primitives/generated`, 133 types) and the generated mappers between the two systems (`RemoteTypeMappers.swift`, `RemoteTypeMappers.kt`) go.
  - **Today:** 123 of the 133 generated model types already have a UniFFI twin (`core/bin/generate/remote_types.yml`). 936 iOS files and 638 Android files import the generated models, and about 210 files per app convert with `toPrimitives()`/`toGem()`. `Chain` crosses as its string code, and `AssetId`, `WalletId`, `TransactionId`, `NFTAssetId`, `NFTCollectionId` and `PerpetualId` cross as their stored strings. iOS `Store` depends on `Primitives` but not on `Gemstone`.
  - **Phase 1, coverage:** declare the 10 generated model types the bindings lack, or delete their app use: `AssetSubtype`, `ContactData`, `Device`, `PriceData`, `QRScanType`, `ScanReceiveMode`, `StakeChain`, `TransactionNFTTransferMetadata`, `TransactionSwapMetadata`, `WCPairingProposal`.
  - **Phase 2, enums and identifiers:** `Chain` may migrate separately to a generated UniFFI enum. Keep the existing handwritten platform wrappers and parsers for `AssetId`, `NFTAssetId`, `NFTCollectionId`, `PerpetualId`, `TransactionId`, and `WalletId`, including their stable stored-string conversions; do not replace them with generated UniFFI records.
  - **Phase 3, storage and routes:** `just generate-models` emits `Codable` and `Hashable` conformances (iOS) and kotlinx serializers (Android) for the generated types that routes and stored JSON carry: iOS `Scenes`, Android route arguments, and three GRDB JSON columns. iOS `Store` gains the `Gemstone` dependency.
  - **Phase 4, the apps, module by module:** replace generated model imports with the UniFFI types and delete each mapper once its last caller goes. Order: store adapters and `Store`, then shared components, then features. VM286 (`SelectAssetType`), VM288 (iOS service wrappers) and VM289 (Android aggregates) land inside this phase.
  - **Phase 5, removal:** stop generating the Swift and Kotlin models, delete both `RemoteTypeMappers` and the mapper sections of the generator, and fold the hand-written rest of the iOS `Primitives` package into `GemstonePrimitives`.
  - **Widget:** the iOS widget links neither `Gemstone` nor `GemstonePrimitives` and decodes API JSON with generated `Codable` models. Default: it keeps a small widget-local model for the fields it shows, so the no-Gemstone rule stands.

### Shared components, second round

- **VM262** **M** **Screens count their own lists so Core can pick the phase.**
  - **iOS:** `SelectAssetSceneViewModel` (`GemAssetSectionCounts`), `WalletSearchSceneViewModel` and `AssetsResultsSceneViewModel` (`GemWalletSearchCounts`), `PerpetualsSceneViewModel` (`GemPerpetualMarketCounts`), and `loadError(state:hasRows:)` in the asset, transactions, fiat transactions, price alerts, asset price alerts, notifications and support screens.
  - **Android:** `BaseSelectAssetViewModel`, `WalletSearchViewModel`, `AssetsResultsViewModel`, `PerpetualsViewModel` and `loadError(..., candles.isNotEmpty())` in `PerpetualViewModel`.
  - **Expected:** the Core call that builds the screen reads the list sizes through the store port it already has and returns sections and phase together; the counting and the `hasRows` arguments go.
- **VM264** **S** **Search result sections are split in the apps.**
  - **iOS:** `WalletSearchSections.from` splits pinned and other assets (`AssetsSections.from`) and calls `perpetualMarketSections`; `WalletHomeState` re-assembles the home sections, header and flags.
  - **Android:** `WalletSearchViewModel` and `WalletViewModel` split the same way; `WalletSummary` re-assembles the home state.
  - **Expected:** the search and home view states carry finished sections; the splitting types go.
- **VM267** **S** **Simulation payload fields are mapped twice.**
  - **iOS:** `SimulationPayloadFieldViewModel` (with `SimulationPayloadFieldKind` and `models(for:)`) maps text, address and timestamp values and wires address taps.
  - **Android:** `SimulationPayloadFieldsContent` does the same per value case.
  - **Expected:** payload rows render through the shared row renderer (address rows as `GemAddressRow`); both go.
- **VM269** **S** **Banner destinations are routed per screen.**
  - **iOS:** `WalletSceneViewModel` opens only URL banners and ignores stake, activate and perpetual destinations; `AssetSceneViewModel` handles all four; both map banner buttons to header actions.
  - **Android:** `WalletScene` ignores the same three; `BannerItem` handles all four.
  - **Expected:** Core emits only destinations the screen can open and each app keeps one banner routing function.


### Screen composition

- **VM279** **S** **The earn screen's rows are built in the apps.**
  - **iOS:** `EarnSceneViewModel` builds the "No data" and "Deposit" rows, decides the empty state and shows "Positions" only when positions exist.
  - **Android:** `EarnViewModel.depositListItem` and `EarnScreen` build the same.
  - **Expected:** the earn view returns its sections and rows.
- **VM280** **M** **Transaction details are composed per row kind in the apps.**
  - **iOS:** `TransactionSceneViewModel` maps each `GemTransactionDetailRow` kind to an item, writes "Swap again", rebuilds the fee amount and routes header taps; `TransactionItemModel` lists the cases.
  - **Android:** `TransactionViewModel`, `TransactionItemUIModel` and `TransactionScene` do the same.
  - **Expected:** the details sections carry finished rows; both item layers go.
- **VM281** **S** **WalletConnect proposal rows are hardcoded in the apps.**
  - **iOS:** `ConnectionProposalSceneViewModel` builds wallet, connection, status and the two permission rows.
  - **Android:** `ConnectionProposalViewModel` builds the same four.
  - **Expected:** a Core proposal record returns the rows.
- **VM282** **S** **The perpetual market hides its header while searching in the views.**
  - **iOS:** `PerpetualsScene` gates the balance header on `!model.isSearching`.
  - **Android:** `PerpetualsScene` gates the header and sections on `isSearching`.
  - **Expected:** `GemPerpetualMarketSession` sections include or omit the header.
- **VM283** **S** **Settings screens hand Core the values its preference store holds.**
  - **iOS:** `PreferencesSceneViewModel` builds `GemPreferencesInput` and `GemPerpetualDefaults` from stored values; `SecuritySceneViewModel` builds `GemSecurityInput`.
  - **Android:** `PreferencesViewModel` and `SecurityViewModel` build the same inputs.
  - **Expected:** the settings services read preferences through their store port and return sections; the inputs go.
- **VM284** **S** **Recipients are built from contacts in each app.**
  - **iOS:** `RecipientSceneViewModel.contactRecipients` flattens contacts into `GemRecipient`s.
  - **Android:** `RecipientViewModel` does the same.
  - **Expected:** `recipientSections` reads contacts through the contact store; both flattenings go.

### Models and adapters

- **VM285** **S** **The fee asset of an asset screen is the chain coin in both apps.**
  - **iOS:** `ChainAssetQuery` loads `assetId.chain.assetId` as the fee asset.
  - **Android:** `ChainAssetQuery` does the same for tokens.
  - **Expected:** Core answers the fee asset for an asset; both stores read it.
- **VM286** **S** **iOS twins `GemSelectAssetType`.**
  - **iOS:** `SelectAssetType` and `SelectAssetSwapType` mirror the generated enum and map back through `flowType`.
  - **Android:** uses `GemSelectAssetType` directly.
  - **Expected:** iOS uses the generated enum; the twin and `flowType` go.
- **VM287** **S** **Amount routes twin `GemAmountRequest`.**
  - **iOS:** `AmountType` and `AmountInput` are rebuilt into a request in `AmountSceneViewModel`.
  - **Android:** `AmountParams` (and `toAmountParams`) is rebuilt into a request in `AmountViewModel`.
  - **Expected:** routes carry `GemAmountRequest` (on `GemAmountSession`).
- **VM288** **M** **iOS re-wraps Core service methods to take Primitives types.**
  - **iOS:** `GemstonePrimitives/Sources/Services/*.swift` (wallet, wallet session, WalletConnect, contacts, wallet home, perpetual details, swap quote, and others) and `GemConfirmMetadata+GemstonePrimitives` wrap Core calls with conversions.
  - **Android:** calls Core with `toGem()` at each call site.
  - **Expected:** callers use the generated types directly; the wrapper extensions go.
- **VM289** **S** **Android domain aggregates wrap Core rows.**
  - **iOS:** uses `GemPerpetualMarketItem` directly in views.
  - **Android:** `PerpetualDataAggregate`, `PerpetualPositionDataAggregate(Impl)`, `WalletSummary`, `LeverageState` and `NftAssetDetailsData` (`gemcore/.../domains`) wrap them.
  - **Expected:** Android uses the records directly; the aggregates go.

### Scenes

- **VM290** **M** **Empty states are decided scene by scene.**
  - **iOS:** `FiatTransactionsScene`, `InAppNotificationsScene`, `PriceAlertsScene` and `TransactionsScene` show the empty view when the list is empty and there is no error; `ContactsScene`, `ConnectionsScene`, `ChainListSettingsScene`, `ValidatorSelectScene`, `WalletImageScene`, `CurrencyScene`, `ImportWalletTypeScene` and `CollectionsScene` check emptiness themselves; `AssetPriceAlertsSceneViewModel.showsEmpty`, `EarnSceneViewModel.showsEmptyState` and `PerpetualsPreviewViewModel.hasNoPositions` decide it in the model.
  - **Android:** `TransactionsScene`, `ConnectionsScreen`, `FiatTransactionsScene`, `EarnScreen`, `StakeScene`, `ContactsScreen`, `InAppNotificationsScene`, `CollectionsScene` (empty and no unverified row), `CurrencyScene` (only while a query is typed, unlike iOS), `PerpetualsScene`, `ValidatorSelectScene`, `PriceAlertsScene`, `NetworkAssetsScreen`, `SelectChain`, `WalletImageScene` and `PerpetualsPreviewSection` do the same.
  - **Expected:** each list screen's Core phase says it is empty and which empty kind to show, as `GemSelectAssetState` already does; the scenes render the phase (land with VM262).
- **VM291** **S** **Whether a wallet can be chosen is decided in the apps.**
  - **iOS:** `RewardsSceneViewModel.showsWalletSelector` checks `wallets.count > 1`.
  - **Android:** `RewardsScreen` checks `availableWallets.size > 1`; `AuthRequestScreen` does the same for WalletConnect authentication.
  - **Expected:** the rewards state and the WalletConnect request say whether a wallet can be chosen; the counts go.
- **VM292** **S** **The rewards screen's intro and action placement are written in the apps.**
  - **iOS:** `RewardsScene` lists the three intro features with their emojis and titles and places share or create-code, use-code and the pending referral.
  - **Android:** `RewardsHead` lists the same three features and `RewardsScene` picks the first share or create-code action, then use-code and the pending referral.
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

- **VM181** **S** **Keystore secrets are exported only for the flows that need them.** `GemKeystore.create_store`, `export_private_key` and `export_recovery_phrase` are exported for app tests alone (iOS `LocalKeystore+Export.swift`, `LocalKeystore+Keystore.swift`; Android `MigrateV3KeystoreFilesTest`, `GemKeystoreBenchmarkTest`, `GemKeystoreConcurrencyTest`), while the apps import and export through the wallet service. Move those tests onto the production path, then make the three methods a plain `impl`, so no secret-exporting symbol exists that no flow uses ([security](../skills/security.md)). `check-ffi-surface.py` allows the three until then. Needs a decision: no production flow reaches these three, but `GemWalletService.export_secret` returns the same secrets in the same process, so dropping them narrows the binding rather than closing a path, and it costs rewriting the iOS keystore test kit (`LocalKeystore+Keystore.swift`, which most wallet tests use to create a wallet) and the keystore integration, benchmark and v3-migration tests on both apps onto `import_wallet` and `export_secret`. Drop them, or keep them for those tests?

## 6. Orchestration, services and stores

Core has no runtime, so scheduling, timers and OS callbacks stay in the apps; what moves is the decision — what to do, in what order, under what condition — returned as one call or one record. And [a store returns what Core reads](ARCHITECTURE.md#4-the-store-trait-is-the-apps-only-persistence-obligation), through one trait per responsibility.

- **VM79** **S** **The root scene stops reading the wallet store.** iOS [`RootSceneViewModel`](../ios/Gem/ViewModels/RootSceneViewModel.swift) reads `stores.walletStore.getWallet` directly; the session service answers the current wallet. Blocked on a synchronous answer: `GemWalletSessionService::get_current_wallet` is async (the wallet store port is async), and the root view needs the wallet on its first render or it flashes onboarding at every launch; either the port gains a synchronous read or the root keeps a stored wallet it can seed before first render.

## 7. Rows and taps

Taps on rows that already exist, not new row types.



## 8. Persistence and parity

- **BD299** **S** **Android loses an NFT's resource when it stores it.**
  - **iOS:** `NFTAssetRecord` stores `resourceUrl` and `resourceMimeType` and reads them back into `NFTAsset.resource` (`ios/Packages/Store/Sources/Models/NFTAssetRecord.swift`).
  - **Android:** `DbNFTAsset.toAssetModel` returns `resource = NFTResource("", "")` and the preview image with an empty mime type (`data/services/gemstone/.../nft/NftModels.kt`), so a stored NFT has no resource.
  - **Expected:** Android stores and returns the resource URL and mime type as iOS does (Room migration for the two columns); lands before GEN300 so one mapping spec serves both apps.


## 9. Behavior differences

Differences between the apps, or between an app and the server, each with its decision.

### Same rule, different answers

- **BD373** **S** **A network's asset list shows assets without an account on Android.**
  - **iOS:** the per-network asset screen requires an account on the chain, like the wallet list.
  - **Android:** the wallet list does since BD349, but the per-network screen (`NetworkAssetsViewModel`) still lists account-less assets.
  - **Expected:** Android matches iOS.
- **BD374** **S** **Swap's percentage buttons fetch on a different schedule.**
  - **iOS:** 25/50/100% and "Use minimum amount" fetch at once.
  - **Android:** they set the amount field like typing, so they wait the 250 ms debounce (`SwapViewModel`).
  - **Expected:** Android matches iOS; the view model tells a button tap from typing.
- **BD375** **S** **A deleted transaction keeps or clears its details screen.**
  - **iOS:** the details screen keeps showing the transaction it was opened with, and reads it before opening.
  - **Android:** the screen loads the transaction just after opening and clears when the row is deleted (`TransactionViewModel`).
  - **Expected:** Android matches iOS.
- **BD376** **S** **Android address rows offer no contact actions.**
  - **iOS:** an unnamed transaction recipient or sender offers "Create New Contact" and "Add to Contact" on a long press, from the address row's `contact`.
  - **Android:** `AddressPropertyItem` offers copy and the explorer only; no add-contact route takes an address.
  - **Expected:** Android matches iOS.
- **VM344** **S** **iOS support chat holds two Core services.** `SupportChatSceneViewModel` holds the support and notifications services ([ARCHITECTURE § 7](ARCHITECTURE.md#7-at-most-one-core-service-observed-reads-are-queries)); Android enables support push through the `EnablePushForSupport` port. **Expected:** the support service answers the push enablement, and the view model holds one service.
- **VM323** **S** **iOS navigation resolves deep links through stores.** `NavigationRouter` holds `AssetStore` and `TransactionStore` (`ios/Gem/Navigation/NavigationRouter.swift`), the same reach past the service as a view model holding a store ([ARCHITECTURE § 7](ARCHITECTURE.md#7-at-most-one-core-service-observed-reads-are-queries)). It reads a stored transaction to open a transaction push and the asset's data to open a recipient link. **Expected:** the router reads through the owning Core service or a query.
- **BD341** **S** **Android finds no stored Earn provider for a token.** Both apps store an Earn provider under its chain's native asset (`DelegationValidator.toRecord`, iOS `StakeValidatorRecord`). iOS `EarnSceneViewModel` reads `ValidatorsQuery(chain:providerType:)` by that native asset; Android `EarnViewModel` reads `ValidatorsQuery` by the screen's asset, and `AmountViewModel` reads `ValidatorQuery` for an Earn deposit the same way, so a Yo USDC or USDT screen gets no providers and Core returns no `deposit_provider`. **Expected:** Android reads Earn providers by the chain's native asset, as iOS does.
- **BD29** **S** **Redemption options with unlimited stock are never listed.** `summary.rs:18` (`remaining.unwrap_or_default() > 0` drops `None`) vs storage `rewards_redemptions_repository.rs:48,96` and Core `rules.rs:146` (`None` = unlimited). Needs a decision: `test_available_redemption_options` pins a `None` stock as hidden, so confirm whether options stored without a stock are meant to be offered before changing the server.

### Freshness

- **BD50** **S** **Public `/chain/fee-estimates` can serve very old estimates.** `core/crates/services/src/chain/fee_estimates_client.rs:73-76` has no freshness check (TTL 5 years, `core/crates/cacher/src/keys.rs:155`); the per-chain route refreshes (`:55-70`). Used by the website, not the apps. **Needs a decision (2026-09-24):** either the public route drops entries past the one-hour fresh key (the website loses chains nobody requested lately) or it refreshes them (a public route then triggers node calls).

## 10. Generated mappers

- **GEN300** **L** **Both apps' persistence mappers are generated from one spec.** Each app hand-writes the record ↔ model mapping for the same tables: iOS about 66 mapping members (about 780 lines) in `ios/Packages/Store/Sources/Models/*Record.swift`, Android about 49 mappers (about 650 lines) in `android/data/services/store/.../database/entities/Db*.kt`, about 60–65% of them plain field copies. They drift (BD299), and Android builds the same record in several places: four `DbAsset` builders (`AssetFull.toRecord`, `Asset.toRecord`, `AssetBasic.toRecord`, `AssetBasic.toUpdateRecord`), `DbPerpetual.toUpdate` duplicating `toDB`, and two `DbPrice.toAssetPrice` that disagree on a missing price (`stores/StoreModels.kt` returns 0, `DbTransactionExtended.kt` drops it).
  - **Expected:** a `records:` section in `remote_types.yml` names each record or entity, the model it maps and the few field rules a mapping needs (rename, flatten a nested record into prefixed columns, integer cast, parent key); the generator writes both directions per app (`Store/Sources/Generated/RecordMappers.swift`, `data/services/store/.../generated/EntityMappers.kt`), and the hand-written copies go. The record and entity declarations and schema stay hand-written. Start with the plain copies (`Account`, `AddressName`, `AssetLink`, `Contact`, `ContactAddress`, `SupportMessage`, `PriceAlert`, `Perpetual`, `PerpetualPosition`, `FiatRate`, `Banner`, `InAppNotification`), then the flattened ones (`Asset`, `NFTAsset`, `NFTCollection`, `FiatTransaction`, `WalletConnection`, `Balance`); mappings with real logic stay hand-written (`Transaction` id split, `AssetMarket` reassembly, `Price` positive check, `Node` status). Land after BD299.

## 11. Module layout and names

A feature module is one product area, and both apps give it the same name. iOS groups by product area and is the reference; Android splits many areas into one module per screen.

**The standard, for every item below.** Names and packages follow [Cross-Platform Awareness rule 7](../skills/cross-platform-awareness.md); on Android that means no `views`, `navigation` or `details` package roots and no singular `viewmodel`. A move renames the Gradle path in `settings.gradle.kts`, every `project(":features:…")` dependency and the imports, and changes no behaviour. One module per change. Verify with `cd android && ./gradlew assembleGoogleDebug test` for an Android move, `cd ios && just build` and `just test-package <Package>` for an iOS rename.

**Names, for every item below.** Each item renames one feature's types to [ARCHITECTURE § Names](ARCHITECTURE.md#names): the base name follows iOS, each app keeps its own form (Android `XScreen` binds the view model and `XScene` is stateless, iOS screen view models are `XSceneViewModel`), a file is named after its main type, and tests, TestKit mocks, routes and factory methods follow the type they name. Renames only, no behaviour change; verify both apps (`just test` on iOS, `./gradlew testDebugUnitTest assembleGoogleDebug` on Android) and `just check-docs`. Each list was checked against the code on 2026-09-26; re-check a name before renaming it.

## 12. Cleanup sweeps

Two passes over the whole repository, Core, iOS and Android, run after the items above have moved their rules and deleted their models. Each goes one platform and one module family per change, builds and tests that module before moving on, and says in the commit what it removed. Neither pass removes something an open item already names (that item deletes it with its replacement), and neither touches a public contract: API routes and fields shipped apps or the website read (see X168), stored formats, database and keystore migrations, and deep link URLs stay until their own item retires them. Changes near key material, signing or transaction construction follow [security](../skills/security.md).

- **CLN318** **L** **Unused and redundant code is removed on every platform.** Dead code is still compiled, generated, reviewed and copied by the next change that reads nearby code.
  - **Unused:** a declaration nothing in production reaches. One reached only by its own test, a mock or a preview counts as unused, and its test goes with it (as `String.words()` went). Find candidates per platform, then confirm each by search and a build:
    - **Core:** `just core unused` (cargo machete) for dependencies; `pub` items with no caller outside their crate, which rustc does not flag; `scripts/check-ffi-surface.py` for exports no app calls, shrinking its `ALLOWED` list to entries that name an open item; generated model types neither app reads; unused `CacheKey` variants, config fields and features.
    - **iOS:** a Periphery scan of the workspace for unused declarations, protocols and conformances; package dependencies in `Package.swift` no source imports; `Style` image assets nothing references.
    - **Android:** Android Lint `UnusedResources`; top-level functions, extensions, DAO queries and classes with no caller; Gradle dependencies no module source uses. Keep what the manifest, Hilt, WorkManager or reflection reaches.
    - **Both apps:** Fluent keys in `localization/` that neither app reads (a key is unused only when both apps drop it), then `just localize`.
  - **Redundant:** two helpers, formatters, extensions or fixtures in one platform that do the same thing keep one, and callers move to it. The twin of a Core record belongs to its `VM` item, not here.
  - **Done when:** each tool reports nothing, or every remaining hit is listed in the commit with the reason it stays.
- **CLN319** **L** **Every unit test is reviewed, and the ones that protect no contract go.** Last in the order: before it, the `VM` items move rules into Core and the `GEN` items replace mocks, which changes which app tests still mean anything. Unit tests only; integration tests and Maestro flows keep their own rules ([testing-maestro](../skills/testing-maestro.md)).
  - **Delete a test that:** asserts a constant, static table or 1:1 enum mapping back at itself; tests generated code (generated models, `RemoteTypeMappers`, generated mocks), which the generator's golden tests cover; re-tests in an app a rule Core owns and tests (the app keeps a test only for wiring and visible output); repeats another test's case in the same platform with no new boundary; covers a getter, `copy`, equality or framework behaviour (SwiftUI, Compose, GRDB, Room) rather than ours; mocks the very code it claims to check; or still passes when the rule it names is inverted.
  - **Keep, and fix rather than delete:** a weak test that guards a real contract gets the assertion the contract needs. Signing, keystore, derivation, transaction construction, amounts and decimals, address validation, wire formats and migrations are never deleted without an equivalent test in the same change, whether in Core or the app.
  - **Check:** for each test kept on a business rule, invert the rule, run the test, confirm it fails, and restore ([engineering principles](../skills/engineering-principles.md#tests)). A contract found with no test gets the smallest one at its owner.
  - **Run:** `cd core && just test <CRATE>`, `cd ios && just test-package <Package>` (confirm the target is still in the test plan and the expected tests ran), `cd android && ./gradlew :<module>:testDebugUnitTest`. Record the test count per module before and after in the commit.
  - **Done when:** every Core crate, iOS package and Android module has been reviewed once.

## Blocked upstream

- **X168** **S** `WalletConfiguration.multi_signature_accounts` ([`wallet_configuration.rs`](../core/crates/primitives/src/wallet_configuration.rs)) is the old name of `externally_controlled_accounts`, which also covers Solana accounts assigned to another program. The API fills both because shipped apps read only the old field. Delete the field, its fill in [`wallet_configuration.rs`](../core/crates/services/src/devices/wallet_configuration_client.rs) and the merge in [`externally_controlled_banners`](../core/gemstone/src/services/wallet_configuration/rules.rs) on 2026-12-18, three months after the release that reads `externally_controlled_accounts`.
- **X163** **M** iOS pins the `Gemstone` package to Swift 5 language mode. Re-tested on 2026-09-16 against uniffi 0.32.1: both `uniffiTraitInterfaceCallAsync` sites still fail with "passing closure as a 'sending' parameter" because the generated `Task { }` captures three `@escaping` non-`Sendable` parameters. Nothing to decide and nothing to do until a uniffi release changes that function; re-test then. Rechecked on 2026-09-22 with `cargo search uniffi`: 0.32.1 is still the latest uniffi release, and the fix is already merged upstream as uniffi #2929 (`694fda6a05`, 2026-07-15, marks the async trait-interface closures `@Sendable`) but is not in 0.32.1. Bump uniffi to the first release that contains #2929, regenerate, and move the `Gemstone` package to Swift 6 mode.
