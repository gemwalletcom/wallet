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

Rebuilt on 2026-09-16 from the row census and a composed-label pass: a `*ViewModel`/`*UIModel` that declares three or more label-shaped `String` members and names no Core `Row`/`ViewState`/`Details`/`Sections` record is deciding its own presentation, and a label whose body interpolates, concatenates or branches is a decision rather than a lookup. Each item names the file the sweep hit; confirm the member before starting, because a member that is one `Localized.` constant is the mapper contract working.

- **R54** **M** `ios/Features/WalletConnector/.../ConnectionProposalViewModel.swift` — 11 label members and no Core record; `appName` and `websiteText` are both composed. Android reads `GemConnectionRow`/`GemConnectionDetails` for the same screen.
- **R55** **M** `ios/Packages/PrimitivesComponents/.../AssetDataViewModel.swift` — 10 label members. The asset row record already exists in Core for the list; the detail screen still composes its own.
- **R56** **M** `ios/Features/Stake/.../DelegationViewModel.swift` — 8 label members plus `rewardsText`, which Android also declares in `StakeViewModel.kt`.
- **R57** **M** `ios/Features/Transfer/.../AmountSceneViewModel.swift` — 7 label members; `assetName` is declared on Android too, in `RecipientError.kt`.
- **R58** **S** `ios/Packages/PrimitivesComponents/.../BalanceViewModel.swift` — 6 members; `energyText` and `bandwidthText` are composed from a Tron resource pair that Core already models.
- **R59** **S** `ios/Packages/PrimitivesComponents/.../AddressListItemViewModel.swift` — 6 members and no Core record.
- **R60** **S** `ios/Features/Onboarding/.../ImportWalletSceneViewModel.swift` — 6 members; the import type list is a product decision.
- **R61** **S** `ios/Features/Contacts/.../ManageContactViewModel.swift` — 6 members beside `GemManageContactService`, which already answers the screen.
- **R62** **S** `ios/Packages/PrimitivesComponents/.../AssetViewModel.swift` — 5 members; the canonical asset title lives in Core.
- **R63** **S** `ios/Features/Settings/.../RewardRedemptionOptionViewModel.swift` — 5 members; the redemption rows landed in Core (b638ab54a9) but this model still composes.
- **R64** **S** `ios/Features/MarketInsight/.../MarketValueViewModel.swift` — 5 members over market statistics Core already carries.
- **R65** **S** `ios/Packages/PrimitivesComponents/.../ChartHeaderViewModel.swift` — 4 members, and `dateText`/`headerValueText` are both declared on Android in `ChartHeaderUIModel.kt`. The same header is composed twice.
- **R66** **S** `ios/Features/Swap/.../PriceImpactViewModel.swift` — 4 members; `showsInSummary` is also declared in Android's `SwapDetailsUIModel.kt`.
- **R67** **S** `ios/Features/Support/.../SupportChatSceneViewModel.swift` — 4 members over a chat transcript Core already groups.
- **R68** **S** `ios/Packages/PrimitivesComponents/.../PerpetualDetailsViewModel.swift` — `positionText`, `leverageText` and `listItemSubtitle` are all composed.
- **R69** **S** `ios/Features/Perpetuals/.../AutocloseViewModel.swift` — `title`, `profitTitle` and `percentText` are composed beside a Core autoclose session that already returns a view state.
- **R70** **S** `ios/Features/Assets/.../AssetSceneViewModel.swift` — `pinText` and `enableText` branch on state to pick a verb.
- **R71** **S** `ios/Features/Perpetuals/.../PerpetualSceneViewModel.swift` — `navigationTitle` is composed from the pair name.
- **R72** **S** `ios/Features/PriceAlerts/.../SetPriceAlertViewModel.swift` — `completeMessage` joins a lowercased direction label with an amount, which is sentence construction in the app.
- **R73** **S** `ios/Features/Settings/Currency/.../CurrencyViewModel.swift` — `title` falls back through `Locale.current.localizedString(forCurrencyCode:)`; Android has its own currency naming.
- **R74** **S** `ios/Features/Transfer/.../ReceiveViewModel.swift` — `copyTitle` and `warningMessage` are composed; the warnings come from Core but the joining does not.
- **R75** **S** `ios/Features/WalletTab/.../PortfolioSceneViewModel.swift` — `navigationTitle` and the statistic `title` are chosen in the model.
- **R76** **S** `ios/Packages/PrimitivesComponents/.../SimulationPayloadFieldViewModel.swift` — `subtitle` picks between an address name and a raw value.
- **R77** **S** `ios/Packages/PrimitivesComponents/.../TransactionViewModel.swift` — `title` is chosen by a badge flag and again by a `switch` over the row subtitle.
- **R78** **S** `android/ui-models/.../chart/ChartHeaderUIModel.kt`, `chart/CandlestickChartUIModel.kt` — the Android half of R65, same three label members.
- **R79** **S** `android/features/bridge/.../WCRequestViewModel.kt` — three label members and two Core services; the other half of R54.
- **R80** **S** `android/ui-models/.../perpetual/autoclose/AutocloseUIModel.kt` — the Android half of R69.

## 17. Numbers the app derives

A rendered number the app computes is the same class of bug as the fiat multiplication fixed on 2026-09-16: the app reaches a `Double` and loses Core's precision rules. These are the remaining sites the arithmetic sweep found outside `Formatters`.

D19 and D25 landed on 2026-09-16. `GemValueTone::of` was the rule both apps were rewriting — Android as `Double?.toValueDirection()` and again inline in `candleUIModel`, iOS as `PriceChangeColor.color(for: candle.close - candle.open)` — so `value_tone` is exported and all three ask Core which way a number points. The app still owns the colour and the glyph, which is the style half of the mapper contract. `PriceChangeColor` itself stays as that mapper: it lives in `Components`, which the widget depends on, and the widget cannot import Gemstone (X159).

- **D13** **S** `ios/Packages/Primitives/Sources/ChartValues.swift:42` — the x-axis is padded by `timeIntervalSince(first) * 0.02`; Core already owns candlestick geometry (08f9789016).
- **D14** **S** `ios/Packages/Components/Sources/Interval.swift:13` — `Interval(value) * 60` converts minutes in the app.
- **D15** **S** `ios/Packages/Primitives/Sources/Extensions/Double+Primitives.swift` `rounded` and `android/gemcore/.../ValueFormatter.kt` `rounded` — the same rounding helper on both apps.
- **D16** **S** `ios/Packages/Formatters/.../BigNumberFormatter.swift` `decimal` and `android/gemcore/.../NumericFormatter.kt` `decimal` — the same decimal parse on both apps.
- **D17** **S** `ios/Packages/Formatters/.../ValueFormatter.swift` and `android/gemcore/.../ValueFormatter.kt` both declare `formattedDustThreshold`; the dust *predicate* is Core's `is_value_dust`, the *threshold text* is written twice.
- **D18** **S** `ios/Packages/PrimitivesComponents/Sources/Types/AmountDisplay.swift:131` — the sign prefix is picked from `value > 0` / `value < 0`; Android does the same in `AutocloseUIModelFactory.kt:70` and `GetWalletSummaryImpl.kt:125`.
- **D20** **S** `android/ui/.../list_item/transaction/TransactionDataAggregateExt.kt:57` and `android/app/.../widgets/PricesWidget.kt:163` — profit colour chosen from `pnl > 0` in two places, with the widget hardcoding hex colours.
- **D21** **S** `android/ui/.../chart/GemLineChart.kt:341,366` — chart point averaging and x-position are computed in the composable.
- **D22** **S** `ios/Features/Perpetuals/.../AutocloseViewModel.swift:44,59` — `isProfit` and the sign are derived beside a Core estimator that already answers both; `android/ui-models/.../AutocloseUIModelFactory.kt:55,70` is the same rule.
- **D23** **S** `ios/Packages/PrimitivesComponents/.../PriceViewModel.swift:60` — the price-change background colour branches on `priceChange > 0`.
- **D24** **S** `ios/Features/WalletTab/.../PortfolioSceneViewModel.swift` and `android/.../PortfolioChartViewModel.kt` both declare `availablePeriods`; Core's `PortfolioData` already carries them.

## 18. Errors the app invents

The rule from 2026-09-16: a `try?` or a `runCatching { }.getOrNull()` that turns a real failure into a silent default is the app inventing a failure mode. `Migrations.swift` (80 sites) and the decoder probes in `AnyCodableValue.swift` are already closed as the idempotent-migration and type-probe idioms; these are the rest.

Four closed on 2026-09-16. **F24** was the rule in its purest form: `BigInt.from(string:)` was declared `throws` and had no `throw` in its body — every branch returned a value — so the two `try?` in `SwapMetadataViewModel` were guarding a failure the signature had invented. The signature is total now and the guard is gone. **F25**'s socket discarded why a request could not be built and reconnected reporting `notConnected`; it carries the real error now. **F20** is closed with no change: all four keystore sites are inside `findV3File`, which was already closed as the directory-scan idiom — the item's claim that only one of them was is wrong. **F23** is closed too: both sites are ISO-8601 parse probes where nil is the answer.

Four more closed on the same day. **F19**: the swap screen re-parsed `selection.payAssetId` with `try? AssetId(id:)`, silently dropping a selection Core had just validated; `AssetId(core:)` exists for exactly that and asserts instead. Its other two sites are the already-closed `suggestPair` and `currentInput`. **F26** found one real site among five: the chart period write was `try?` on iOS and uncaught on Android, so the same storage failure was silently ignored on one app and a crash on the other — both report it now. The other four are "this wallet has no account on that chain" and two photo loads, where nil is the answer. **F21** is the decoder-probe idiom inside generated code, and **F22**'s wallet lookups answer "the deeplink names a wallet that is gone".

The Android half (F27–F31) closes with no change: every site either logs before returning null (`WalletConnectCoordinator.activeSessions`, `HyperliquidObserverService.connection`) or is a genuine probe whose null is the answer — a locale with no currency, a route argument that is absent rather than malformed. The unpack side is guarded by `checkNotNull` with a message, so a malformed payload fails fast rather than silently.


## 19. Thresholds and limits written at a call site

A comparison against a literal in a view model is a product rule with no name. The sweep excluded layout numbers, `AuthenticationPolicy` bit flags and SQL.

- **S19** **S** `ios/Features/Assets/Sources/Types/AddAssetInput.swift:12` `chains.count > 1`, `ios/Features/Onboarding/.../ImportWalletSceneViewModel.swift:87` `importTypes.count > 1`, `ios/Features/Settings/.../RewardsViewModel.swift:107` `wallets.count > 1` — "more than one, so offer a picker" written three times on iOS.
- **S20** **S** `ios/Packages/PrimitivesComponents/.../AssetDataViewModel.swift:101,117` — "has a balance" as `> 0` in two places; `ios/Features/Perpetuals/.../PerpetualsHeaderViewModel.swift:59` and `ios/Features/Assets/.../AssetSceneViewModel.swift:201` repeat it.
- **S21** **S** `ios/Features/Stake/.../EarnSceneViewModel.swift:92` — `.filter { BigInt($0.base.balance) > 0 }` decides which delegations show.
- **S22** **S** `ios/Features/NFT/.../CollectionsViewModel.swift:40` and `android/features/nft/.../NftListScene.kt:116` — the unverified-collections row appears when the count is above zero, decided on both apps.
- **S23** **S** `ios/Features/Swap/Sources/Types/SwapValueFormatter.swift:17` — a zero guard in front of swap value text.
- **S24** **S** `android/data/services/store/.../entities/DbAssetInfo.kt:124,146` — resource metadata and price presence decided by `> 0` while mapping a row out of the database.
- **S25** **S** `android/features/asset_select/.../BaseAssetSelectViewModel.kt:126` — the balance filter is `it.balance.totalAmount > 0.0` in the view model.
- **S26** **S** `android/ui-models/.../chart/CandlestickChartUIModel.kt:57` — `if (tickCount < 2) return emptyList()`.
- **S27** **S** `android/app/.../di/ClientsModule.kt:29-31` — connect timeout, read timeout and the idle connection pool are literals; iOS sets its own in `URLSessionConfiguration`, so the network budget is decided twice.
- **S28** **S** `android/features/update_app/.../InAppUpdateServiceImpl.kt:42-43` — a second, different pair of HTTP timeouts inside the same app.
- **S29** **S** `ios/GemPriceWidget/Widget/PriceWidgetProvider.swift:33` one minute vs `android/app/.../widgets/WidgetPriceSyncWorker.kt:34` `REFRESH_INTERVAL_MINUTES` — the widget refresh cadence is a product decision made twice.
- **S30** **S** `ios/Packages/PrimitivesComponents/.../CopyTypeViewModel.swift:63` — the pasteboard expiry interval is set in the app; Android's clipboard path has its own.
- **S31** **S** `ios/Features/WalletTab/.../WalletSearchSceneViewModel.swift:173,181` — the section caps come from Core's `limits` but the `prefix` is applied app-side on iOS only.
- **S32** **S** `ios/GemPriceWidget/.../PriceWidgetViewModel.swift:23,25` — one coin for the small family, three for the medium; the Android widget picks its own counts.
- **S33** **S** `android/data/services/store/.../PerpetualDao.kt:34` — `WHERE volume24h > 0` decides which markets exist, in SQL.

## 20. Ordering, filtering and grouping in app code

Which rows exist, and in what order, is a product decision. The sweep skipped stores and DAOs except where the query encodes a rule.

The rest of the section closed on 2026-09-16, in three groups.

**Already through Core (C15, C16, C21, C24).** Both apps call `sortedWallets` for the wallet list; both group price alerts by the asset Core says they group by; recents and validator selection go through `GemRecentActivityService` and `selectableValidators`. The lens matched the `.sorted`/`.filter` call, not a rule.

**Ordering that has to live in the query (C19, C20, and the DAO hits).** Assets are ordered by fiat total then rank — written once as a GRDB `.order` and once as `ORDER BY balanceFiatTotalAmount DESC, assetRank DESC`. Core cannot write either app's query, and doing it in memory would mean sorting a thousand rows per emission. This is the same boundary as X158 and belongs there rather than as an open item.

**C18 is blocked on a record shape, and trying it found the reason.** Both apps sort delegations by balance in memory, so it looks like the easiest item in the section — but `Gemstone.Delegation` carries only `base` and `validator`, and both apps' mappers fill `price` with nothing on the way back. Routing the sort through Core would have silently dropped the delegation price on every staking row on both platforms. It needs `price` on the Core record first, which is a shape question, not a move. C12, C14, C17, C22 and C23 are list-widget filtering with no cross-app counterpart.

C11 and C13 landed with C10. The per-asset alert list asked `type != .auto` while the alerts screen beside it already asked Core `alertKind(...).groupsByAsset()` for the same question; both ask Core now. The delegation scene was splitting Core's row list into "everything except rewards" and "rewards" inside the `View` — the boundary is the view, not the view model ([ARCHITECTURE.md](ARCHITECTURE.md)), so the split moved to `detailRows`/`rewardsRow` and the view renders what it is handed.

C10 landed on 2026-09-16. The network-assets screen split pinned from unpinned with its own `filter { $0.metadata.isPinned }` while the select-asset screen next to it already went through `AssetsSections.from`, which calls Core's `asset_sections`. It uses the same path now. Worth noting for the rest of this section: the Core rule is keyed by asset id, so it dedupes — the one test that broke was building three assets with the same id, which is not a list the screen can ever receive.

- **C18** **S** `android/data/coordinators/.../stake/StakeReadsImpl.kt` — delegation filtering beside a Core stake service.

## 21. Time decided on a client

Seven closed on 2026-09-16. **P71 and V65 were backwards**: iOS already reads both socket numbers from Core — `GemConnectionService` conforms to `Reconnectable` retroactively, and the protocol exists so `SwiftHTTPClient` can stay Gemstone-free, which is the pattern rather than a gap. **P70** is inside a `@Preview`. **P68/P69** stamp `updatedAt` while writing a row, which is the store adapter's job on both apps. **P63** groups a transcript and an activity list by *local* day through `Calendar.current`, and a local day depends on the device's calendar and time zone — that is platform territory, and Core has no day-bucketing rule to move to. **P66/P67** are a `dateComponents` helper and an epoch default for a missing `updatedAt`.

- **P64** **S** `ios/Features/Stake/.../StakeSceneViewModel.swift:121` — the unlock date is built by adding `service.lockTimeSeconds(chain:)` to now; Core has the seconds and could carry the date.
- **P65** **S** `ios/Features/Support/.../SupportChatSceneViewModel.swift:51` — the sync cursor is derived from the last agent message's timestamp in the model.

## 22. URLs built in the app

- **V59** **M** `ios/Packages/GemstonePrimitives/Sources/Config.swift` (4 URLs) against `android/gemcore/.../AppUrl.kt` and `android/gemcore/.../ext/UpdateUrl.kt` — the app's own URLs are listed twice, once per platform.
- **V60** **S** `ios/Packages/PrimitivesComponents/.../DeepLinkViewModel.swift` (5 URLs) — deep link targets built in a view model while Core owns `Deeplink::to_gem_url`.
- **V61** **S** `ios/GemPriceWidget/Services/WidgetPriceService.swift:82`, `ios/Packages/Components/Sources/AssetImageView.swift`, `.../Grid/GridPosterView.swift`, `.../Lists/ListAssetItemView.swift`, `ios/Packages/GemstonePrimitives/.../GemImage+GemstonePrimitives.swift` — the asset image URL is assembled from `assets.gemwallet.com/blockchains/<chain>/assets/<tokenId>` in five places on iOS.
- **V62** **S** `ios/Features/NFT/.../CollectibleViewModel.swift`, `ios/Features/Transfer/.../TransferDataViewModel.swift`, `ios/Features/WalletConnector/.../ConnectionView.swift`, `ios/Features/Settings/.../PreferencesScene.swift`, `ios/Features/QRScanner/.../QRScannerScene.swift` — one hand-built URL each.
- **V63** **S** `android/ui/.../UriHandlerExt.kt` and `android/ui/.../Markdown.kt` — URL handling helpers with no Core counterpart.
- **V64** **S** `android/features/settings/networks/.../NodeItem.kt` and `android/features/update_app/.../InAppUpdateBanner.kt` — URLs built in composables.

## 23. Ownership: a view model holding more than its service

The rule is in [ARCHITECTURE.md](ARCHITECTURE.md) §7 and now covers stores as well as services. The store sweep is clean on both apps; these are the remaining multi-service holders.

- **O31** **S** `ios/Features/Contacts/.../ManageContactViewModel.swift` — `GemManageContactServiceProtocol` plus `GemNameServiceProtocol`, held only to pass to `ManageContactAddressViewModel`'s `AddressInputViewModel`. Decide whether a shared component's service is a port or a second service.
- **O32** **S** `ios/Features/Onboarding/.../ImportWalletViewModel.swift` — `GemWalletServiceProtocol` plus `GemNameServiceProtocol`, the same conduit shape as O31.
- **O33** **M** `ios/Gem/ViewModels/RootSceneViewModel.swift` — four Core services plus `ViewModelFactory`; the app root, and the one rule inside it (a required update offers only the update action) is unreachable from a test.
- **O34** **S** `android/features/bridge/.../WCRequestViewModel.kt` — `GemWalletConnectServiceInterface` plus `GemSignMessageServiceInterface`.
- **O35** **S** `ios/Gem/ViewModels/RootSceneViewModel.swift:41` — `currentWallet` reads `viewModelFactory.stores.walletStore.getWallet(id:)` with `try?` on every `body` pass. The session service has the async answer; making it sync would flash onboarding, so this needs a decision, not a rewrite.

## 24. Chain-specific branches in app code

N10 closed on 2026-09-16, and the sweep was right that the rule was written twice with a divergence — iOS refused a multicoin wallet with no Ethereum account, Android fell back to the first account, so a malformed account list would have produced a different id on each platform. Neither is on the live path: Core's `GemWalletService` creates wallets now, iOS's `WalletId.from(type:accounts:)` had only test callers and is deleted, and Android's `WalletIdGenerator` is reached only by `Migration_63_64`, whose behaviour must stay frozen because it has already run on installed databases. A migration is the one place a rule does not get consolidated.

- **N11** **S** `ios/Packages/PrimitivesComponents/.../BannerViewModel.swift:41` — a `case .bitcoin` branch decides banner behaviour.
- **N12** **S** `ios/Packages/PrimitivesComponents/Sources/Types/ChainImage.swift` — 14 chain cases; confirm against Android's chain icon map, which the mapper contract allows, and close if it matches.
- **N13** **S** `ios/Packages/PrimitivesComponents/.../SwapProviderType+Gemstone.swift:25` — a lone `case .hyperliquid` beside the provider icon map.

## 25. The About screen, decided twice

- **L15** **S** `ios/Features/Settings/.../AboutUsScene.swift:36` and `android/features/settings/aboutus/.../AboutUsScreen.kt:46` — the label-map fingerprint pairs these at 0.83 on `community`, `privacypolicy`, `termsofservice`, `version`, `website`. The rows of the About screen, their order and their links are chosen in each app.

## 26. Records that hand the app a bare number

Closed on 2026-09-16 after reading every one. The lens asked the wrong question: it matched on the *field* being a number, when the violation is the app *deciding* something from it. Three classes came back, none of them a decision written twice.

**Inputs, not outputs.** `GemAssetDetailsInput.price`, `GemFiatQuoteRequest.amount`, `GemPerpetualTransferData.price`/`leverage`, `GemAutocloseField.price`/`original_price`, `GemPriceAlertSession.current_price` and `GemBannerContext.asset_rank_score` are all built *by* the app and handed *to* Core. A number going in is the app telling Core what the user did.

**Values the app must render with its own locale.** `GemFormattedNumber.value` is carried beside the finished text on purpose — both apps format it through their own number formatter, which is the divergence X158 names. `GemDurationPart.value` is the same shape and symmetric on both apps. `GemRewardsState.invite_reward_points` is interpolated into a sentence whose localization lives in the app, not in Core's `localizer`.

**Values nothing renders.** `GemBalanceValue.amount` is written to the database by a store adapter, `GemPriceUpdate` is a store write, and `ChainConfig`'s activation fees and `GemChart.base_value` are named by no app source at all.

The one real hit was `GemPerpetualChartLayout`, and not for its numbers: `price_low`/`price_high` are axis bounds Core is right to carry, but both apps derived the candle's direction from `close - open` themselves. That is D19/D25 and it landed — see § 17.


## 27. Screens with no Core service

156 iOS view models name no `Gem*ServiceProtocol`; Android has 8. The asymmetry is the shape of the gap: Android injects a service into almost every model, iOS keeps a family of small models that decide presentation locally. A child model handed a Core record is allowed — these are the ones heavy enough to be deciding something. The weight in brackets is members plus methods.

- **B11** **M** `PrimitivesComponents/NetworkFeeSceneViewModel.swift` [29] — the heaviest model in the repo with no service, beside a Core fee-rate record Android reads through `FeeDetailsModel`.
- **B12** **M** `Features/Perpetuals/AutocloseSceneViewModel.swift` [28] — Android's `AutocloseViewModel` drives the same screen from `GemAutocloseSession`.
- **B13** **M** `PrimitivesComponents/AssetDataViewModel.swift` [26] — the shared asset model behind most rows; see also R55.
- **B14** **M** `Features/WalletConnector/ConnectionProposalViewModel.swift` [24] — Android's proposal scene holds `GemWalletConnectServiceInterface`.
- **B15** **M** `PrimitivesComponents/PerpetualDetailsViewModel.swift` [22] — Android holds `GemPerpetualDetailsServiceInterface` for this screen.
- **B16** **M** `Features/AppLock/LockSceneViewModel.swift` [22] — the lock screen, which is a security surface with no Core service on either app.
- **B17** **S** `Features/Support/SupportMessageBubbleViewModel.swift` [19] — bubble grouping and status beside `GemSupportService`.
- **B18** **M** `Features/Swap/SwapDetailsViewModel.swift` [18] — Android builds the same rows in `SwapDetailsUIModelFactory` from `GemSwapQuoteSummary`.
- **B19** **S** `Features/Perpetuals/PerpetualPositionViewModel.swift` [18].
- **B20** **S** `Features/Transactions/TransactionsFilterViewModel.swift` [16] — the activity filter; Android's `TransactionsViewModel` holds the Core filter list.
- **B21** **S** `Features/Perpetuals/CandlestickChartViewModel.swift` [16] — see D25.
- **B22** **S** `PrimitivesComponents/TransactionViewModel.swift` [15] — see R77.
- **B23** **S** `PrimitivesComponents/NetworkFeeCustomViewModel.swift` [15] — Android's namesake takes `FeeDetailsModel` and calls `customFee`; iOS does not.
- **B24** **S** `PrimitivesComponents/BannerViewModel.swift` [14] — `canClose` is declared on Android too, in `BannerItemUIModel.kt`.
- **B25** **S** `Features/Onboarding/VerifyPhraseViewModel.swift` [14] — phrase verification is a security rule with no Core owner.
- **B26** **S** `PrimitivesComponents/AddressListItemViewModel.swift` [13] — see R59.
- **B27** **S** `PrimitivesComponents/InputValidationViewModel.swift` [11] — validation on the iOS side of the `Validators` boundary.
- **B28** **S** `PrimitivesComponents/BalanceViewModel.swift` [11] — see R58.
- **B29** **S** `Features/Settings/ChainNodeViewModel.swift` [11] — node rows beside `GemChainSettingsService`.
- **B30** **S** `Features/QRScanner/QRScannerSceneViewModel.swift` [11] — scan handling with no Core payment service.
- **B31** **S** `Features/FiatConnect/FiatQuoteViewModel.swift` [11] — quote rows beside `GemFiatQuoteService`.
- **B32** **S** `Features/Assets/AssetsFilterViewModel.swift` [11] — `chainsFilter` is declared on Android too, in `TransactionsViewModel.kt`.
- **B33** **S** `PrimitivesComponents/AssetViewModel.swift` [10] — see R62.
- **B34** **S** `PrimitivesComponents/PriceViewModel.swift` [9] — see D23.
- **B35** **S** `PrimitivesComponents/EmptyContentTypeViewModel.swift` [9] — `actions` is declared on Android too, in `DelegationViewModel.kt`.
- **B36** **S** `PrimitivesComponents/TextInputSheet/TextInputViewModel.swift` [9].
- **B37** **S** `Features/Transfer/TransferDataViewModel.swift` [9].
- **B38** **S** `PrimitivesComponents/FiatTransactionViewModel.swift` [8] — the fiat transaction row, which SERVICES.md says already reads the same on both apps.
- **B39** **S** `PrimitivesComponents/CopyTypeViewModel.swift` [8] — see S30.
- **B40** **S** `PrimitivesComponents/Types/ChartHeaderViewModel.swift` [8] — see R65.
- **B41** **S** `PrimitivesComponents/Protocols/ValueHeaderViewModel.swift` [8].
- **B42** **S** `Features/Transfer/TransactionInputViewModel.swift` [8].
- **B43** **S** `Features/Swap/SwapTokenViewModel.swift` [8].
- **B44** **S** `Features/Swap/PriceImpactViewModel.swift` [8] — see R66.
- **B45** **S** `Features/Settings/RewardRedemptionOptionViewModel.swift` [8] — see R63.
- **B46** **S** `Features/PriceAlerts/PriceAlertItemViewModel.swift` [8].
- **B47** **S** `Features/Perpetuals/OpenPositionItemViewModel.swift` [8].
- **B48** **S** `PrimitivesComponents/WalletHeaderViewModel.swift` [7].
- **B49** **S** `PrimitivesComponents/ChainViewModel.swift` [7].
- **B50** **S** `GemPriceWidget/CoinPriceRowViewModel.swift` [7] — the widget row, which cannot import Gemstone today.
- **B51** **S** `Features/Support/SupportMessageInputBarViewModel.swift` [7].
- **B52** **S** `Features/Settings/ServiceStatusItemViewModel.swift` [7].
- **B53** **S** `Features/Perpetuals/PerpetualsHeaderViewModel.swift` [7].
- **B54** **S** `Features/Perpetuals/PerpetualViewModel.swift` [7] — `priceText` is declared on Android too, in `ChartHeaderUIModel.kt`.
- **B55** **S** `Features/Perpetuals/PerpetualPositionItemViewModel.swift` [7].
- **B56** **S** `Features/Perpetuals/AutocloseViewModel.swift` [7] — see R69 and D22.
- **B57** **S** `PrimitivesComponents/SimulationWarningViewModel.swift` [6] — simulation warnings landed in Core (d458faac6e); confirm this model only maps them.
- **B58** **S** `PrimitivesComponents/PnLViewModel.swift` [6] — position profit landed in Core (6321c9eb20); same check.
- **B59** **S** `PrimitivesComponents/ListAssetItemViewModel.swift` [6].
- **B60** **S** `PrimitivesComponents/ChartValuesViewModel.swift` [6].
- **B61** **S** `Features/WalletTab/PerpetualsPreviewViewModel.swift` [6].
- **B62** **S** `Features/Transactions/TransactionTypesFilterViewModel.swift` [6].
- **B63** **S** `Features/Swap/SwapProvidersViewModel.swift` [6] and `SwapButtonViewModel.swift` [6].
- **B64** **S** `Features/Perpetuals/PerpetualItemViewModel.swift` [6].
- **B65** **S** `Features/Assets/AssetHeaderViewModel.swift` [6].

## 28. Core duplicated inside Core

Closed on 2026-09-16 after reading all thirteen. Twelve were the *convention*, not duplication: `calculate_transaction_fee`, `calculate_fee_rates` and `calculate_network_apy` take different arguments and compute different chains' fees; `create_staking_client` is a testkit helper per chain; `checksum_address`, `deposit_addresses`, `chain_from_id` and `for_chain` are per-provider tables; `config_session_properties` is a service forwarding to the collaborator it composes, which is how composition reads; `has_price`/`has_size`/`execution_error` are a primitive and its accessor. Naming the same operation the same way across chain crates is what makes them readable side by side — the lens cannot tell that apart from a copy, so match on body shape and signature, not on name.

The one real copy was `create_eth_client`, identical in `swapper` and `yielder` down to the `EVMChain::from_chain(...).ok_or(...)` line and differing only in error type. `EthereumClient::for_chain` now owns it in `gem_evm`, which both crates already depend on, and each factory maps the `None` to its own error.


## 29. Gaps the screen-service map already names

[SERVICES.md](SERVICES.md) says a screen service only one app holds is the next consolidation. These are the rows where the table itself shows one side empty or asymmetric.

- **P72** **M** `GemAppUpdateService` — iOS `AboutUsViewModel` holds it; Android uses Play in-app update instead, so the update decision is made by two different owners. `AppUpdateCoordinator` already maps `upgradeRequired` itself (see F29).
- **P73** **M** `GemAvatarService` — Android has no avatar surface at all, so wallet avatars are an iOS-only feature rather than a Core one.
- **P74** **S** `GemNotificationsService` — iOS `NotificationsViewModel` holds it; Android's `SettingsViewModel` uses push cases instead.
- **P75** **S** `GemTransactionDetailsService` — iOS holds the service, Android reaches the same answer through `GetTransactionDetailsImpl` as an observed read, so the links are built in two places.
- **P76** **S** `GemChainService` — iOS holds it in the chain picker, Android in two unrelated models (`ContactChainSelectViewModel`, `SelectImportTypeViewModel`); confirm the three screens ask the same question.
- **P77** **S** `GemRecentActivityService` — three iOS holders against two Android; the extra iOS holder is `SelectAssetViewModel`, which Android answers inside `BaseAssetSelectViewModel`.
- **P78** **S** `GemBannerService` — held by no screen on either app, composed inside two services; confirm the banner rules have not drifted between those two compositions.
- **P79** **S** `GemWalletSessionService` — iOS spreads it over `RootSceneViewModel` and `NavigationHandler`; Android keeps it in `SessionCoordinator`. The iOS split is what produced O35.
- **P80** **S** `GemStakeService` — three screens each side, but iOS `EarnSceneViewModel` filters delegations itself (S21) where Android does not.
- **P81** **S** About 67 exported records and enums are named by neither app. SERVICES.md says review before deleting; do the review and record the ones that are reached through a nested field so the list stops being re-swept.

## 30. App ports that Core could own

- **V66** **S** `ios/Packages/.../ConnectionComponentMonitoring.swift` and `ConnectivityMonitor.swift` — two single-method protocols over connection health, beside `GemConnectionService`.
- **V67** **S** `ios/Packages/.../WebSocketRequestProvider.swift` — one method, building the authenticated socket request that Core's device auth already signs.
- **V68** **S** `ios/Packages/Store/Sources/BindableQuery.swift` — a one-method protocol behind every observed read on iOS; Android has narrow cases instead. Worth one decision about which shape both apps use.
- **V69** **M** `ios/Packages/Formatters` and `ios/Packages/Validators` cannot import Gemstone, which is what keeps D15–D17 duplicated. The item is the dependency, not the formatter: decide whether the widget and these two packages get a Gemstone-free Core surface or move under one that can import it.
- **V70** **S** `android/features/update_app/.../InAppUpdateServiceImpl.kt` — an app-owned update service beside `GemAppUpdateService`, with its own HTTP client and timeouts (S28).
- **V71** **S** `android/ui/.../UriHandlerExt.kt` — URL opening policy in the UI module (V63).
- **V72** **S** `ios/Packages/GemstonePrimitives/Sources/Config.swift` — the iOS half of V59; decide whether app URLs are a Core config record or stay per-platform.

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

[ARCHITECTURE.md](ARCHITECTURE.md) says a screen whose state changes is a session. 83 iOS view models declare six or more `var`s and name no `Gem*Session`; Android has two. The count in brackets is mutable members. These are hard because a session is a Core object with a lifetime, not a record — each one is a flow to model, and the screen's state has to move in one move.

- **S34** **L** `Features/Settings/RewardsViewModel.swift` [40] — the widest stateful screen with no session: wallet selection, sheets, alerts, toasts and the rewards state.
- **S35** **L** `Features/WalletTab/WalletSearchSceneViewModel.swift` [32].
- **S36** **L** `Features/Assets/AssetSceneViewModel.swift` [30].
- **S37** **L** `Features/Assets/SelectAssetViewModel.swift` [26].
- **S38** **L** `Features/Transfer/ConfirmTransferSceneViewModel.swift` [25] — confirm has `GemConfirmation`, which SERVICES.md explicitly calls not a session; this is the item that decides whether that is still right.
- **S39** **M** `PrimitivesComponents/AssetDataViewModel.swift` [24].
- **S40** **M** `Features/Transfer/AmountSceneViewModel.swift` [24].
- **S41** **M** `Features/WalletConnector/SignMessageSceneViewModel.swift` [22] — a signing surface holding its own state.
- **S42** **M** `PrimitivesComponents/PerpetualDetailsViewModel.swift` [20].
- **S43** **M** `PrimitivesComponents/NetworkFeeSceneViewModel.swift` [20].
- **S44** **M** `Features/WalletTab/WalletSceneViewModel.swift` [20].
- **S45** **M** `Features/Stake/DelegationViewModel.swift` [20].
- **S46** **M** `Features/Stake/StakeSceneViewModel.swift` [19].
- **S47** **M** `Features/Onboarding/ImportWalletSceneViewModel.swift` [19] — wallet import state, a recovery-critical flow.
- **S48** **M** `Features/Contacts/ManageContactViewModel.swift` [19].
- **S49** **M** `Features/WalletConnector/ConnectionProposalViewModel.swift` [18].
- **S50** **M** `Features/Settings/PreferencesViewModel.swift` [17].
- **S51** **M** `Features/Perpetuals/PerpetualSceneViewModel.swift` [17].
- **S52** **M** `Features/WalletTab/NetworkAssetsSceneViewModel.swift` [16].
- **S53** **M** `Features/Perpetuals/PerpetualsSceneViewModel.swift` [16] and `PerpetualPositionViewModel.swift` [16].
- **S54** **M** `Features/Transfer/ReceiveViewModel.swift` [15].
- **S55** **M** `Features/Swap/SwapDetailsViewModel.swift` [15].
- **S56** **M** `Features/Support/SupportMessageBubbleViewModel.swift` [15].
- **S57** **M** `Features/NFT/CollectibleViewModel.swift` [15].
- **S58** **M** `Features/Stake/EarnSceneViewModel.swift` [14].
- **S59** **M** `Features/Settings/SecurityViewModel.swift` [14] — a security surface whose lock-period state produced a crash on 2026-09-15.
- **S60** **M** `Features/AppLock/LockSceneViewModel.swift` [14] — the lock screen state machine.
- **S61** **S** `PrimitivesComponents/BannerViewModel.swift` [13].
- **S62** **S** `Features/WalletTab/AssetsResultsSceneViewModel.swift` [13].
- **S63** **S** `Features/Transfer/RecipientSceneViewModel.swift` [13].
- **S64** **S** `Features/Settings/AboutUsViewModel.swift` [13] — see L15 and P72.
- **S65** **S** `Features/WalletConnector/ConnectionsViewModel.swift` [11].
- **S66** **S** `Features/Stake/DelegationSceneViewModel.swift` [11].
- **S67** **S** `Features/ManageWallets/WalletIDetailViewModel.swift` [11] — also the one iOS file whose name carries a typo (`WalletIDetail`).
- **S68** **S** `Features/Settings/ChainNodeViewModel.swift` [10] and `Features/FiatConnect/FiatQuoteViewModel.swift` [10].
- **S69** **S** `Features/Onboarding/VerifyPhraseViewModel.swift` [9] and `Features/Contacts/ManageContactAddressViewModel.swift` [9].
- **S70** **M** `android/features/perpetual/.../PerpetualDetailsViewModel.kt` and `android/features/confirm/.../ConfirmViewModel.kt` — the only two Android screens in this shape; both have a Core service but keep six `MutableStateFlow`s of their own.

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
- **X141** **M** `core/crates/gem_rewards/src/risk_scoring/scoring.rs` (1113) — a scoring model with no app reader; confirm it is server-side only.
- **X142** **M** `core/crates/storage/src/schema.rs` (1111) — generated by Diesel; confirm and record so it stops being swept.
- **X143** **M** `core/crates/swapper/src/stonfi/provider.rs` (1011), `across/provider.rs` (915), `chainflip/provider.rs` (906) — three swap providers over 900 lines each; compare their shapes before splitting any one.
- **X144** **M** `core/crates/gem_tron/src/signer/chain_signer.rs` (976) — the widest chain signer.
- **X145** **M** `core/crates/gem_hypercore/src/provider/perpetual_mapper.rs` (925).
- **X146** **S** `core/gemstone/src/models/remote_types.rs` (1940) — generated by `just generate-models`; already closed as X90 and kept here only so the count is explained.

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

[PERFORMANCE.md](PERFORMANCE.md) sets p95 ≤ 100 ms to first feedback, ≤ 200 ms to useful content from local data, and ≤ 100 ms from a received update to the frame. The budgets exist; these screens have no measurement against them.

- **PERF20** **M** The wallet screen against the warm-data budget, on both apps, with 1000 assets seeded.
- **PERF21** **M** The asset screen (S36, 30 mutable members, 10-collaborator service).
- **PERF22** **M** The confirm screen, whose rules file is the widest in Core (X133).
- **PERF23** **M** The swap screen's quote refresh against the update-to-frame budget.
- **PERF24** **M** The activity list with its filters applied (B20, C10).
- **PERF25** **M** The perpetuals screen, which holds a live socket and a chart (X107).
- **PERF26** **S** Launch to the wallet screen on both apps, which is what `GemAppStartService` (X117) orchestrates.
- **PERF27** **S** The select-asset sheet, which composes six services (X114) and filters in the view model (S25).
- **PERF28** **S** The iOS `RootSceneViewModel.currentWallet` database read on every `body` pass (O35) — measure it before deciding whether it matters.

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
