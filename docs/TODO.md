# Open work

**Refilled on 2026-09-16.** Every id from the 2026-09-15 sweeps is closed — landed, or closed against evidence, with the reasoning kept in the lower half of this file so the same lead is not re-raised. Sections 16–25 are the app-shaped pass, built from lenses the structural sweeps miss: the row census, a composed-label pass, client-side arithmetic, invented failures, call-site thresholds, collection shaping, client-side time, hand-built URLs and cross-app member-name collisions. Sections 26–30 are the Core-shaped pass over the same corpus: records that cross with a bare number, screens holding no Core service at all, rules duplicated between Core crates, the gaps the screen-service map already names, and app ports Core could own. Sections 31–37 are the structural pass — service composition depth, screens that change state with no session, chain coverage the docs do not state, the FFI surface neither app names, the boundaries that block whole families above, and the performance budgets nothing measures. Those are **L** and **M** by nature: each needs a decision before it needs a commit. No test items: coverage is tracked by the decisions a screen makes, not by file names.

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

All five closed on 2026-09-16, each against evidence rather than a re-read.

**X42** is fixed in code and needed no install data after all. `CetusAggregator` was a live provider that wrote `cetus_aggregator` into `TransactionSwapMetadata.provider`, and the stored value is a free string, so deleting the variant never broke a stored row — it broke reading one back, in the single place that parses it (`get_transaction_swap_url`). The variant is gone and `cetus_aggregator` is now an alias onto `CetusClmm` for both `FromStr` and serde, so an old Sui swap keeps its "Cetus" name and its explorer link. Neither app parses the enum from a stored row; both only pass the string to Core.

**X41** had its premise backwards. `from_locale_identifier` lowercases the language and hands the bare tag to `from_client`, so `pt-PT` reaches the `"pt"` arm and `sv-SE` reaches the long English fallback list — both are load-bearing for the clients shipping today, not legacy. The only arm the normalizer never reaches is the bare `"zh"`, because it always names the script; that one is the wire-level safety net for a raw value. A test now asserts the dependency so the list is not "cleaned up" later.

**X35** was re-tested, not re-read: the pin was removed and uniffi bumped to 0.32.1, the newest release. Both `uniffiTraitInterfaceCallAsync` sites still fail with "passing closure as a 'sending' parameter", because the generated `Task { }` captures three `@escaping` non-`Sendable` parameters. The pin is correct and the bump is not the fix; re-test after a uniffi release that changes that function.

**X36** is decided rather than deferred. `FileMigrator.migrate` returns the new URL untouched once the old path is empty, so the standing cost is one `fileExists` per launch, and the downside of deleting early is a lost wallet. It stays. Install numbers would only ever justify saving one stat call, which is not worth a tracked item.

**X40** keeps its dated note beside the route in `core/apps/api/src/devices/mod.rs`. The date is the tracker; a backlog line restating a comment that already carries the deadline is duplicate bookkeeping.

## 11. Missing tests

### Hardcoded user-visible strings

Swept on 2026-09-15 over every non-preview, non-test iOS file: 25 hits, of which 20 are the developer screen (a debug screen that is deliberately untranslated) and 4 are inside a `PreviewProvider`. The one real hit was the wallet screen's "Trade Perpetuals" row, now `perpetuals_trade`. The same sweep over Android found none. All three new keys — `perpetuals_trade`, `widget_empty` and `widget_empty_short` — were translated into the other 30 locales on 2026-09-16, each following the locale's existing `perpetuals_title` and `errors_no_data_available` wording.


T29 closed on 2026-09-16. `widget_empty` and `widget_empty_short` are translated in all 31 locales. What is left is one rule — how many coins each widget family shows — and a row model that formats a price, a percentage and a colour. Both would cost a new app-extension test target plus the package-product link that T28 found to be broken, which is more scaffolding than the rule is worth; the rule belongs in the same fix.

T28 closed on 2026-09-16 after trying it. `ScanReceiveModeViewModel` is an id and a title, `MainTabViewModel` is one `ObservableQuery`, and `RootSceneViewModel` is the composition root: thirteen collaborators and a body that is `Task { await service... }` in every method, so a test of it is a test of the mocks. The one rule worth covering — a required upgrade offers only the update action — sits behind that constructor. Wiring the existing empty `GemTests` unit-test target into `unit_frameworks.xctestplan` was tried and does not link: attaching any package product to a test bundle makes Xcode rebuild `Store` as a dynamic package product, and that product does not resolve `BigInt`, so the link fails before the tests run. That is the thing to fix first if the app target ever needs tests.

T27 closed on 2026-09-16. `ReceiveViewModel` was already covered by eleven tests — the count was by file name again. `AmountEarnViewModel` and `PerpetualModifyViewModel` decide something and are tested; `KeystoreAuthenticationViewModel` is a three-case glyph map and `ReceiveNetworkSelectorViewModel` builds its items from the ids it is handed, which is the mapper contract, not a decision.

T26 closed on 2026-09-16 with one file. `SwapTokenViewModel` decides what the pay and receive rows allow and prices what was typed, and is tested; `SwapProvidersViewModel` is a `SelectableListAdoptable` whose three members are localized constants and `SwapPairSelectorViewModel` is two optional asset ids with no logic at all — a test over either would assert the compiler.

### iOS view models with no test file

Counting by file name is what made this section long, and it was wrong three times: `Features/Transactions` (T21), `ReceiveViewModel` (T27) and `ConfirmViewModel` (T37) were all already covered from differently named files. Count the decisions, not the files.


T37 closed on 2026-09-16. `ConfirmViewModel` already had eight tests across four files named for what they cover — the header, the request, the retry and the network fee sheet — so the count by file name missed it again. `NetworkFeeCustomViewModel` was the real gap and is tested: it opens on the rate it was handed, it never lets a letter reach the rate, and a rate over the maximum cannot be confirmed.

T33 closed on 2026-09-16. `AutocloseViewModel` and `PerpetualDetailsViewModel` are tested; `PerpetualsPreviewViewModel` is two `stateIn` passthroughs over a config flag and the position list, with nothing of its own to assert.

T31 closed on 2026-09-16. Four of the five — `BuySelectViewModel`, `SendSelectViewModel`, `ManageSelectViewModel`, `ReceiveSelectViewModel` — are one-line bindings of a `GemSelectAssetType` to `BaseAssetSelectViewModel`, so the base is what was tested: the chain filter narrows the list, clearing the filters puts it back, and pinning an asset tells Core and names it in the toast. Two more were dropped after being written: the recent list reacts to `snapshotFlow { queryState.text }`, which needs Compose snapshot notifications a plain JVM test does not dispatch, and an asset carrying a balance never reaches `assetsContent` in a JVM test, so the balance filter cannot be exercised from outside the view model.

### Android view models with no test

All closed. The pattern that recurred: a feature's view models are usually one orchestrator plus several one-line bindings of an enum to it, so the orchestrator is what earns a test.


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

## 16. Presentation still decided on a client

Closed on 2026-09-16 after classifying all 27 by what their label members actually are. Ten compose a string, five are pure `Localized.` constants, and the rest declare no label member the pass could see — the census had counted computed properties of other shapes. Reading the ten found one duplicated decision, not twenty-seven.

**R58 landed.** Both apps composed the Tron resource line the same way — iOS `"\(metadata.energyAvailable) / \(metadata.energyTotal)"` in `BalanceViewModel`, Android `"${metadata.energyAvailable} / ${metadata.energyTotal}"` in `EnergyItem` — and Core had no rule for it. `balance_resource_rows` returns a row per resource with the finished text, and each app maps the resource to its own localized title, which is the mapper contract.

**The rest are not decisions written twice.** R54's proposal screen already asks Core for `shortName` and `host` through `GemApplicationMetadataService`; what is left is `AppDisplayFormatter`, an iOS-only `"Name (host)"` with no Android counterpart, because the two screens show the app differently — a product difference, not drift. The composed labels in R63, R69, R70, R72, R73, R75, R76 and R77 interpolate a Core value into a sentence whose localization lives in the app, which is where [one localized-text file per module](ARCHITECTURE.md) puts it. R55–R57, R59–R62, R64–R68, R71, R74 and R78–R80 are localized constants beside a Core value.

The lens to keep: a label that *interpolates* is worth reading, but only the ones where **both** apps compose the same shape are items. Comparing the two apps first would have cut this section from 27 to 1.


## 17. Numbers the app derives

A rendered number the app computes is the same class of bug as the fiat multiplication fixed on 2026-09-16: the app reaches a `Double` and loses Core's precision rules. These are the remaining sites the arithmetic sweep found outside `Formatters`.

The section closed on 2026-09-16. **D22 landed**: both apps hand-built the expected PnL as `"+$X (Y%)"` — iOS with `pnl >= 0 ? "+" : "-"`, Android with `if (pnl >= 0.0) "+" else "-"` — while `PriceChangeCalculator` already exported `sign` and `pnl_text`, and two other iOS screens already used them. Both call Core now. **D24** was a dead `PerpetualPortfolio.availablePeriods` extension on iOS; both apps read `available_periods` off Core's `PortfolioData`, so it is deleted.

The rest are not shared decisions. D15, D16 and D17 are the `Formatters`/`Validators` boundary (X158) and cannot call Core at all. D13, D21 and D25 are canvas geometry — SwiftUI Charts and Compose draw differently, so the axis padding and the point averaging are each app's rendering. D14 is a `seconds`/`minutes` units helper. D18, D20 and D23 pick a colour or a prefix from a tone, which is the style half of the mapper contract; the *tone* is what moved to Core, below.

D19 and D25 landed on 2026-09-16. `GemValueTone::of` was the rule both apps were rewriting — Android as `Double?.toValueDirection()` and again inline in `candleUIModel`, iOS as `PriceChangeColor.color(for: candle.close - candle.open)` — so `value_tone` is exported and all three ask Core which way a number points. The app still owns the colour and the glyph, which is the style half of the mapper contract. `PriceChangeColor` itself stays as that mapper: it lives in `Components`, which the widget depends on, and the widget cannot import Gemstone (X159).


## 18. Errors the app invents

The rule from 2026-09-16: a `try?` or a `runCatching { }.getOrNull()` that turns a real failure into a silent default is the app inventing a failure mode. `Migrations.swift` (80 sites) and the decoder probes in `AnyCodableValue.swift` are already closed as the idempotent-migration and type-probe idioms; these are the rest.

Four closed on 2026-09-16. **F24** was the rule in its purest form: `BigInt.from(string:)` was declared `throws` and had no `throw` in its body — every branch returned a value — so the two `try?` in `SwapMetadataViewModel` were guarding a failure the signature had invented. The signature is total now and the guard is gone. **F25**'s socket discarded why a request could not be built and reconnected reporting `notConnected`; it carries the real error now. **F20** is closed with no change: all four keystore sites are inside `findV3File`, which was already closed as the directory-scan idiom — the item's claim that only one of them was is wrong. **F23** is closed too: both sites are ISO-8601 parse probes where nil is the answer.

Four more closed on the same day. **F19**: the swap screen re-parsed `selection.payAssetId` with `try? AssetId(id:)`, silently dropping a selection Core had just validated; `AssetId(core:)` exists for exactly that and asserts instead. Its other two sites are the already-closed `suggestPair` and `currentInput`. **F26** found one real site among five: the chart period write was `try?` on iOS and uncaught on Android, so the same storage failure was silently ignored on one app and a crash on the other — both report it now. The other four are "this wallet has no account on that chain" and two photo loads, where nil is the answer. **F21** is the decoder-probe idiom inside generated code, and **F22**'s wallet lookups answer "the deeplink names a wallet that is gone".

The Android half (F27–F31) closes with no change: every site either logs before returning null (`WalletConnectCoordinator.activeSessions`, `HyperliquidObserverService.connection`) or is a genuine probe whose null is the answer — a locale with no currency, a route argument that is absent rather than malformed. The unpack side is guarded by `checkNotNull` with a message, so a malformed payload fails fast rather than silently.


## 19. Thresholds and limits written at a call site

A comparison against a literal in a view model is a product rule with no name. The sweep excluded layout numbers, `AuthenticationPolicy` bit flags and SQL.

Nine closed on 2026-09-16, because most of the hits are not rules. `count > 1`, `balance > 0`, `unverifiedCount > 0` and `tickCount < 2` all say "is there more than nothing here" — S22 is the clearest case: both apps ask Core `unverifiedCollections(...)` and both then check the size, so the only thing Core could add is the `> 0`, and [a lookup wrapper over uniffi is not an export](ARCHITECTURE.md). S21, S24 and S25 are the collection filtering closed in § 20, and S33 is a SQL predicate.

What is left below is the part that *is* a decision: a number someone chose, that the two apps chose differently.

- **S27** **S** `android/app/.../di/ClientsModule.kt:29-31` — connect timeout, read timeout and the idle connection pool are literals; iOS sets its own in `URLSessionConfiguration`, so the network budget is decided twice.
- **S28** **S** `android/features/update_app/.../InAppUpdateServiceImpl.kt:42-43` — a second, different pair of HTTP timeouts inside the same app.
- **S29** **S** `ios/GemPriceWidget/Widget/PriceWidgetProvider.swift:33` one minute vs `android/app/.../widgets/WidgetPriceSyncWorker.kt:34` `REFRESH_INTERVAL_MINUTES` — the widget refresh cadence is a product decision made twice.
- **S30** **S** `ios/Packages/PrimitivesComponents/.../CopyTypeViewModel.swift:63` — the pasteboard expiry interval is set in the app; Android's clipboard path has its own.
- **S31** **S** `ios/Features/WalletTab/.../WalletSearchSceneViewModel.swift:173,181` — the section caps come from Core's `limits` but the `prefix` is applied app-side on iOS only.
- **S32** **S** `ios/GemPriceWidget/.../PriceWidgetViewModel.swift:23,25` — one coin for the small family, three for the medium; the Android widget picks its own counts.

## 20. Ordering, filtering and grouping in app code

Which rows exist, and in what order, is a product decision. The sweep skipped stores and DAOs except where the query encodes a rule.

The rest of the section closed on 2026-09-16, in three groups.

**Already through Core (C15, C16, C21, C24).** Both apps call `sortedWallets` for the wallet list; both group price alerts by the asset Core says they group by; recents and validator selection go through `GemRecentActivityService` and `selectableValidators`. The lens matched the `.sorted`/`.filter` call, not a rule.

**Ordering that has to live in the query (C19, C20, and the DAO hits).** Assets are ordered by fiat total then rank — written once as a GRDB `.order` and once as `ORDER BY balanceFiatTotalAmount DESC, assetRank DESC`. Core cannot write either app's query, and doing it in memory would mean sorting a thousand rows per emission. This is the same boundary as X158 and belongs there rather than as an open item.

**C18 is blocked on a record shape, and trying it found the reason.** Both apps sort delegations by balance in memory, so it looks like the easiest item in the section — but `Gemstone.Delegation` carries only `base` and `validator`, and both apps' mappers fill `price` with nothing on the way back. Routing the sort through Core would have silently dropped the delegation price on every staking row on both platforms. It needs `price` on the Core record first, which is a shape question, not a move. C12, C14, C17, C22 and C23 are list-widget filtering with no cross-app counterpart.

C11 and C13 landed with C10. The per-asset alert list asked `type != .auto` while the alerts screen beside it already asked Core `alertKind(...).groupsByAsset()` for the same question; both ask Core now. The delegation scene was splitting Core's row list into "everything except rewards" and "rewards" inside the `View` — the boundary is the view, not the view model ([ARCHITECTURE.md](ARCHITECTURE.md)), so the split moved to `detailRows`/`rewardsRow` and the view renders what it is handed.

C10 landed on 2026-09-16. The network-assets screen split pinned from unpinned with its own `filter { $0.metadata.isPinned }` while the select-asset screen next to it already went through `AssetsSections.from`, which calls Core's `asset_sections`. It uses the same path now. Worth noting for the rest of this section: the Core rule is keyed by asset id, so it dedupes — the one test that broke was building three assets with the same id, which is not a list the screen can ever receive.


## 21. Time decided on a client

**P65 landed.** Both apps computed the support sync cursor the same way — the `createdAt` of the last message whose sender is an agent, in seconds, else zero — iOS as `query.value.last { $0.sender.isAgent }`, Android as `lastOrNull { it.sender is Agent }`. `sync_from_timestamp` owns it.

Seven more closed on 2026-09-16. **P71 and V65 were backwards**: iOS already reads both socket numbers from Core — `GemConnectionService` conforms to `Reconnectable` retroactively, and the protocol exists so `SwiftHTTPClient` can stay Gemstone-free, which is the pattern rather than a gap. **P70** is inside a `@Preview`. **P68/P69** stamp `updatedAt` while writing a row, which is the store adapter's job on both apps. **P63** groups a transcript and an activity list by *local* day through `Calendar.current`, and a local day depends on the device's calendar and time zone — that is platform territory, and Core has no day-bucketing rule to move to. **P66/P67** are a `dateComponents` helper and an epoch default for a missing `updatedAt`.

- **P64** **S** `ios/Features/Stake/.../StakeSceneViewModel.swift:121` — the unlock date is built by adding `service.lockTimeSeconds(chain:)` to now; Core has the seconds and could carry the date.

## 22. URLs built in the app

V60 closed on 2026-09-16: its five URLs are native app schemes — `tg://resolve?domain=`, `twitter://user?screen_name=`, `youtube://` — which have no Android counterpart because Android opens the same apps through intents. Core's `config/social.rs` already owns the web URLs; the scheme mapping is platform territory. 

V61–V64 closed on 2026-09-16. Of the five iOS sites said to build the asset image URL, four are `#Preview` literals and test fixtures; the only production one is `WidgetPriceService`, which builds it by hand because the widget cannot import Gemstone — that is X159, not a separate item. V62's five "hand-built URLs" are three `UIApplication.openSettingsURLString` calls and a `URL(string:)` around a URL Core already supplied. V63 and V64 are a URI opener and two composables opening a link.

- **V59** **M** `ios/Packages/GemstonePrimitives/Sources/Config.swift` (4 URLs) against `android/gemcore/.../AppUrl.kt` and `android/gemcore/.../ext/UpdateUrl.kt` — the app's own URLs are listed twice, once per platform.

## 23. Ownership: a view model holding more than its service

The rule is in [ARCHITECTURE.md](ARCHITECTURE.md) §7 and now covers stores as well as services. The store sweep is clean on both apps; these are the remaining multi-service holders.

- **O31** **S** `ios/Features/Contacts/.../ManageContactViewModel.swift` — `GemManageContactServiceProtocol` plus `GemNameServiceProtocol`, held only to pass to `ManageContactAddressViewModel`'s `AddressInputViewModel`. Decide whether a shared component's service is a port or a second service.
- **O32** **S** `ios/Features/Onboarding/.../ImportWalletViewModel.swift` — `GemWalletServiceProtocol` plus `GemNameServiceProtocol`, the same conduit shape as O31.
- **O33** **M** `ios/Gem/ViewModels/RootSceneViewModel.swift` — four Core services plus `ViewModelFactory`; the app root, and the one rule inside it (a required update offers only the update action) is unreachable from a test.
- **O34** **S** `android/features/bridge/.../WCRequestViewModel.kt` — `GemWalletConnectServiceInterface` plus `GemSignMessageServiceInterface`.
- **O35** **S** `ios/Gem/ViewModels/RootSceneViewModel.swift:41` — `currentWallet` reads `viewModelFactory.stores.walletStore.getWallet(id:)` with `try?` on every `body` pass. The session service has the async answer; making it sync would flash onboarding, so this needs a decision, not a rewrite.

## 24. Chain-specific branches in app code

N11, N12 and N13 closed on 2026-09-16 as the mapper contract working. `ChainImage`'s 14 cases and the swap provider's `hyperliquid` case are icon maps, which is the style half both apps are supposed to own. N11 is not a chain branch at all — the lens matched the word: `case .bitcoin` there is a variant of Core's `GemBannerIcon`, mapped to an image like every other variant beside it.

N10 closed on 2026-09-16, and the sweep was right that the rule was written twice with a divergence — iOS refused a multicoin wallet with no Ethereum account, Android fell back to the first account, so a malformed account list would have produced a different id on each platform. Neither is on the live path: Core's `GemWalletService` creates wallets now, iOS's `WalletId.from(type:accounts:)` had only test callers and is deleted, and Android's `WalletIdGenerator` is reached only by `Migration_63_64`, whose behaviour must stay frozen because it has already run on installed databases. A migration is the one place a rule does not get consolidated.


## 25. The About screen, decided twice

Closed on 2026-09-16. Both apps already `switch` over Core's `GemAboutRow` and Android reads `aboutSections()` — which is exactly why the label-map fingerprint paired them at 0.83. Matching variant sets is what the mapper contract *looks like*; the fingerprint cannot tell a shared Core enum from a decision made twice, so a pair is only an item when neither side names a Core type.


## 26. Records that hand the app a bare number

Closed on 2026-09-16 after reading every one. The lens asked the wrong question: it matched on the *field* being a number, when the violation is the app *deciding* something from it. Three classes came back, none of them a decision written twice.

**Inputs, not outputs.** `GemAssetDetailsInput.price`, `GemFiatQuoteRequest.amount`, `GemPerpetualTransferData.price`/`leverage`, `GemAutocloseField.price`/`original_price`, `GemPriceAlertSession.current_price` and `GemBannerContext.asset_rank_score` are all built *by* the app and handed *to* Core. A number going in is the app telling Core what the user did.

**Values the app must render with its own locale.** `GemFormattedNumber.value` is carried beside the finished text on purpose — both apps format it through their own number formatter, which is the divergence X158 names. `GemDurationPart.value` is the same shape and symmetric on both apps. `GemRewardsState.invite_reward_points` is interpolated into a sentence whose localization lives in the app, not in Core's `localizer`.

**Values nothing renders.** `GemBalanceValue.amount` is written to the database by a store adapter, `GemPriceUpdate` is a store write, and `ChainConfig`'s activation fees and `GemChart.base_value` are named by no app source at all.

The one real hit was `GemPerpetualChartLayout`, and not for its numbers: `price_low`/`price_high` are axis bounds Core is right to carry, but both apps derived the candle's direction from `close - open` themselves. That is D19/D25 and it landed — see § 17.


## 27. Screens with no Core service

156 iOS view models name no `Gem*ServiceProtocol`; Android has 8. The asymmetry looked like the shape of the gap — until the section's own test was applied to all 55 on 2026-09-16.

**43 of them are the allowed shape and are closed.** Each is a `struct` with no observable state, no query and no async work, whose initializer takes Core records and projects them. `NetworkFeeSceneViewModel` was called the heaviest model in the repo with no service; it is a value type handed `GemConfirmFeeSelection`, `GemFeeRateRows` and `GemFeeOptionItem` plus two callbacks. A value type projecting Core records does not need a service, and the member count says nothing about whether it decides anything. The weight column was measuring size, not ownership.

**The last twelve closed on 2026-09-16 too, and one of them was worth the whole section.**

**B25 landed.** Both apps shuffle the recovery phrase *within groups of four* so the user re-picks the words in order — iOS as `words.shuffleInGroups(groupSize: 4)`, Android as its own chunk-and-shuffle loop with a private `wordsPerGroup = 4`. The same rule, written twice, on the wallet-recovery surface, where the two implementations drifting means one platform verifying a phrase the other would not. `phrase_verification_words` owns it now, and both screens take the shuffled list from the service that creates the wallet.

Five of the rest already read Core and map it: `AutocloseSceneViewModel`, `LockSceneViewModel`, `SwapDetailsViewModel`, `TransactionsFilterViewModel` and `NetworkFeeCustomViewModel` — B23 in particular looked like a gap because Android has a `CustomFee` domain class iOS lacks, but both call `GemCustomFee.estimate` and read `isOverMax`, `isBelowMinimum`, `isValid`, `feeValue`, `maxRate` and `minimumRate` from it; only the rate *text* is formatted per app, which is X158. `CoinPriceRowViewModel` is the widget (X159) and `InputValidationViewModel` sits behind the `Validators` boundary (X158). `QRScannerSceneViewModel`, `TextInputViewModel` and `SupportMessageInputBarViewModel` are camera, keyboard and attachment plumbing, and `PerpetualsPreviewViewModel` is two query passthroughs.


## 28. Core duplicated inside Core

Closed on 2026-09-16 after reading all thirteen. Twelve were the *convention*, not duplication: `calculate_transaction_fee`, `calculate_fee_rates` and `calculate_network_apy` take different arguments and compute different chains' fees; `create_staking_client` is a testkit helper per chain; `checksum_address`, `deposit_addresses`, `chain_from_id` and `for_chain` are per-provider tables; `config_session_properties` is a service forwarding to the collaborator it composes, which is how composition reads; `has_price`/`has_size`/`execution_error` are a primitive and its accessor. Naming the same operation the same way across chain crates is what makes them readable side by side — the lens cannot tell that apart from a copy, so match on body shape and signature, not on name.

The one real copy was `create_eth_client`, identical in `swapper` and `yielder` down to the `EVMChain::from_chain(...).ok_or(...)` line and differing only in error type. `EthereumClient::for_chain` now owns it in `gem_evm`, which both crates already depend on, and each factory maps the `None` to its own error.


## 29. Gaps the screen-service map already names

[SERVICES.md](SERVICES.md) says a screen service only one app holds is the next consolidation. These are the rows where the table itself shows one side empty or asymmetric.

Five were confirmations rather than decisions and are closed. **P78**: both compositions forward identically to `self.banners.banner_content(event, asset)`, so the banner rules have not drifted. **P76**: all three screens ask `getChains(query:)` — the same question. **P77**: iOS's third `GemRecentActivityService` holder does not hold it; `SelectAssetViewModel` takes it in `init` and passes it straight to a child model, the conduit shape O31 and O32 describe. **P80**: the iOS delegation filter was closed with § 19. **P81** was already finished as § 35 — the trace is done and the answer is two.

**P76 turned up something the sweep did not name.** `ImportWalletTypeViewModel` reaches `GemChainService.shared` at file scope while its sibling `ChainListSettingsViewModel` takes the same service in its initializer, and thirteen more iOS sites do the same with `GemAddressService`, `GemAssetConfigService`, `GemApplicationMetadataService` and `GemConnectionService`. SERVICES.md forbids a file-scope `Gem*Service` so a test can substitute it, but every one of these is a stateless rule object with no constructor arguments, so injecting it into fourteen initializers buys no substitutability. That is one decision — does a stateless Core rule object count as a service under § 7, or as a free function — and it should be settled once rather than fourteen times.

- **P72** **M** `GemAppUpdateService` — iOS `AboutUsViewModel` holds it; Android uses Play in-app update instead, so the update decision is made by two different owners. `AppUpdateCoordinator` already maps `upgradeRequired` itself (see F29).
- **P73** **M** `GemAvatarService` — Android has no avatar surface at all, so wallet avatars are an iOS-only feature rather than a Core one.
- **P74** **S** `GemNotificationsService` — iOS `NotificationsViewModel` holds it; Android's `SettingsViewModel` uses push cases instead.
- **P75** **S** `GemTransactionDetailsService` — iOS holds the service, Android reaches the same answer through `GetTransactionDetailsImpl` as an observed read, so the links are built in two places.
- **P79** **S** `GemWalletSessionService` — iOS spreads it over `RootSceneViewModel` and `NavigationHandler`; Android keeps it in `SessionCoordinator`. The iOS split is what produced O35.

## 30. App ports that Core could own

V66, V67 and V71 closed on 2026-09-16 as platform ports rather than thin wrappers: `ConnectionComponentMonitoring` has two conformers and `WebSocketRequestProvider` has two, so they carry real polymorphism over platform APIs, and `UriHandler.open` is Chrome Custom Tabs with a fallback, which Core cannot express. V70 and V72 restated decisions already open as P72 and V59 and are folded into them.

- **V68** **S** `ios/Packages/Store/Sources/BindableQuery.swift` — a one-method protocol behind every observed read on iOS; Android has narrow cases instead. Worth one decision about which shape both apps use.
- **V69** **M** `ios/Packages/Formatters` and `ios/Packages/Validators` cannot import Gemstone, which is what keeps D15–D17 duplicated. The item is the dependency, not the formatter: decide whether the widget and these two packages get a Gemstone-free Core surface or move under one that can import it.

## 31. Services with more collaborators than a service should have

29 `uniffi::Object` services hold four or more `Arc` collaborators. Composition is the sanctioned answer to "a screen needs several owners" ([ARCHITECTURE.md](ARCHITECTURE.md) § 7), so depth alone is not a defect — but a service that composes a dozen others is the place a cycle appears, and it is the hardest thing in Core to change without touching every screen. Each item is one service to read for a responsibility that belongs to a collaborator.

- **X107** **L** `GemStreamService` — 13 collaborators (`GemBalanceService`, `GemDeviceService`, `GemFiatService`, `GemNftService`, `GemNotificationStore`, `GemPerpetualService` and seven more). The socket fan-out is the widest object in Core, and every screen's freshness depends on it.
- **X108** **L** `GemWalletService` — 10, spanning the keystore, the avatar, the explorer and the file store. Wallet creation, naming, avatars and secret export are one object.
- **X109** **L** `GemAssetDetailsService` — 10; the asset screen composes banners, deeplinks, price alerts and balances into one answer.
- **X110** **M** `GemPerpetualService` — 9, including the gateway and two stores.
- **X111** **M** `GemTransactionsService` — 8, including the device API client and the status service.
- **X112** **M** `GemConfirmTransferService` — 8; the confirm flow reaches the keystore password, the name service and the asset config.
- **X113** **M** `GemConfirmService` — 8, including simulation and scanning. Confirm is split across two eight-collaborator services; decide whether that split is the right seam.
- **X114** **M** `GemAssetSelectionService` — 8; asset selection composes perpetuals, price alerts and recent activity.
- **X115** **M** `GemStakeService` — 7.
- **X116** **M** `GemDeveloperService` — 7 after the 2026-09-16 migration; the developer screen is now the widest debug surface in Core.
- **X117** **M** `GemAppStartService` — 7; launch orchestration.
- **X118** **M** `GemWalletHomeService` — 6.
- **X119** **M** `GemWalletConnectService` — 6, including its own signer and simulation.
- **X120** **M** `GemTransactionStateService` — 6.
- **X121** **M** `GemBalanceService` and `GemAssetsService` — 6 each, and they compose each other's neighbours; the balance/assets pair is worth reading as one seam.
- **X122** **M** `GemAssetDiscoveryService` — 6.

## 32. A screen that changes state with no session

[ARCHITECTURE.md](ARCHITECTURE.md) says a screen whose state changes is a session. 83 iOS view models declare six or more `var`s and name no `Gem*Session`; Android has two.

**Nine are closed**: they have no observable state at all — `@Observable`, `MutableStateFlow` and `mutableStateOf` are all absent — so the `var` count was counting computed properties on a value type. A screen with no state that changes has nothing to model as a session.

**Fifteen more closed on 2026-09-16 after counting the state properly.** The original "six or more `var`s" counted computed properties — `var title: String { Localized... }` is not state. Counting only *stored* mutable members, and setting aside the presentation ones (`isPresenting*`, toasts, alerts, focus, scroll):

- Eight hold no domain state at all — `SwapDetailsViewModel` and `EarnSceneViewModel` have zero stored `var`s of any kind, and `SignMessageSceneViewModel`, `StakeSceneViewModel`, `PreferencesViewModel`, `PerpetualSceneViewModel`, `NetworkAssetsSceneViewModel` and `ConnectionsViewModel` hold only sheets and toasts. A screen with no state that changes has no session to model.
- Six hold exactly one: an input string, a loading flag, an image-loaded flag, a request, a release, a name field. One field is not a lifetime.
- `PerpetualDetailsViewModel` already reaches a session.

**Thirteen are left**, each with two to six pieces of real domain state — `SelectAssetViewModel` has six, `ConfirmTransferSceneViewModel` and `ImportWalletSceneViewModel` four each. Those are where the session question is real, and it is a shape to agree rather than a move to make.

- **S34** **L** `Features/Settings/RewardsViewModel.swift` [40] — the widest stateful screen with no session: wallet selection, sheets, alerts, toasts and the rewards state.
- **S35** **L** `Features/WalletTab/WalletSearchSceneViewModel.swift` [32].
- **S37** **L** `Features/Assets/SelectAssetViewModel.swift` [26].
- **S38** **L** `Features/Transfer/ConfirmTransferSceneViewModel.swift` [25] — confirm has `GemConfirmation`, which SERVICES.md explicitly calls not a session; this is the item that decides whether that is still right.
- **S40** **M** `Features/Transfer/AmountSceneViewModel.swift` [24].
- **S47** **M** `Features/Onboarding/ImportWalletSceneViewModel.swift` [19] — wallet import state, a recovery-critical flow.
- **S48** **M** `Features/Contacts/ManageContactViewModel.swift` [19].
- **S53** **M** `Features/Perpetuals/PerpetualsSceneViewModel.swift` [16] and `PerpetualPositionViewModel.swift` [16].
- **S54** **M** `Features/Transfer/ReceiveViewModel.swift` [15].
- **S59** **M** `Features/Settings/SecurityViewModel.swift` [14] — a security surface whose lock-period state produced a crash on 2026-09-15.
- **S60** **M** `Features/AppLock/LockSceneViewModel.swift` [14] — the lock screen state machine.
- **S63** **S** `Features/Transfer/RecipientSceneViewModel.swift` [13].
- **S69** **S** `Features/Onboarding/VerifyPhraseViewModel.swift` [9] and `Features/Contacts/ManageContactAddressViewModel.swift` [9].

## 33. Chain coverage the matrix does not state

A chain crate that does not implement a trait its siblings do is either a chain that cannot do that thing or a gap nobody wrote down.

X124–X132 are closed on 2026-09-16 after reading every arm the sweep found. All of them are fail-closed: the Aptos signer's five arms return `SignerError::InvalidInput`, the Uniswap, Across and Relay deployment tables return `None` for a chain with no deployment, Chainflip returns `SwapperError::NotSupportedChain`, GoPlus returns an `Err` naming the chain, and the five fiat mappers return `FiatTransactionStatus::Unknown`, which is a named outcome rather than a silent default. `EvmStakingClient` is absent from every non-EVM chain because its name states the constraint. The lens counts a defaulting arm; it cannot see that the default *is* the answer, so exclude `None`/`false`/`Err`/`Unknown` arms in provider tables next time and keep only arms that pick a wrong concrete value.

- **X123** **M** `gem_bitcoin`, `gem_cosmos`, `gem_ton` and `gem_hypercore` implement `ChainTraits` but not `ChainProvider`; `gem_bsc`, `gem_monad` and `gem_optimism` implement `ChainProvider` but not `ChainTraits`. Two overlapping abstractions with different membership.

## 34. Files that are a table and a rule set at once

The 2026-09-16 pass closed the "outgrown one module" items by measuring what each length was made of. These are the ones where the length is a table *and* rules, so splitting is real work rather than bookkeeping.

- **X133** **L** `core/gemstone/src/services/confirm/rules.rs` (1474 lines) — the confirm rules, the widest rule file in Core and the one two eight-collaborator services share.
- **X134** **L** `core/gemstone/src/services/perpetual/rules.rs` (1447).
- **X135** **L** `core/crates/primitives/src/chain_config.rs` (1389) — 102 `ChainConfig` literals plus the config types; the table half is generated-shaped and could be data.
- **X136** **L** `core/gemstone/src/services/transactions/rules.rs` (1313).
- **X137** **L** `core/gemstone/src/services/stake/rules.rs` (1272).
- **X138** **L** `core/gemstone/src/services/amount/rules.rs` (1251).
- **X139** **L** `core/gemstone/src/services/transfer/rules.rs` (1191).
- **X140** **L** `core/gemstone/src/services/assets/rules.rs` (1173).
- **X143** **M** `core/crates/swapper/src/stonfi/provider.rs` (1011), `across/provider.rs` (915), `chainflip/provider.rs` (906) — three swap providers over 900 lines each; compare their shapes before splitting any one.
- **X144** **M** `core/crates/gem_tron/src/signer/chain_signer.rs` (976) — the widest chain signer.
- **X145** **M** `core/crates/gem_hypercore/src/provider/perpetual_mapper.rs` (925).

## 35. The FFI surface neither app names

Closed on 2026-09-16 by finishing the trace instead of re-listing it. Of 413 exported records and enums, 351 are named by an app and 275 appear in an exported signature; seeding reachability from *both* sets and walking field types leaves 411 reachable. "Named by neither app" was never the right question — a record crosses because something that crosses carries it.

Exactly two were unreachable and are now un-exported: `GemSwapButtonInput` was built and consumed inside `GemSwapSession::button_action`, with only `GemSwapButtonAction` crossing, so its `#[uniffi::export] impl` generated bindings nothing could call; `GemTransferOutput` is an internal trait return. Both keep their Rust callers and no longer appear in `Gemstone.swift` or `gemstone.kt`.

The trace to keep: seed from app-named **and** signature-named types, then walk field types transitively. Seeding from app-named alone reports 12 false orphans.


## 36. Boundaries that block the rest

Each of these is one decision that unblocks a family of items above. They are listed last because none is a code change until the decision is made.

- **X158** **L** `ios/Packages/Formatters` and `ios/Packages/Validators` cannot depend on Gemstone, which is what keeps D15–D17 duplicated and what left `GemPriceWidget` (B50) without Core. Decide whether these packages move under a target that can import Gemstone, or Core grows a dependency-free surface for them.
- **X159** **M** `GemPriceWidget` is a separate target with its own `SharedPreferences` and `WidgetPriceService`; the Android widget has its own too (S29, S32). A widget that cannot call Core is a second implementation of the price screen on each platform.
- **X160** **M** iOS keeps a family of small row view models (156 with no service, § 27) where Android injects a service into almost every model. That is a platform-shaped difference in where presentation is decided; pick one shape and write it in ARCHITECTURE.md before migrating the individual models.
- **X161** **M** `BindableQuery`/`ObservableQuery` on iOS against Android's narrow application cases (V68) — two answers to "how does a screen watch the database", and every observed read on both apps sits on one of them.
- **X162** **M** The `GemTests` target cannot link package products because Xcode rebuilds `Store` as a dynamic product that does not resolve `BigInt` — recorded when T28 closed. Until that is fixed the app target has no unit tests, which is what keeps `RootSceneViewModel` (O33, S34-adjacent) untestable.
- **X163** **M** iOS pins the `Gemstone` package to Swift 5 language mode; re-tested on 2026-09-16 against uniffi 0.32.1 and the two `uniffiTraitInterfaceCallAsync` sites still fail. Until uniffi changes that function, the generated bindings cannot be Swift 6 clean, and neither can anything downstream that would otherwise adopt strict concurrency.
- **X164** **M** `core/apps/api` and `core/apps/daemon` share `primitives` and `storage` with the mobile FFI, so a `primitives` change is a server change. Issue #1202 holds the measured baseline and the phased plan; the item is to decide whether the mobile surface gets its own crate boundary.
- **X165** **M** Two services own confirm (`GemConfirmService`, `GemConfirmTransferService`, X112/X113) and two own assets (`GemAssetsService`, `GemAssetDetailsService`, X109/X121). Both pairs predate the screen-service rule; decide the seam before the § 31 items are started.

## 37. Performance budgets nothing measures yet

[PERFORMANCE.md](PERFORMANCE.md) sets p95 ≤ 100 ms to first feedback, ≤ 200 ms to useful content from local data, and ≤ 100 ms from a received update to the frame.

Attempted on 2026-09-16 and the attempt is the finding. `am start -W` over eight cold launches of the debug build on the API 17 emulator gives a median `TotalTime` of 4.7 s (range 3.7–8.2 s), and `dumpsys gfxinfo` right after launch reports 5 frames, 100% janky — statistically empty. Neither number is evidence: the debug build has no R8, and PERFORMANCE.md § How to test says measurement needs release-like builds on physical devices, with 5 warmups and 30 measured runs.

The prerequisite is that **no harness exists**: there is no Macrobenchmark module in `android/settings.gradle.kts` and no XCTest metric target on iOS, and the doc's own escape hatch — "until a benchmark harness exists for a journey, record the manual profiler setup and steps" — is what these nine items were. Adding a benchmark module and wiring it into CI is scaffolding to agree before any of the nine can produce a number worth comparing. The eight launch samples above are recorded so the harness has something to sanity-check against.


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
