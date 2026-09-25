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

1. **Server:** BD30, BD52.

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
| Activity, asset/position history, transaction details | `GemTransactionsService`, `GemTransactionDetailsService`, detail records, native indexed queries | — |
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
| Info sheets, docs links and shared display components | `GemInfoTopic`, `GemFormattedNumber`, shared rich/plain renderers, the two mapper files per app | — |
| Widgets | `GemWidgetService` (Android); the iOS widget stays off Gemstone by rule | retain native widget scheduling |
| Stores and persistence | `Gem*Store` traits and both adapters | — |

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

- **VM181** **S** **Keystore secrets are exported only for the flows that need them.** `GemKeystore.create_store`, `export_private_key` and `export_recovery_phrase` are exported for app tests alone (iOS `LocalKeystore+Export.swift`, `LocalKeystore+Keystore.swift`; Android `MigrateV3KeystoreFilesTest`, `GemKeystoreBenchmarkTest`, `GemKeystoreConcurrencyTest`), while the apps import and export through the wallet service. Move those tests onto the production path, then make the three methods a plain `impl`, so no secret-exporting symbol exists that no flow uses ([security](../skills/security.md)). `check-ffi-surface.py` allows the three until then. Needs a decision: no production flow reaches these three, but `GemWalletService.export_secret` returns the same secrets in the same process, so dropping them narrows the binding rather than closing a path, and it costs rewriting the iOS keystore test kit (`LocalKeystore+Keystore.swift`, which most wallet tests use to create a wallet) and the keystore integration, benchmark and v3-migration tests on both apps onto `import_wallet` and `export_secret`. Drop them, or keep them for those tests?

## 6. Orchestration, services and stores

Core has no runtime, so scheduling, timers and OS callbacks stay in the apps; what moves is the decision — what to do, in what order, under what condition — returned as one call or one record. And [a store returns what Core reads](ARCHITECTURE.md#4-the-store-trait-is-the-apps-only-persistence-obligation), through one trait per responsibility.

- **VM79** **S** **The root scene stops reading the wallet store.** iOS [`RootSceneViewModel`](../ios/Gem/ViewModels/RootSceneViewModel.swift) reads `stores.walletStore.getWallet` directly; the session service answers the current wallet. Blocked on a synchronous answer: `GemWalletSessionService::get_current_wallet` is async (the wallet store port is async), and the root view needs the wallet on its first render or it flashes onboarding at every launch; either the port gains a synchronous read or the root keeps a stored wallet it can seed before first render.

## 7. Rows and taps

Taps on rows that already exist, not new row types.


## 8. Persistence and parity


## 9. Behavior differences

Differences between the apps, or between an app and the server, each with its decision.

### Same rule, different answers

- **BD29** **S** **Redemption options with unlimited stock are never listed.** `summary.rs:18` (`remaining.unwrap_or_default() > 0` drops `None`) vs storage `rewards_redemptions_repository.rs:48,96` and Core `rules.rs:146` (`None` = unlimited). Needs a decision: `test_available_redemption_options` pins a `None` stock as hidden, so confirm whether options stored without a stock are meant to be offered before changing the server.
- **BD30** **S** **"Use code" is offered when it can't succeed.** Core always offers `UseReferralCode` (`core/gemstone/src/services/rewards/rules.rs`, `actions`), but the server only accepts a code on a device and wallet set up within the eligibility window (`core/crates/rewards/src/referral.rs`, `validate_use`; attribution referrers skip it in `rewards_client.rs`). Hiding it needs the device and wallet age facts in the rewards summary.

### Freshness

- **BD50** **S** **Public `/chain/fee-estimates` can serve very old estimates.** `core/crates/services/src/chain/fee_estimates_client.rs:73-76` has no freshness check (TTL 5 years, `core/crates/cacher/src/keys.rs:155`); the per-chain route refreshes (`:55-70`). Used by the website, not the apps. **Needs a decision (2026-09-24):** either the public route drops entries past the one-hour fresh key (the website loses chains nobody requested lately) or it refreshes them (a public route then triggers node calls).
- **BD52** **S** **The fiat quote cache lasts exactly the app's refresh interval and is keyed by IP, so Buy late or after a network change gives "Forbidden".** `core/crates/cacher/src/keys.rs:136` (5 min), key `fiat_cacher_client.rs:52`, miss → 403 (`:40`); app refresh `core/gemstone/src/services/fiat/mod.rs:24` (5 min, active screen only). **Decided:** key the quote cache by device instead of IP, and keep it for 15 minutes.

## Blocked upstream

- **X168** **S** `WalletConfiguration.multi_signature_accounts` ([`wallet_configuration.rs`](../core/crates/primitives/src/wallet_configuration.rs)) is the old name of `externally_controlled_accounts`, which also covers Solana accounts assigned to another program. The API fills both because shipped apps read only the old field. Delete the field, its fill in [`wallet_configuration.rs`](../core/crates/services/src/devices/wallet_configuration_client.rs) and the merge in [`externally_controlled_banners`](../core/gemstone/src/services/wallet_configuration/rules.rs) on 2026-12-18, three months after the release that reads `externally_controlled_accounts`.
- **X163** **M** iOS pins the `Gemstone` package to Swift 5 language mode. Re-tested on 2026-09-16 against uniffi 0.32.1: both `uniffiTraitInterfaceCallAsync` sites still fail with "passing closure as a 'sending' parameter" because the generated `Task { }` captures three `@escaping` non-`Sendable` parameters. Nothing to decide and nothing to do until a uniffi release changes that function; re-test then. Rechecked on 2026-09-22 with `cargo search uniffi`: 0.32.1 is still the latest uniffi release, and the fix is already merged upstream as uniffi #2929 (`694fda6a05`, 2026-07-15, marks the async trait-interface closures `@Sendable`) but is not in 0.32.1. Bump uniffi to the first release that contains #2929, regenerate, and move the `Gemstone` package to Swift 6 mode.
