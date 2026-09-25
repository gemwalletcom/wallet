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

1. **Shared records:** VM88 (info sheets), VM180 (one mapper file per app), then VM6.
2. **Balances and storage:** D76, D77, VM98 (an Android migration).
3. **Server:** BD23, BD30, BD51, BD52.

Waiting on the owner: BD29 and BD50 (server), VM79, VM181. Waiting on a date or a release: X168, X163.

## Screen coverage and existing infrastructure

This map routes work to current owners. It groups existing ids rather than creating a new service or task for every view. Shared components and platform-only features keep their established boundaries; the service map in [ARCHITECTURE.md](ARCHITECTURE.md#screen-services) supplies the current app callers.

| Screens / entry points | Existing owner or infrastructure to extend | Open work |
|---|---|---|
| App start, foreground, wallet switch, deep links and pushes | `GemAppStartService`, `GemWalletSessionService`, `GemNavigationService`, `GemAppUpdateService`, native lifecycle hosts | VM79 |
| Create/import wallet, terms, phrase generation and verification | `GemWalletService`, `GemVerifyPhraseSession`, `phrase_suggestions`, keystore and native auth ports | — |
| Wallet list/detail, rename, avatar, secret export | Wallet rows/details, existing export flow and NFT avatar selection | — |
| Wallet home, header, network assets, banners | `GemWalletHomeService`, `GemBalanceService`, `GemBannerService`, shared asset rows and banner context | — |
| Asset search/select, add token, recents | `GemAssetSelectionService`, `GemSelectAssetFlow`, `GemAddAssetService`, recent activity | — |
| Asset details and asset actions | `GemAssetDetailsService`, shared rows, copy, info and load state | — |
| Portfolio chart/statistics | `GemPortfolioService`, chart load rules, shared numbers and rows | — |
| Asset chart/market/alerts sections | `GemChartService`, `GemChartSession`, shared list renderer | — |
| Receive, QR display, address details | `GemReceiveService`, `GemAddressDetailsService`, `GemCopy`, payment encoding | Retain existing native QR/share adapters |
| Scanner, payment links, deep links and pushes | Existing payment decoder, `GemPaymentService`, push/navigation preparation | — |
| Amount entry, fiat equivalent, amount extras | `GemAmountService`, `GemAmountRequest`, `GemAmountEntry`, `GemAutocloseDraft` | — |
| Confirmation, fees, simulation, acquisition | `GemConfirmTransferService`, `GemConfirmation`, `GemConfirmScreen`, shared headers/rows/info | — |
| Swap, providers, slippage and swap details | `GemSwapQuoteService`, `GemSwapSession`, `GemSlippageSession` | — |
| Activity, asset/position history, transaction details | `GemTransactionsService`, `GemTransactionDetailsService`, detail records, native indexed queries | VM98 |
| Buy/sell quotes, provider opening, fiat history | `GemFiatQuoteService`, `GemFiatSession`, existing fiat transaction owner | — |
| Perpetual market list/search/pins and balance | `GemPerpetualService`, market session/rows, native search indexes | — |
| Perpetual position/details/candles/activity | `GemPerpetualDetailsService`, `GemCandleSession`, position rows, chart load rules | — |
| Stake, validators, delegation and claim | `GemStakeService`, validator/delegation records, generated transfer input | Preserve exact atomic values |
| NFT root/collection/unverified, detail/report/avatar | `GemNftService`, `GemCollectibleService`, shared rich rows and avatar flow | — |
| Price-alert list/target/auto-alert controls | `GemPriceAlertService`, alert session and existing notification port | — |
| Rewards/create/use/redeem referral | `GemRewardsService`, rewards state, shared load and list records | — |
| Contacts/list/editor/address picker | `GemContactService`, `GemContactEditorService`, contact session/name component | — |
| Networks/node list/add/check | `GemChainSettingsService`, node sessions, shared rows | — |
| Settings/preferences/currency/language/appearance/about | `GemSettingsService`, `GemCurrencyService`, `GemAppUpdateService`, preference observation | retain native locale/theme application |
| Security/lock/biometry/recovery | `GemSecurityService`, existing keystore/auth ports and settings sections | Retain platform-only privacy lock |
| Push settings, in-app notifications, support chat | Notification services, `GemSupportService`, permission and lifecycle ports | — |
| WalletConnect list/detail/proposal/request/signing | `GemWalletConnectService` (sign messages scanned through `GemScanService`), `GemSignMessageService`, Reown adapters | retain Android-only one-click auth |
| Info sheets, docs links and shared display components | `GemInfoTopic`, `GemFormattedNumber`, shared rich/plain renderers, the two mapper files per module | VM6, VM88, VM180 |
| Widgets | `GemWidgetService` (Android); the iOS widget stays off Gemstone by rule | retain native widget scheduling |
| Stores and persistence | `Gem*Store` traits and both adapters | VM98 |

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

## 4. Numbers and text the apps still format

[A number crosses as a value and a style](ARCHITECTURE.md#a-number-crosses-as-a-value-and-a-style-never-as-a-string-or-a-callback) and [the record carries the finished value](ARCHITECTURE.md#the-record-carries-the-finished-value-not-the-ingredients). Each item is a raw amount, price or seconds value that both apps format, convert or compose themselves.


## 5. Twins, adapters, redundant models and dead code

[An app row model stores the row and nothing else](ARCHITECTURE.md#an-app-row-model-stores-the-row-and-nothing-else), [a details screen gets a details record](ARCHITECTURE.md#a-details-screen-gets-a-details-record-not-a-row-plus-the-object-it-came-from), and a type that only crosses the FFI is used as the generated type. Delete-first items are at the end.

- **VM6** **L** **Views take Core records; the adapter protocols go.** Four app-side view protocols force a model per Core record: `ItemModelProvidable` (10 conformances on 2026-09-23, mostly confirm and transaction details), `ListSectionProvideable` (10, settings-style screens that already hold `GemListSection`), `SimpleListItemViewable` (4) and `ValueHeaderViewModel` (7). Once the items in § 2 and § 3 leave no decision in those models, the shared views take the Core record directly and render it through the module's mapper extensions, and each adapter is deleted rather than kept as a pass-through. Update [An app row model stores the row and nothing else](ARCHITECTURE.md#an-app-row-model-stores-the-row-and-nothing-else) in the same change: it currently lets a model stand when "a list protocol demands one", which is the allowance these adapters live on. Each adapter goes once its screen's model holds no decision.
- **VM180** **M** **One localization mapper and one style mapper per app.** The owner wants the Core-to-platform mappers in one file per app (2026-09-24); today [ARCHITECTURE § One mapper per module](ARCHITECTURE.md#one-mapper-per-module-names-every-core-key-it-renders) asks for two files per module, so the same Core enum can be mapped in several modules and `just check-mappers` compares module by module. Rewrite that section in the same change.
  - **iOS:** 17 `Gemstone+Localized.swift` files (one per feature, plus `PrimitivesComponents` and `WalletConnectorService`) and 6 `Gemstone+Style.swift` files. They merge into `PrimitivesComponents/Extensions/Gemstone+Localized.swift` and `Gemstone+Style.swift`, which every feature already imports.
  - **Android:** 25 `localization/GemstoneText.kt` files (`ui`, `app` and 23 feature modules) and 4 `style/GemstoneStyle.kt` files. They merge into `ui/localization/GemstoneText.kt` and `ui/style/GemstoneStyle.kt`, where the shared strings already live.
  - **Expected:** one file of each per app; a mapper written in two modules (the same Core enum mapped twice) becomes one; `scripts/check-mapper-parity.py` compares the two app files directly.
- **VM181** **S** **Keystore secrets are exported only for the flows that need them.** `GemKeystore.create_store`, `export_private_key` and `export_recovery_phrase` are exported for app tests alone (iOS `LocalKeystore+Export.swift`, `LocalKeystore+Keystore.swift`; Android `MigrateV3KeystoreFilesTest`, `GemKeystoreBenchmarkTest`, `GemKeystoreConcurrencyTest`), while the apps import and export through the wallet service. Move those tests onto the production path, then make the three methods a plain `impl`, so no secret-exporting symbol exists that no flow uses ([security](../skills/security.md)). `check-ffi-surface.py` allows the three until then. Needs a decision: no production flow reaches these three, but `GemWalletService.export_secret` returns the same secrets in the same process, so dropping them narrows the binding rather than closing a path, and it costs rewriting the iOS keystore test kit (`LocalKeystore+Keystore.swift`, which most wallet tests use to create a wallet) and the keystore integration, benchmark and v3-migration tests on both apps onto `import_wallet` and `export_secret`. Drop them, or keep them for those tests?

## 6. Orchestration, services and stores

Core has no runtime, so scheduling, timers and OS callbacks stay in the apps; what moves is the decision — what to do, in what order, under what condition — returned as one call or one record. And [a store returns what Core reads](ARCHITECTURE.md#4-the-store-trait-is-the-apps-only-persistence-obligation), through one trait per responsibility.

- **VM79** **S** **The root scene stops reading the wallet store.** iOS [`RootSceneViewModel`](../ios/Gem/ViewModels/RootSceneViewModel.swift) reads `stores.walletStore.getWallet` directly; the session service answers the current wallet. Blocked on a synchronous answer: `GemWalletSessionService::get_current_wallet` is async (the wallet store port is async), and the root view needs the wallet on its first render or it flashes onboarding at every launch; either the port gains a synchronous read or the root keeps a stored wallet it can seed before first render.
- **VM88** **M** **Info sheets come from Core.** Every sheet is written twice: iOS `InfoSheetType` (32 cases) with [`InfoSheetModelFactory`](../ios/Features/InfoSheet/Sources/Factory/InfoSheetModelFactory.swift) (252 lines), and Android `InfoSheetEntity` (33 classes in `InfoBottomSheet.kt`, 414 lines); 44 iOS and 49 Android call sites create them. No sheet exists on only one app.
  - **Core:** `GemInfoTopic` ([`models/list.rs`](../core/gemstone/src/models/list.rs)) grows to every sheet kind: it adds watch wallet, payment verification, staking reserved fees, pending unconfirmed balance, asset status, existing wallet imported and the `GemConfirmErrorSheet` kinds. `GemInfoTopic::sheet(platform) -> GemInfoSheet { title, description, image, docs_url, action }` composes each sheet: title and description keys with their finished arguments (network name, symbol, formatted amounts including the "amount (~fiat)" join, which `GemConfirmErrorInfo+InfoSheet.swift:14-18` and `ConfirmErrorUIModel.kt:85-89` compose today; the renderer applies bold), an image descriptor (logo, network fee, watch, token status, provider, asset icon, transaction state with its badge), the docs URL finished by `DocsUrl::url_for`, and an action kind (`LearnMore | Buy | Acquire | Continue`) the app wires to its closure. The call composes values per sheet, so it is not a constant export.
  - **iOS:** one mapper `GemInfoSheet → InfoSheetModel`; delete the per-case content, `InfoSheetModelFactory`, `GemConfirmErrorInfo+InfoSheet.swift` and `ConfirmInfoSheetBuilder`. The feature sheet enums (`SwapSheetType`, the amount, confirm, transaction, stake and asset sheets) carry `.info(GemInfoTopic)`; their other cases are navigation.
  - **Android:** one mapper `GemInfoSheet → InfoSheetEntity`; delete the 33 subclasses, `GemInfoTopic.infoSheet` and the sheet building in `ConfirmErrorUIModel`.
  - **Expected, the iOS content:** the network fee title key (iOS `info.network_fee.title`, Android `transfer_network_fee`); the network-fee-required title argument (iOS the asset's display title, Android the native symbol) and image (iOS the fee asset, Android the native asset); the account-minimum-balance and stake-frozen images (iOS logo, Android the asset icon; Android also passes an unused `titleArgs`); the stake lock time and APR images (iOS the placeholder only, Android the asset icon); bold `available` and `shortfall` in the swap-minimum sheet (iOS only); the network-fees link on the staking-reserved-fees sheet (iOS only); a balance-required or swap-minimum sheet without an acquire action (iOS shows it without a button, Android shows none).
  - **Expected, Android's rule:** every row that carries a topic opens it. iOS drops the autoclose topic on the amount screen (`AmountPerpetualViewModel.swift:58`) and on the confirm modify-autoclose row (`ConfirmTransferScene.swift:80`). Android's network-fee-required docs link, which can never open, goes.
- **D76** **S** **Whether one failed balance request discards its network's other answers.** `chain_balances` in [`balance/mod.rs`](../core/gemstone/src/services/balance/mod.rs) joins the coin, staking, token and earn results with `?`, so a failed staking or earn request throws away the coin and token balances that succeeded on that network, and no test covers the case; the product intent in [product/wallet.md](product/wallet.md) says a slow or failing request must not hold back the others. **Decided:** publish the components that answered and return the first component failure (extend `published_balances` to per-component results, add the test).
- **D77** **S** **When the wallet list updates during a balance refresh.** `update` waits for every network (`join_all`) and every component (`join!`) before its single write, so the fastest network's coin balance shows only when the slowest has answered or failed; [ARCHITECTURE](ARCHITECTURE.md#publish-a-multi-source-refresh-as-one-batch) chose one batch on purpose (fewer observer notifications, no mixed-age totals), and the product owner wants balances "as soon as possible". **Decided:** write each network as it finishes, each write atomic and lane-ordered, per the product rule in [product/wallet.md](product/wallet.md); update the ARCHITECTURE section in the same change.

## 7. Rows and taps

Taps on rows that already exist, not new row types.


## 8. Persistence and parity

- **VM98** **M** **Transaction assets are stored two ways.**
  - **iOS:** stores every asset a transaction touches through `transactionAssetIds` into its transaction-assets table.
  - **Android:** stores only swap pairs through `transactionSwapPair` into `DbTransactionSwapMetadata`.
  - **Expected:** iOS's schema. Android gains the transaction-assets table in a Room migration that carries the existing swap pairs over.

## 9. Behavior differences

Differences between the apps, or between an app and the server, each with its decision.

### Same rule, different answers

- **BD23** **S** **ENS/UD names with a first part over 20 characters, and provider outages, both look like "name not found".** `core/crates/name_resolver/src/client.rs:26-29` with `max_name_length: 20` (`core/Settings.yaml:156`); `core/apps/api/src/devices/mod.rs:197-200` discards errors (`.ok().flatten()`); Core accepts any label (`core/gemstone/src/services/name/rules.rs:28-31,57-62`). **Decided:** the API answers a provider outage with the standard `ApiError`, so Core reads it as an error rather than a missing name, and a missing name keeps its 200 answer with no record; `max_name_length` stays 20.
- **BD29** **S** **Redemption options with unlimited stock are never listed.** `summary.rs:18` (`remaining.unwrap_or_default() > 0` drops `None`) vs storage `rewards_redemptions_repository.rs:48,96` and Core `rules.rs:146` (`None` = unlimited). Needs a decision: `test_available_redemption_options` pins a `None` stock as hidden, so confirm whether options stored without a stock are meant to be offered before changing the server.
- **BD30** **S** **"Use code" is offered when it can't succeed.** Core always offers `UseReferralCode` (`core/gemstone/src/services/rewards/rules.rs`, `actions`), but the server only accepts a code on a device and wallet set up within the eligibility window (`core/crates/rewards/src/referral.rs`, `validate_use`; attribution referrers skip it in `rewards_client.rs`). Hiding it needs the device and wallet age facts in the rewards summary.

### Freshness

- **BD50** **S** **Public `/chain/fee-estimates` can serve very old estimates.** `core/crates/services/src/chain/fee_estimates_client.rs:73-76` has no freshness check (TTL 5 years, `core/crates/cacher/src/keys.rs:155`); the per-chain route refreshes (`:55-70`). Used by the website, not the apps. **Needs a decision (2026-09-24):** either the public route drops entries past the one-hour fresh key (the website loses chains nobody requested lately) or it refreshes them (a public route then triggers node calls).
- **BD51** **S** **The country from an IP check is cached 30 days for rewards but 1 day for fiat.** `core/crates/cacher/src/keys.rs:122` vs `:137`. Likely. **Decided:** one day for both.
- **BD52** **S** **The fiat quote cache lasts exactly the app's refresh interval and is keyed by IP, so Buy late or after a network change gives "Forbidden".** `core/crates/cacher/src/keys.rs:136` (5 min), key `fiat_cacher_client.rs:52`, miss → 403 (`:40`); app refresh `core/gemstone/src/services/fiat/mod.rs:24` (5 min, active screen only). **Decided:** key the quote cache by device instead of IP, and keep it for 15 minutes.

## Blocked upstream

- **X168** **S** `WalletConfiguration.multi_signature_accounts` ([`wallet_configuration.rs`](../core/crates/primitives/src/wallet_configuration.rs)) is the old name of `externally_controlled_accounts`, which also covers Solana accounts assigned to another program. The API fills both because shipped apps read only the old field. Delete the field, its fill in [`wallet_configuration.rs`](../core/crates/services/src/devices/wallet_configuration_client.rs) and the merge in [`externally_controlled_banners`](../core/gemstone/src/services/wallet_configuration/rules.rs) on 2026-12-18, three months after the release that reads `externally_controlled_accounts`.
- **X163** **M** iOS pins the `Gemstone` package to Swift 5 language mode. Re-tested on 2026-09-16 against uniffi 0.32.1: both `uniffiTraitInterfaceCallAsync` sites still fail with "passing closure as a 'sending' parameter" because the generated `Task { }` captures three `@escaping` non-`Sendable` parameters. Nothing to decide and nothing to do until a uniffi release changes that function; re-test then. Rechecked on 2026-09-22 with `cargo search uniffi`: 0.32.1 is still the latest uniffi release, and the fix is already merged upstream as uniffi #2929 (`694fda6a05`, 2026-07-15, marks the async trait-interface closures `@Sendable`) but is not in 0.32.1. Bump uniffi to the first release that contains #2929, regenerate, and move the `Gemstone` package to Swift 6 mode.
