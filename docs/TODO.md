# Open work

Every open item carries a stable id (V vocabulary, R rows, C composition, S sessions, B view boundary, F formatting, P parity, D decisions, O ownership, X platform, G guidance, T tests, L localization, N naming, PERF performance) and a size (**S**/**M**/**L**). Contracts are in [ARCHITECTURE.md](ARCHITECTURE.md) and [SERVICES.md](SERVICES.md). **Delete an item's line in the commit that lands it** — ids are never reused.

The goal is that Gemstone decides once and both clients read that decision. Track duplicated decisions and concrete performance work at their existing owners: shared rules and orchestration in Core; rendering, observation, scheduling, and localized formatting in the apps.

Keep each item independently reviewable. Shared decision changes land in Core and both apps; platform-only work stays on that platform. Regenerate only when shared interfaces or integration change, and run the applicable [Quality Checks](../skills/quality-checks.md). Verify affected primary-screen journeys under [Performance](PERFORMANCE.md). Remove replaced paths within the item's scope; do not bundle an unrelated row migration or product change.

This list was rebuilt on 2026-09-15 from scripted sweeps over the whole repo and widened the same day by a second, deeper pass. Each item names the file or symbol the sweep hit, so it can be confirmed before it is started; a sweep hit is a lead, not a verdict, and an item that turns out to be correct as written is closed by deleting its line with a one-line note in the commit.

These files were checked during the 2026-09-15 mapper sweep and need no change — each calls its module mapper or switches over an app type, not a Core one: iOS `NFT/CollectibleViewModel`, `Settings/GemAddNodeFailure+Settings`, `Swap/SwapSlippageViewModel`, `Swap/Views/SwapDetailsView`, `Transfer/Types/ConfirmInfoSheetBuilder`, `Transfer/ConfirmRecipientViewModel`, `Transfer/RecipientSceneViewModel`; Android `earn`, `import_wallet`, `perpetual`.

## 7. Platform

### Hardcoded dp (Android rule: theme constants only)

Re-checked on 2026-09-15 by separating a `\d+.dp` written at a call site from one written as a named constant: **Android has none of the former**. All 58 remaining sites are `private val name = N.dp` declarations, which is the pattern the codebase already uses for a component's own dimensions, and only a handful of those carried a value the theme also names for the same kind of thing — those now read from the theme. What is left is a QR finder's 25 dp corner, a 42 dp search bar, a 296 dp slippage sheet and the like: dimensions with no theme equivalent, each named where it is used. X23–X29 are closed with no change.

### Swallowed errors

Closed on 2026-09-15. Four of the six were the same `try { focusRequester.requestFocus() } catch {}` written out in four screens — Compose throws when the target is not attached yet, so the catch is right and the duplication was the problem; they now call `requestFocusIfAttached()`. The other two were real: binding the camera use cases swallowed its failure, so a camera that could not start showed a blank preview with nothing in logcat, and the collectible details flow swallowed its load error and stayed null forever. Both report now.

### Deferred notes still in the code

- **X35** **S** `ios/Packages/Gemstone/Package.swift` pins Swift 5 language mode. Re-checked on 2026-09-15 against the current toolchain: dropping the pin fails on two `uniffiFutureContinuationCallback` sites in the generated `Gemstone.swift` — "passing closure as a 'sending' parameter risks causing data races". The fix is upstream in uniffi's Swift bindgen, not here; re-check after the next uniffi bump.
- **X36** **S** `LocalKeystore.swift` and `DB.swift` each run `FileMigrator` at launch to move the keystore directory and the database out of the documents directory into application support. Both were marked "delete in 2026" and the notes are gone, but the code stays until someone with the install numbers decides: deleting it while any user is still on a pre-move build points the keystore at a path that does not exist, and that user loses their wallet. The cost of keeping it is one `fileExists` per launch after the first (`FileMigrator.migrate` returns the new URL untouched when the old path is empty), so the default is to keep it. What settles it: the share of active installs whose last upgrade predates the move.
- **X40** **S** `core/apps/api/src/devices/mod.rs` — the legacy singular route is due after 2026-11-15.
- **X41** **S** `core/crates/primitives/src/device_locale.rs` `from_client` accepts codes the current apps never send. Checked on 2026-09-15: `in`, `iw` and `tl` are not legacy at all — the JVM still returns those for Indonesian, Hebrew and Filipino, so those arms are permanent. What is removable is the region-less `zh` and `pt` and the long "fall back to English" list, and only once no installed client sends them; `from_locale_identifier` already normalises what both apps send today. The note in the code that called all of it legacy is gone.
- **X42** **S** `core/crates/primitives/src/swap_provider.rs` still carries `CetusAggregator` beside `CetusClmm`. Checked on 2026-09-15: a completed swap stores its provider as a string in `TransactionSwapMetadata.provider`, so dropping the variant does not break a stored row — it breaks reading one back, and an old Cetus swap loses its provider name and its swap-again action. The query that settles it is whether any stored swap metadata still carries `cetus_aggregator`; the note in the code, which blamed client references, is gone.


### View models with no test

The logic weight in brackets is methods plus computed properties. 95 of 159 iOS and 45 of 65 Android feature view models have no test file; these are the heaviest.


## 11. Missing tests

### Hardcoded user-visible strings

Swept on 2026-09-15 over every non-preview, non-test iOS file: 25 hits, of which 20 are the developer screen (a debug screen that is deliberately untranslated) and 4 are inside a `PreviewProvider`. The one real hit was the wallet screen's "Trade Perpetuals" row, now `perpetuals_trade`. The same sweep over Android found none. All three new keys — `perpetuals_trade`, `widget_empty` and `widget_empty_short` — were translated into the other 30 locales on 2026-09-16, each following the locale's existing `perpetuals_title` and `errors_no_data_available` wording.


T29 closed on 2026-09-16. `widget_empty` and `widget_empty_short` are translated in all 31 locales. What is left is one rule — how many coins each widget family shows — and a row model that formats a price, a percentage and a colour. Both would cost a new app-extension test target plus the package-product link that T28 found to be broken, which is more scaffolding than the rule is worth; the rule belongs in the same fix.

T28 closed on 2026-09-16 after trying it. `ScanReceiveModeViewModel` is an id and a title, `MainTabViewModel` is one `ObservableQuery`, and `RootSceneViewModel` is the composition root: thirteen collaborators and a body that is `Task { await service... }` in every method, so a test of it is a test of the mocks. The one rule worth covering — a required upgrade offers only the update action — sits behind that constructor. Wiring the existing empty `GemTests` unit-test target into `unit_frameworks.xctestplan` was tried and does not link: attaching any package product to a test bundle makes Xcode rebuild `Store` as a dynamic package product, and that product does not resolve `BigInt`, so the link fails before the tests run. That is the thing to fix first if the app target ever needs tests.

T27 closed on 2026-09-16. `ReceiveViewModel` was already covered by eleven tests — the count was by file name again. `AmountEarnViewModel` and `PerpetualModifyViewModel` decide something and are tested; `KeystoreAuthenticationViewModel` is a three-case glyph map and `ReceiveNetworkSelectorViewModel` builds its items from the ids it is handed, which is the mapper contract, not a decision.

T26 closed on 2026-09-16 with one file. `SwapTokenViewModel` decides what the pay and receive rows allow and prices what was typed, and is tested; `SwapProvidersViewModel` is a `SelectableListAdoptable` whose three members are localized constants and `SwapPairSelectorViewModel` is two optional asset ids with no logic at all — a test over either would assert the compiler.

### iOS view models with no test file

130 across `Features`, `Packages`, `Gem` and the widget — the wider count that X55–X64's preamble narrows to feature modules. Grouped by module so each item is one test target's worth of work; X55–X64 already name the heaviest. The count is by file name, so a module whose view models are tested from a differently named file still shows up: `Features/Transactions` already had seven test files when T21 was written, and closing it meant covering the two that decide something — the activity list and its filter — not all sixteen.


T37 closed on 2026-09-16. `ConfirmViewModel` already had eight tests across four files named for what they cover — the header, the request, the retry and the network fee sheet — so the count by file name missed it again. `NetworkFeeCustomViewModel` was the real gap and is tested: it opens on the rate it was handed, it never lets a letter reach the rate, and a rate over the maximum cannot be confirmed.

T33 closed on 2026-09-16. `AutocloseViewModel` and `PerpetualDetailsViewModel` are tested; `PerpetualsPreviewViewModel` is two `stateIn` passthroughs over a config flag and the position list, with nothing of its own to assert.

T31 closed on 2026-09-16. Four of the five — `BuySelectViewModel`, `SendSelectViewModel`, `ManageSelectViewModel`, `ReceiveSelectViewModel` — are one-line bindings of a `GemSelectAssetType` to `BaseAssetSelectViewModel`, so the base is what was tested: the chain filter narrows the list, clearing the filters puts it back, and pinning an asset tells Core and names it in the toast. Two more were dropped after being written: the recent list reacts to `snapshotFlow { queryState.text }`, which needs Compose snapshot notifications a plain JVM test does not dispatch, and an asset carrying a balance never reaches `assetsContent` in a JVM test, so the balance filter cannot be exercised from outside the view model.

### Android view models with no test

49 of them; X65 already names three.


## 14. Decisions still made on a client

The standing goal, restated on 2026-09-16: **any business logic moves to Gemstone, and nothing is decided twice on the clients.** The sweep that finds this work compares the two apps rather than reading one: the same computation on both sides, or one app calling Core where the other does the arithmetic itself, is a divergence waiting to happen — not a style difference. Four landed on 2026-09-16:

- Rejecting a WalletConnect proposal. iOS mapped the error to a CAIP-25 reason and deleted the stored session; Android sent the string `"Reject Session"` for every rejection and kept the session. `session_rejection` now returns the reason, the code, the message the dApp sees and whether the session is deleted.
- The slippage percent. Android asked Core; iOS wrote `Double(bps) / 100` in two places, and Android wrote it a third time in `SwapDetailsUIModelFactory`. All three read `slippage_percent` now, and `GemSwapQuoteSummary` carries it for the screens that hold the summary but no service.
- Margin usage on the portfolio screen. Both apps computed `account_value * usage` and `usage * 100` and composed the same `"value (percent)"`. `PortfolioMarginUsage` carries `used_value` and `usage_percent`.
- Crypto to fiat. iOS converted through `CryptoFiatConverter`; Android multiplied two doubles in `BalanceInfoUIModel`, `DelegationInfo` and `AssetPriceValue`, which loses Core's precision rules. All of them go through Core now, and `GemSwapValue::fiat_value` is exported for the swap screens.

Crypto to fiat came back on 2026-09-16: four iOS sites still read the atomic value into a `Double` and multiplied it by the price — the asset balance, the swap pay row, the swap provider row and both delegation rows — which is the precision loss Android had already been fixed for. `CryptoFiatConverter.to_fiat` no longer throws (a price that is not a number is not a price, and a `BigInt` always stringifies), and `PriceViewModel.fiatValueText(value:decimals:)` is the one iOS path into it.

Checked and clean: the price-impact model, the swap rate text and Android's `EquivalentValue` all read a Core value and map only the label, the colour or the locale format.

A fifth landed the same day: the developer screen. iOS held five stores beside its service and built an eleven-row table of sample transactions by hand; Android held the service alone and could offer none of the actions. `GemDeveloperStore` names the seven database operations, `sample_transactions` owns the table, and both screens now reach the database only through `GemDeveloperService`.




## 15. More platform work

### iOS errors thrown away

Re-checked on 2026-09-15. `Store/Migrations.swift` holds 88 of the 174 sites and they are the idempotent-migration idiom — `try? db.alter` for a column that may already exist, `try? db.drop` for a table that may not. Rewriting those against a shipped wallet database is a data-loss risk with no defect behind it, so they stay. `clearChainData` was the one that was wrong: it deleted a removed chain's rows from seven tables with `try?`, so a locked table or a constraint left the rows behind silently. It now asks `tableExists` the way `clearTables` beside it already did and lets a real error through.

`WalletIdMigration.swift` had the same shape and the same one real problem: it rewrote `walletId` across ten child tables and deleted a wallet's child rows with `try?`, so a failure on any one of them left the wallet's data pointing at an id that no longer exists, and `cleanupOrphanedRecords` then swallowed the foreign-key check as well. All three now skip a table that does not exist and let a real failure roll the migration back.

Checked and kept: `LocalKeystore.findV3File` scans a directory and `try?` is how an unreadable file is skipped; `SwapSceneViewModel.currentInput` returns nil because "no complete input yet" is what its errors mean; `NavigationPathState` and `NavigationHandler` decode a path or a wallet id where nil is a real answer.

The rest are one or two per file and each needs reading on its own.

X74 is closed after reading the remaining files. One was wrong: `RewardsViewModel` built the referral link with `try? ... ?? ""`, so a link the service could not build was shared and copied as an empty string instead of the action disappearing; `referralLink` now returns nil and logs, and `shareText` follows it. The rest are the nil-is-the-answer shape — `try? AssetId(id:)` on a deeplink string, `try? NumberInput.value` on half-typed input in the swap and delegation rows, `try? service.suggestPair` where no suggestion is a normal outcome — plus the developer screen, which is deliberately forgiving.

`AnyCodableValue.swift`, `AssetId.swift` and `AnyCodableValue+Store.swift` (X73) are closed with no change. All fifteen sites are the decoder-probe idiom: `try? container.decode(Bool.self)` asks "is this a bool", and the answer "no" is what drives the next branch — the last one throws a real `DecodingError` when nothing matched, and `AssetId` falls through from the string form to the keyed form. The `decode(_:)` and `encode(_:)` helpers return an Optional on purpose; nil means "the value is not that type", which is the same answer. Rewriting any of them would turn a branch condition into a thrown error with nowhere to go.


`WalletConnectorService.swift` (X71) held the three worst of them and is done. Two were the reject path swallowing its own failure, which now logs. The third encoded a WalletConnect request's params with `try?` and fell back to `""`, so a request whose params would not encode reached Core as a request with no parameters; it now rejects the request instead. Rejecting a proposal also stopped being decided twice — see [the finished value](ARCHITECTURE.md) — so Core names the reason, the code, the message and whether the session is deleted.



### Logging left in shipping paths

Re-checked on 2026-09-15 by separating what actually ships: 116 of the 122 iOS sites are `debugLog`, which compiles to nothing outside `DEBUG`, and the remaining six `print` calls are inside a `#Preview` or a macOS-only availability note. On Android 60 of the 69 are `Log.e` on a real failure. The seven `Log.d` that shipped are gone — they were logging WalletConnect request method, chain and id, the whole stream payload, and connection transitions into logcat on a release build — and the two migration errors that were logged at debug level now log as errors. X75–X83 are closed.

### iOS layout numbers outside Style

Re-checked on 2026-09-15 the way the Android dp items were: most of the 31 hits are inside a `#Preview` or are a named configuration field (`QRScannerDisplayConfiguration.default`), which is the pattern. What was real: a segmented picker width repeated in three scenes and a chart height repeated in two now read `Sizing.picker.segmentedWidth` and `Sizing.chart.height` (Android has named the chart height all along), and the `spacing: 0` / `cornerRadius: 10` / `spacing: 24` call sites read from `Spacing`. X84–X88 are closed.


### Files that have outgrown one module, second pass

All five closed on 2026-09-16 after measuring what each length is made of.

**X89** and **X90** are the same file in two languages and both say so on line one: `RemoteTypeMappers.kt` and `models/remote_types.rs` are emitted by `just generate-models` from `core/bin/generate/remote_types.yml`. The answer to "how many the generator could emit" is all of them; the length is the number of types that cross the FFI.

**X91** `chain_config.rs` is a table — the config types, then one `ChainConfig { .. }` literal per chain, 102 of them. **X92** `message/signer.rs` measured 771 file lines and 231 production lines; the rest is its test module, the same miscount as X46–X52. **X95**'s `GemCandlestickChart.kt` is one chart and its private drawing helpers, and `AmountListHead.kt` is one component family — five composables sharing a private item type and private layout state — so splitting it means widening those to `internal` and trading file length for a wider surface.

### The shared localized mapper

**X94** closed on 2026-09-16 with no change. `PrimitivesComponents/Extensions/Gemstone+Localized.swift` is 35 extensions that each give one Core enum its localized label, and the length is the number of Core enums the apps draw, not a mix of concerns — the style half already lives beside it in `Gemstone+Style.swift`. One shared mapper per app is the rule ([ARCHITECTURE.md](ARCHITECTURE.md)); splitting it into several files is what that rule exists to prevent, and a module that wants its own copy is the mistake the rule catches. It grows when Core names a new outcome, which is the contract working.

### The two Android migration files

**X96** closed on 2026-09-16. Both are reachable: `provideRoom` registers every step from 41 to 71 by hand and then `gemDatabaseMigrations` from 71 to the current 94, so the chain from the oldest schema is unbroken and neither can be deleted without breaking an upgrade from an old install. `Migration_71_72.kt` stays as one file — it is one schema step whose nine ordered helpers have to run in one transaction, and the comment at the top says why the order matters. `Migration_41_42.kt` was the real find: it held three migrations (41→42, 42→43, 43→44) under one migration's name, which is why it measured long. Each now has its own file, the way every migration from 44 on already did.

### The services factory

**X97** closed on 2026-09-16 with no change. `ServicesFactory.makeServices` is one function because it is one dependency graph: 91 local bindings in construction order, each feeding the next, ending in a single `AppResolver.Services`. `ViewModelFactory` split cleanly (X54) because every screen constructor is independent of the others; here a split into `makeCoreServices`, `makeWalletServices` and so on would have to thread twenty-odd intermediate values between the halves, which is more moving parts than the straight line it replaces. The length tracks the number of services the app has, not a mix of concerns.

### App-side twins of Core types

[No hand-written twins](ARCHITECTURE.md): an FFI-only type is used as the uniffi type; a twin is only for a type an app persists. Each of the three below has the same cases and the same payload types as its Core counterpart and is never written to storage.

Checked and kept: `KeystoreAuthentication` and `LockPeriod` are both written to the keychain by raw value, which the rule allows; `AmountType` carries a recipient its Core namesake does not; `SelectAssetType` and `PaymentDestination` are navigation types carrying app payloads and already map to Core through `flowType`.

## Closed with no change

Each of these was a section of the 2026-09-15 sweeps. The work was to check them; the answer was that the sweep measured the wrong thing. They are kept so the same sweep is not run again with the same conclusion.

### 2. Lists and screens without a row record

All 13 closed by 2026-09-16. Four landed a record — the recipient sections, the perpetual market sections, the contact address fields and the node check rows — and the rest were screen chrome over decisions Core already makes.

Nine were checked and need no record. The add-node, contact, add-asset, import-wallet and connection-proposal screens are the same case in a different shape: every decision they make — the phase of the node check, whether a chain takes a memo, which import kinds a chain offers, whether an address shows a view-only warning — already comes from Core, and what is left is a screen title, a field label and a button, fixed and unconditional. The connection proposal's two permission lines are the same list in the same order on both apps with nothing conditional behind them; naming them in Core would be the lookup wrapper the guidance bans. The amount screen's balance and reserved-fee lines are a localized template around an amount each platform formats with the device locale, and the `shows_asset_balance` and `can_change_value` decisions already come from `GemAmountInput` — moving the text into Core would take the locale out of the number. The autoclose summary was the one real find: Android hand-joined `"$label: $value"` where iOS and Android's own position row both call `trigger_order_text`, so a cleared trigger read differently; Android now calls it too. The scanner error is two platform conditions — no camera, permission denied — with the same two strings on both apps and nothing for Core to decide.

Copy: [`GemAssetRow`](../core/gemstone/src/services/assets/model.rs) → [iOS](../ios/Packages/PrimitivesComponents/Sources/ViewModels/ListAssetItemViewModel.swift), [Android](../android/gemcore/src/main/kotlin/com/gemwallet/android/domains/asset/aggregates/AssetInfoDataAggregate.kt). Each of these iOS view models composes four or more user-facing strings and holds no Core record; the count in brackets is how many. Land each with its Android mirror.

### 5. Screens that may want a session

Both closed on 2026-09-16. The chain settings screen got one: the nodes and their statuses were two app-side fields and each app guarded staleness differently, so `GemNodeListSession` now drops a status for a node the list no longer has and Android's refresh nonce is gone. The transactions filter did not need a session — it needed the filter set, which Core now builds; the sheet's own state is two selections and a presentation flag.

### 3. Views that decide
Both closed on 2026-09-15. **B9**: all 21 iOS scenes and views that name `Gemstone` `switch` over a row key or read a row record — the contract working — and the two that branch on a Core value (`TransactionSwapProgressView` showing the estimated time on the spinner step, `SupportMessageBubble`) make the same call Android makes in the same place. **B10**: of 52 Android composables that `when` over a Core enum, the branch is an icon, a colour or a painter in almost every case — the style half of the mapper contract — and `android/ui` was keeping it in eight ad-hoc files. The tone, state and verification mappings now live in `ui/style/GemstoneStyle.kt` beside the header-button icon. What is left branches on a Core value to pick a keyboard, a paste handler or a trailing composable, which is rendering.

### 4. Ownership
O15–O21 are closed with no change. `LockSceneViewModel` builds `GemSecurityService` only inside its `static var preview`, which is SwiftUI preview code, not wiring. The 42 "types named by neither app" are the same mistake as § 10: a type that reaches an app as a nested field or an enum payload is never spelled out in Swift or Kotlin, so naming is the wrong test. Every one of the 42 was checked — `GemAmountStakeType` is a field of `GemAmountType`, `GemCollectibleAttribute` is a payload of `GemCollectibleSection`, `GemEIP712Message` is built by the message signer — and none is unreachable. A genuinely dead Core type is still worth finding; a name search does not find it.

### 8. Performance
The one item here landed: the node list built its rows one FFI crossing per node on every emission, and Core now takes the nodes and their statuses together. No other measured regression is open — [PERFORMANCE.md](PERFORMANCE.md) names the owner of each primary journey and says plainly that none has been profiled on a device.

### 9. Core decides it, only one app reads it

All 35 closed by 2026-09-15. Four landed a change — the confirm error chevron, the listed-asset-rank consolidation, the Android amount prefill and the sign-message forwarders — and the rest measured the wrong thing.

Each of these was an `#[uniffi::export]` the sweep found named in one app and in neither the other app's Kotlin nor its Swift. The first run skipped every iOS file whose name ends `+Gemstone.swift`; the corrected run is the one these notes describe.

The sweep measures which *export* each app names, which is not the same as which app *owns* the decision. Eleven were checked on 2026-09-15 and closed with no change, because the app that never calls the export still reads the same Core answer through a different one: Android reads the abbreviation cutoff through `GemValueStyle.abbreviates`, the price-alert kind through the aggregate's `kind.groupsByAsset()`, the dApp name through `GemConfirmDestination.Generic` and `connection_row`, whether to show a memo through the confirm row set, the latest block through the node row's `GemNodeSubtitle.LatestBlock`, the swap minimum through `GemSwapButtonAction.UseMinimumAmount`, and whether to offer rewards through the `GemSettingsRow.REWARDS` the settings service emits from it.

The second pass closed ten more. Three were never exported: `named`, `synchronize` and the avatar service's `set_image` / `remove_image` sit in plain `impl` blocks that the sweep's name match picked up, and the avatar methods reach both apps through `GemWalletService`. Four are the same Core answer asked for differently: Android passes `submit_attempted` straight into the autoclose session constructor instead of calling `on_submit_attempt`, clears the add-asset form with `new_session` instead of `on_chain`, reads the recommended validators out of the stake selection record, takes `swap_quote` through `swapper_quote_summary`, and reads the swap error display off the session rather than through the free function iOS uses to give `SwapperError` a description. The rest are platform plumbing, not decisions: Android shows no error at all for a failed biometric prompt so it has nothing to gate on `is_cancelled`, its image loader caches a support attachment by URL so it never needs `image_file`, and the developer screen simply offers fewer actions than the iOS one. The second pass also closed twelve on the iOS side. `update_balance` and the five perpetual sync methods are plain `impl` blocks, not exports — only the iOS mock reimplements them by name. `listed_asset_rank` is gone: both apps now read the rank from the asset config service. `decode_url` has one caller in the whole repo, an Android instrumentation test; both apps decode a payment link through `load`, which goes to the same `PaymentURLDecoder`. Android needs `chain_from_caip2` because it routes on the `Chain` enum app-side while iOS hands the CAIP-2 string straight to Core, and it builds a session from `on_auto` where iOS builds the same session from the `Auto` selection. The rest are one app not having the flow at all: iOS inserts no default asset record when a wallet is created, has no invalid-word highlighting in the import field, no NFT receive chain picker, no token-search sync behind its price widget, no timed retry after a failed biometric prompt, and its update check compares versions inside `newest` rather than against a Play archive.

The WalletConnect items resolved on the same pass. `message_preview`, `message_address_names` and `address_url` were forwarders on the connect service over the sign-message service iOS already calls directly; they are gone and Android holds the sign-message service. `is_origin_rejected` and `user_rejected_error` are both applied inside `process_request` and `session_proposal`, which is how iOS gets them; Android's extra calls belong to the one-click authentication request, a flow iOS does not implement at all — the same reason `authentication_accounts`, `authentication_chain_ids` and `authentication_methods` look one-sided. The swap session transitions are not two sets either: `on_request_changed` composes `on_refresh_requested`, which composes `on_quote_invalidated`. Android calls the innermost one eagerly when the user picks an asset, switches the pair or changes slippage, so the transfer phase clears before the request recomputes; iOS reaches the same state through the outer transition.




### iOS does not read an Android-read decision

### 11. Core behaviour with no test

All eight closed on 2026-09-15. The orchestration in `wallet_home`, `asset_discovery`, `assets/details`, `app_start`, `rewards` and `perpetual` now has tests. `asset_discovery/testkit.rs` assembles the balance, discovery, transactions and NFT graph behind one constructor; `app_start` and `rewards` build on the wallet testkit so they sign with a real keystore; and `TestAlienProvider::with_json_by_path` answers each endpoint with its own body.

### 10. Exports no app calls at all
Closed on 2026-09-15 with no change. The sweep counted **app** callers, which is the wrong test for a `rules.rs` function: rules are called by the service that owns them, and the app calls the service. Every function listed here has Core callers — `sanitize_number_input` has fifteen, `node_url` twenty-three, `price_alert_toggle` is read by the asset row, `shows_header` by the confirm screen — and the explorer getters are used by nine other services. A Core export with no caller anywhere is still worth finding; counting app callers alone does not find it.

### Files that have outgrown one module

All nine closed by 2026-09-16, two of them by changing the file.

**X46–X52** counted file lines, and in a Rust module the tests live at the bottom of the file they test. The production halves are 622, 596, 556, 518, 489, 465 and 443 lines — the top of a smooth distribution across 44 `rules.rs` files whose median is about 130, not a cliff. Each one is also a single subject: `transactions/rules.rs` is 38 functions that all feed `row`, `detail_rows` and `details`, and `transfer/rules.rs` is one trait and its impl for `TransactionInputType`. Splitting either would move a private helper away from its only caller and add a module layer that removes nothing.

**X53** is real and fixed: `Keychain/Types/Status.swift` was 1240 lines because 820 of them were a hand-copied English table of Apple's own `OSStatus` messages. `SecCopyErrorMessageString` returns the same message from the system, localized, so `description` is now one line and the file is 434 — the enum of status codes and nothing else.

**X54** is real and fixed: `ViewModelFactory.swift` was one struct with 55 stored services, 98 imports and 61 screen constructors. The struct and its properties stay in `ViewModelFactory.swift`; the constructors moved to `ViewModelFactory+Wallet`, `+Settings`, `+Wallets`, `+Transfer`, `+Activity`, `+Perpetuals` and `+Collectibles`, each carrying only the imports its screens need. The largest is 217 lines.

### Core hardening, second pass

**X106** closed on 2026-09-16. `block_explorer/explorer.rs` is gone: `Explorer::new(&str)` was dead outside its own tests, and the service builds `Explorer { chain }` from a real `Chain`. The four `X::from_chain(chain).unwrap()` sites in `gateway/chain_factory.rs` and `signer/chain.rs` sit inside a matching `chain_type()` arm, and that invariant is now a test — `every_chain_type_resolves_to_its_sub_chain` walks every chain and asserts the Bitcoin, EVM and Cosmos conversions exist for the type the config claims — so a new chain added with the wrong type fails in CI instead of in the app. `device.rs` keeps both: a device key must not be built from a failed OS RNG or a seed the signer rejected, and there is no safe value to fall back to.

### Core `unwrap` and `expect`

**X45** closed on 2026-09-16. The 255 it counted was 255 of nothing in particular: the sweep excluded `#[cfg(test)] mod tests` and nothing else, so every `#[cfg(all(test, feature = "chain_integration_tests"))]` module, every `#[cfg(test)] impl`, every bare `#[test]` function and every `src/testkit/` directory counted — which is why it named three integration-test modules as the leaders. The real number was 150, and 100 of those are gone: the UTXO branch of `Transaction::finalize` and the Bitcoin transaction mapper no longer index into a node response that may not carry an address, the ThorChain and Uniswap quote builders, the HyperCore EIP-712 writers and the Sui stake and transfer builders return their parse errors, a swap provider whose chain has no endpoint is left out of the list instead of panicking the whole swapper, the daemon's metric locks and the wallet-connect seen-message lock survive a poisoned mutex, `get_block_explorer` falls back to the chain's first explorer when a stored name is gone, an asset or NFT link with an unknown type is skipped instead of crashing the import, and every `SystemTime::now().duration_since(UNIX_EPOCH)` takes the zero default.

The 50 that remain are two deliberate groups. Boot wiring — the daemon's `main`, the API's stream producer, settings, search index, migrations, the localizer fallback and the six reqwest builders — is meant to stop the process, and degrading instead is a product decision, not a cleanup. The rest are infallible by construction: `Chain::from_str(self.as_ref())` on the chain enums, a const base58, BOC, address or URL parsed once behind a `LazyLock`, an HMAC that accepts any key length, and every `X::from_chain(chain).unwrap()` inside a matching `chain_type()` arm. Making that last set provable needs a total conversion keyed on `ChainType` rather than a different call at each site, and that is a type change, not a panic to remove.

### The TON verified-collection allowlist

**X39** closed on 2026-09-16 with no change beyond deleting the note. Every NFT provider derives verification from whatever its upstream gives it — OpenSea and Alchemy read a safelist status, Alchemy also reads a spam flag — and TON's token metadata carries no such field, only a `valid` flag that already gates which token info is picked and the marketplace the collection was listed on. The allowlist is that provider's signal, not a placeholder for one. Widening it is data work on the collections the API already stores (`nft_collections.is_verified`), not a gap in this crate.

### The currency field's height pin

**X33** closed on 2026-09-16 with no change. Measured on iOS 26.5, the newest runtime: unpinned, the field is 53.0 pt while it is empty and 54.7 pt as soon as it holds a digit, so the amount jumps on the first keystroke and the pin is still doing its job. `CurrencyTextFieldTests` now measures the height across four amounts, so the check is already written for whoever revisits this once the SwiftUI fix lands.

### 12. Localization hygiene

Twenty English strings exist under two keys — `wallet_send` / `transfer_send_title`, `wallet_stake` / `transfer_stake_title`, `wallet_import_address_field` / `transfer_recipient_address_field` and seventeen more. Checked on 2026-09-15: each pair carries its own context comment in `localization/app/en.ftl` and names a different place in the product, so the pairs are deliberate and must not be merged — a language that needs a different form for a label and a screen title depends on them being separate.
What went wrong in V40 and V41 was not the pair; it was one app's mapper reaching for the other half of a pair. That is only visible by comparing the two apps, which `just check-mappers` now does on every variant both apps map. The fifteen keys nothing read are gone.
