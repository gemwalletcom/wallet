# Feature architecture

Use the relevant sections when changing shared feature behavior or its app integration. These are current ownership contracts, not a mandate to migrate adjacent code. Resolve disagreements with source using [guidance precedence](../AGENTS.md#using-the-guidance).

## Find the Relevant Contract and Example

Read the contract and the named implementation, then the actual owner and callers being changed. Examples illustrate the named responsibility, not every convention in their file; do not copy unrelated scaffolding or existing violations.

| Task | Contract | Source to inspect |
|---|---|---|
| Pure feature rule | [§ 1](#1-rules-are-pure-and-have-a-test-that-flips), [§ 6](#6-where-derived-domain-answers-live) | [`price_alert/rules.rs`](../core/gemstone/src/services/price_alert/rules.rs), including its tests |
| Service orchestration and store port | [§ 2](#2-the-service-orchestrates-it-owns-its-store-and-depends-on-services), [§ 4](#4-the-store-trait-is-the-apps-only-persistence-obligation) | [`price_alert/mod.rs`](../core/gemstone/src/services/price_alert/mod.rs), [`store.rs`](../core/gemstone/src/services/price_alert/store.rs) |
| Screen state, list rows, or what a view may name | [§ 3](#3-return-one-record-that-answers-the-whole-question), [§ 5](#5-the-app-maps-it-does-not-decide) | [`fiat/session.rs`](../core/gemstone/src/services/fiat/session.rs), [`assets/model.rs`](../core/gemstone/src/services/assets/model.rs) |
| App mapping, dependency ownership, or construction | [§ 5](#5-the-app-maps-it-does-not-decide), [§ 7](#7-at-most-one-core-service-on-ios-narrow-cases-on-android), [§ 8](#8-services-are-injected-never-constructed-at-a-call-site) | The changed screen's view model and its factory/Hilt provider; follow the examples in those sections |
| List rows and sections | [§ 5](#a-list-row-renders-from-one-shared-row-model) | [`ListItemModel.swift`](../ios/Packages/Components/Sources/Types/ListItemModel.swift), [`ListItemModel.kt`](../android/ui/src/main/kotlin/com/gemwallet/android/ui/components/list_item/ListItemModel.kt), [`ListSections.kt`](../android/ui/src/main/kotlin/com/gemwallet/android/ui/components/list_item/ListSections.kt) |
| Loading UI | Shared [reuse rule](../skills/engineering-principles.md#clean-code-principles) | Current screen state first; [`LoadingView.swift`](../ios/Packages/Components/Sources/LoadingView.swift), [`LoadingScene.kt`](../android/ui/src/main/kotlin/com/gemwallet/android/ui/components/screen/LoadingScene.kt) |
| REST or JSON-RPC client | [§ 12](#12-a-clients-requests-are-one-enum-the-client-only-sends) | [`AptosClient`](../core/crates/gem_aptos/src/rpc/client.rs) for direct sends, [`TronGridClient`](../core/crates/gem_tron/src/rpc/trongrid/client.rs) for shared credentials, [`SolanaRpc`](../core/crates/gem_solana/src/jsonrpc.rs) for RPC |
| Tests and fixtures | [§ 10](#10-tests) and the platform testing guide | The owner's existing tests, [`primitives/src/testkit/asset_mock.rs`](../core/crates/primitives/src/testkit/asset_mock.rs) for fixtures, [`gem_client/testkit.rs`](../core/crates/gem_client/src/testkit.rs) for wire behavior |

Read [§ 13](#13-shapes-that-were-tried-and-reverted) only when the task needs the rationale for a rejected design. Subsystem-specific contracts remain in their own documents.

## The one rule

**Core decides. The apps ask, render, and store.**

A rule is anything that could produce a different answer on one platform than the other: a filter, a threshold, a gate, an ordering, a mapping from state to what the user sees. If iOS and Android could ever disagree about it, it belongs in Core.

Everything else is platform work: rendering, navigation, observation, secure storage, keychain and biometrics, and the SQL that stores rows.

## Leave it better than you found it

**A change that adds to a shape you know is wrong is not finished.** When the code you are about to extend already violates a rule here, fix the shape first and then add to the fixed one. Adding a fifth argument to a call that should take one, a second copy of a decision, or another service to a view model that should hold one, buys today's feature by making tomorrow's harder — and the cost is never paid back on its own, because the next change inherits the same excuse.

Three questions before you add anything:

1. **Would I write it this way from scratch?** If not, what stops you is the fix.
2. **Am I threading something through a place that does not use it?** A parameter a function only passes on means the wiring is in the wrong place, not that the function needs another parameter.
3. **Is this the second time?** The first copy of a decision is a choice; the second is a defect. Move it to Core before adding the second.

When the fix is genuinely larger than the change in hand, say so plainly, land the smaller piece, and add the larger one to [Open work](TODO.md) with the shape it should take — never leave it unsaid. What is not allowed is to add to a shape you know is wrong and stay quiet about it.

## Layout

```
core/gemstone/src/services/<feature>/
    mod.rs      the service: owns the feature flow and exported UniFFI methods
    rules.rs    pure feature decisions + their unit tests
    model.rs    feature records/enums and intrinsic behavior; only FFI types derive UniFFI
    store.rs    the trait each app implements over its own database
    error.rs    structured feature errors when GemServiceError is insufficient
```

Only the files the feature needs. A feature with no persistence has no `store.rs`.

## 1. Rules are pure and have a test that flips

A rule is a function or receiver method that takes values and returns an answer. It performs no I/O, holds no service dependency and does not read the clock. Pass time in as a value when it is part of the decision. Pure does not mean publicly exported: use the narrowest Rust visibility and shape the FFI-facing API according to § 6.

```rust
// services/confirm/rules.rs
use num_bigint::BigUint;

pub(super) fn selectable_fee_assets(assets: Vec<Asset>, balances: Vec<GemAssetBalance>, prices: Vec<GemAssetPrice>) -> Vec<GemFeeAsset> {
    balances
        .into_iter()
        .filter(|balance| balance.available > BigUint::from(0u32))
        .filter_map(|balance| {
            let asset = assets.iter().find(|asset| asset.id == balance.asset_id)?.clone();
            let price = prices.iter().find(|price| price.asset_id == balance.asset_id).cloned();
            Some(GemFeeAsset { asset, balance, price })
        })
        .collect()
}
```

Its test follows Core's `test_<function_name>` convention, covers the meaningful cases together, and fails if the rule flips:

```rust
#[test]
fn test_selectable_fee_assets() {
    let funded = Asset::from_chain(Chain::Tempo);
    let empty = Asset::from_chain(Chain::Ethereum);

    let selectable = selectable_fee_assets(
        vec![funded.clone(), empty.clone()],
        vec![balance(&funded.id, 1), balance(&empty.id, 0)],
        vec![],
    );

    assert_eq!(selectable.iter().map(|fee| fee.asset.id.clone()).collect::<Vec<_>>(), vec![funded.id]);
}
```

Verify changed domain rules using the shared [test-intent rule](../skills/engineering-principles.md#tests).

## 2. The service orchestrates; it owns its store and depends on services

A service composes rules with I/O. It may hold its own feature store and narrow platform ports. For another domain, it depends on that domain's service, never its store — a store belongs to one owner, and reaching around that owner creates a second read path it cannot see.

Inspect the dependency graph before replacing a foreign-domain store with its service. Never introduce an `Arc` cycle to satisfy this rule; split out a narrow query service, invert the dependency, or redesign the ownership boundary first. `GemWalletService` already depends on `GemWalletSessionService`, so making the session service depend back on the wallet service would be worse than the store debt it replaces.

```rust
// services/confirm/mod.rs
#[derive(uniffi::Object)]
pub struct GemConfirmService {
    gateway: Arc<GemGateway>,
    balance: Arc<GemBalanceService>,
    price: Arc<GemPriceService>,
    assets: Arc<GemAssetsService>,
}

#[uniffi::export]
impl GemConfirmService {
    pub fn fee_assets(&self, wallet_id: WalletId, chain: Chain) -> Result<Vec<GemFeeAsset>, GemConfirmError> {
        let fee_asset_ids = chain_fee_asset_ids(chain);
        if fee_asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        let assets = self.assets.assets(fee_asset_ids.clone())?;
        let balances = self.balance.balances(wallet_id, fee_asset_ids.clone())?;
        let prices = self.price.prices(fee_asset_ids)?;
        Ok(rules::selectable_fee_assets(assets, balances, prices))
    }
}
```

`GemConfirmError` implements `From<GemServiceError>` once in `error.rs`, so the three reads use `?` without repeating the same `Load` conversion.

The method is thin: gather inputs, call the rule, return. Product or domain-decision branching belongs in `rules.rs`; I/O sequencing, error propagation and empty-work short circuits may remain in the service.

**Point reads should be synchronous where the platform allows it.** `GemWalletSessionStore.get_current_wallet_id` is a sync trait method, so `GemWalletSessionService` answers the current-wallet id without `await`. An `async` point read pushes the caller back to the store, which is how the confirm screen ended up reading `AssetStore` directly for two years.

It is not free in both directions. `GemWalletStore.get_wallet` is still `async` because the two platforms disagree about what a blocking row read costs: iOS reads GRDB synchronously, while Room forbids a blocking query on the main thread, so the same signature would make Android either open a non-suspend DAO with a threading rule nothing enforces or block inside `runBlocking`. Make a point read sync when the row is already in memory or the store is the app's own preferences; when it is a database row that one platform can only read off its main thread, keep it `async` and let the caller hold the value rather than re-reading it per render.

### No trivial exports

An export earns its place by making a decision. A function that looks up a constant for a variant, or wraps a value the app already holds so the app can ask for it back, is not a decision — it is a second spelling of a `match` the app will write anyway, plus an FFI crossing per call. Two of these were caught and reverted: an icon name per row key, and a title per enum case. Both belong on the enum as data the screen record already carries, resolved once in the app's [module mapper](#one-mapper-per-module-names-every-core-key-it-renders).

The test is what the caller could not have worked out: if the answer depends only on the variant, the variant is the answer and the app maps it. If it depends on state, configuration, a chain rule or several values at once, it is a decision and Core owns it.

The one exception is a **projection**: a pure function of a value Core already defines, where the value is a remote record and Rust allows no inherent `impl` to hang it off. `walletRow(wallet)` and `emptyState(input)` are that shape.

### A staged load names what each stage waits for

A load that fans out and then narrows is a graph, not a list, and the graph has to be written down before anyone reorders it. [`GemConfirmService.load`](../core/gemstone/src/services/confirm/mod.rs) is the worked example:

```
        ┌─ preload ────────┐ metadata
start ──┼─ fee rates ──────┼──▶ validate scan ──▶ select fee rate ──▶ transaction load ──▶ confirm data
        ├─ scan ───────────┤                                          (metadata + gas price)
        └─ simulate ───────┘
```

The four openers run together and every one of them is awaited before the first gate. What comes after is ordered for two different reasons, and they are not interchangeable:

- **Data.** The transaction load needs the preload's metadata and the gas price of the selected fee rate. It needs nothing from the scan or the simulation.
- **Policy.** The scan verdict and the simulation gate the load anyway, because a rejected input must cost no provider work. A transport failure in the preload or the fee rates is reported before the scan verdict is read, so a malicious verdict is only ever surfaced for an input that would otherwise have loaded.

**The scanner fails open.** A scanner outage yields no verdict and the send continues; only an explicit `is_malicious` or an unmet `is_memo_required` stops it. That is the policy, not an oversight; changing it is a product decision rather than a performance one.

Before moving any stage earlier, audit what the stage actually does on every provider — "load" is not a promise of read-only. Every chain family but one answers `get_transaction_load` with RPC estimates and local arithmetic. HyperCore is the exception: its swap and perpetual path creates and persists an agent keypair in the secure store and writes approval-cache preferences, so starting it before the scan verdict would provision durable credentials for transactions the scanner then rejects. That single provider is why the gate stays where it is, and an implementation commit has to move the provisioning out of the load first — the overlap is safe for the read-only families only once each one is pinned as read-only.

The order itself is tested, not assumed: a malicious verdict must leave the chain unasked, and a clean verdict must let the load through.

## 3. Return one record that answers the whole question

A screen that needs five things should make one call, not five. Core assembles the answer.

```rust
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmPreload {
    pub confirm_data: GemConfirmData,
    pub metadata: GemConfirmMetadata,
    pub fee_asset: Asset,
    pub amount: GemTransferAmountResult,
}
```

Two things this record gets right:

**A recoverable failure is a value, not an error.** An unaffordable transfer still has a fee, fee rates and a simulation to render, so the amount is an enum rather than collapsing the whole call:

```rust
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransferAmountResult {
    Amount { amount: GemTransferAmount },
    Error { error: GemConfirmError },
}
```

The error is the same `GemConfirmError` every other confirm failure uses, carrying the `Asset` it names and the required/available values, so the app renders it the same way whether it came from the preload or the send.

**State that travels together is one type.** An approval is either an exact amount or unlimited — never a string plus a boolean the caller has to reassemble:

```rust
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemApprovalValue {
    Exact { value: GemBigUint },
    Unlimited,
}
```

### A list row is a record of choices

A row is the smallest case of this rule and the one the codebase repeats most. Core returns what the row *means* — which name it shows, whether the symbol would repeat that name, what sits underneath, what trails it — and the app turns each case into a widget. A list whose entries are fixed and unconditional is not one of these: the tab bar names three or four destinations with no rule behind them, so it stays app-side until a destination becomes conditional. The record carries the value the row shows, already formatted, and the name of every outcome the row draws. It carries the choices that would otherwise be re-made, differently, in each list on each platform.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetRow {
    pub title: GemAssetRowTitle,
    pub shows_symbol: bool,
    pub subtitle: GemAssetRowSubtitle,
    pub trailing: GemAssetRowTrailing,
}
```

The screen asks its service for the row once and passes it down, so wallet, search, select and network-asset lists all render from one answer and a change to what a row means is one edit in Core. The app's row model holds the record and exposes platform values from it:

```swift
public var name: String {
    switch row.title {
    case .asset: assetDataModel.name
    case .canonicalAsset: assetDataModel.asset.id.type == .native ? assetDataModel.asset.chain.asset.name : assetDataModel.name
    case .network: assetDataModel.asset.chain.networkName
    }
}
```

```kotlin
private fun AssetInfo.title(naming: GemAssetRowTitle): String = when (naming) {
    GemAssetRowTitle.ASSET -> asset.name
    GemAssetRowTitle.CANONICAL_ASSET -> when (asset.subtype) {
        AssetSubtype.NATIVE -> asset.chain.asset().name
        AssetSubtype.TOKEN -> asset.name
    }
    GemAssetRowTitle.NETWORK -> asset.id.chain.asset().name
}
```

`GemValidatorRow`, `GemFiatQuoteRow`, `GemWalletRow` and `GemBalanceRow` are the same shape for their lists. A session is for a screen the user drives, with events and a derived view state; a projection of one value that answers the same way every time is a row. The thing to look for in a row model is a decision the record could carry: if both apps compute it, it belongs in the record, not in two view models.

### The record carries the finished value, not the ingredients

An app that receives a number and a flag has to decide what to print, and the two apps decide differently: one groups the block height with the device locale and the other with a US formatter, one shows a dash for a missing chain id and the other an empty cell, one draws a check emoji for a synced node and the other a tinted icon. None of that is a platform capability; it is the same answer computed twice.

So the row carries the text and names the outcome:

```rust
pub enum GemNodeCheckRow {
    ChainId { value: String },
    InSync { state: GemNodeSyncState },
    LatestBlock { value: String },
    Latency { milliseconds: u32 },
}
```

`ChainId` and `LatestBlock` arrive printable — Core groups the digits and substitutes the placeholder, so neither app carries a formatter for them. `InSync` carries a named state rather than a `bool`, because a boolean forces the app to pick the glyph and the two apps pick differently; the name is mapped once in each app's style file, the way every other Core case is. `Latency` stays a number because its text is a localized template with a number in it, and that template lives in the app's catalog.

The app is then a map over the rows, and the only thing it decides is which widget draws each one:

```swift
var fields: [ListItemField] {
    result.rows().map { ListItemField(title: $0.title, value: $0.text) }
}
```

```kotlin
check.rows().forEach { row ->
    when (row) {
        is GemNodeCheckRow.InSync -> PropertyItem(
            title = { PropertyTitleText(row.stringRes()) },
            data = { PropertyDataText("", badge = { Icon(row.state.icon(), tint = row.state.tint()) }) },
        )
        else -> PropertyItem(row.stringRes(), row.text())
    }
}
```

iOS draws the sync state as an emoji in the value column and Android as a tinted icon beside it, so Android branches on the row to pick the widget while iOS does not. That branch is layout; both read the same `GemNodeSyncState` from their style file and neither decides what the state means.

Two things still map per platform, and only two: the localized label for each case, and the glyph or colour for each named outcome. Anything else in a row model — a formatter, a placeholder, a ternary over a flag — is a decision that belongs in the record.

The same holds for a value an SDK sends on the wire. Rejecting a WalletConnect proposal used to be decided twice: iOS mapped the error to a CAIP-25 reason and deleted the stored session, Android sent the string `"Reject Session"` for every rejection and kept the session. `session_rejection` now returns the finished rejection — the named reason, the CAIP-25 code, the message the dApp receives and whether the session is deleted — and each app maps only its own SDK's error type onto the named reason, which is the one thing Core cannot see.

```swift
let rejection = service.sessionRejection(reason: GemWalletConnectRejectionReason(from: error))
try await WalletKit.instance.rejectSession(proposalId: proposal.id, reason: RejectionReason(rejection.reason))
if rejection.deletesSession {
    try await service.deleteSession(sessionId: proposal.pairingTopic)
}
```

```kotlin
val rejection = walletConnectService.sessionRejection(reason)
walletConnectClient.rejectSession(proposal, rejection) {
    if (rejection.deletesSession) {
        scope.launch { walletConnectService.deleteSession(proposal.pairingTopic) }
    }
}
```

### A row that a screen only ever draws one way keeps its shape app-side

`GemAssetRow` carries the layout because the same asset row is drawn four ways: the wallet list prices it, select-asset names its network, manage-tokens toggles it, receive copies it. The choice varies, so Core makes it once and both apps switch on `subtitle` and `trailing`.

The rows that do not vary keep their shape in the app. A perpetual market row always shows its price under the name and its volume at the end; a position row always shows direction and leverage under the name and its margin at the end; a price-alert row always shows the price and a toggle. Each is one row on one screen, so a `GemListItemSubtitle`/`GemListItemAccessory` that every row carried would add a crossing per row and settle nothing — the cost [§ 3 Keep the crossings few](#keep-the-crossings-few) warns about, paid for a decision no one is making twice. Reviewed on 2026-09-15 and left as is.

The test is whether the same row is drawn differently somewhere: if it is, the shape is a choice and belongs in the record; if it is not, it is layout and belongs in the view.

### A row is projected from its value, never fetched from a service

`walletRow(wallet)`, `walletRows(wallets)`, `secretPhraseRows(wordCount)` and `emptyState(input)` are pure functions of the value, so they are exported as functions, not hung off a service. Reading a row must never require a service the screen does not otherwise have — that is what forces a second service into a view model, a row to be passed down as a constructor argument, or a factory to call `service.walletRow(...)` at the composition root. All three were tried on the wallet row and all three read as the same mistake: a projection dressed up as a dependency. This is the one exception to [no free exports](#no-trivial-exports): a projection has no owner to be a receiver on, because the value it projects is a remote record and Rust allows no inherent `impl` for it.

A view model that already owns the screen's service still asks that service for anything the *screen* decides. The line is whether the answer depends on state the service holds.

### The row carries the whole answer; the view model only reads it

Once Core owns the row, the app-side model has nothing left to decide, and a model whose every property is a one-line read of the record should not exist. `WalletViewModel` was deleted for exactly this: `GemWalletRow` carries `id`, `name`, `subtitle`, `placeholder`, `showsWatchBadge`, `hasAvatar` and `imageUrl`, and both apps now hand the record straight to the view. Android passes `row: GemWalletRow` to `WalletItem`; iOS passes it to `WalletListItemView`.

Two things a record cannot carry are a localized string and a bundled image asset, and both have one home per platform.

**Every Core case that becomes localized text maps in one file per module.** iOS has `Gemstone+Localized.swift`, Android a `localization/GemstoneText.kt` in each feature module that needs one; both hold `GemTransactionTitle`, `GemWalletSubtitle` and every case that follows. Core returns the case, the file returns the string. Do not add a per-type `Gem*+Localized.swift`, and never push the app's string catalog into Core through `GemLocalizer` — that trades a maintained translation for an FFI crossing per row. `GemLocalizer` is for text Core composes itself, such as a default wallet name.

**A Core case that becomes a bundled image maps in that type's own extension.** `GemWalletRow+PrimitivesComponents.swift` turns `placeholder` into an `Image` and the row into an `AssetImage`; Android's `GemWalletPlaceholder.iconModel()` and `GemWalletRow.supportIcon()` are the same two mappings. One place per platform, read by every screen.

A row model earns its place only by owning something the record cannot: formatting that depends on locale or user preference, a binding, a bundled asset, or a join to app-side data. Keep it for those and read the record for everything else. A model whose every property is a one-line read of the record owns nothing, and the view takes the record instead.

### A details screen gets a details record, not a row plus the object it came from

A row is shaped for a list, and a details screen of the same value always needs more: the identity it navigates with, the state it gates on, the one address it shows. Holding the row *and* the domain object it was projected from is how that extra arrives in an app, and it makes both sides answer the same question — the app reads `wallet.accounts` while Core already knows whether there is a single account to show. The details record carries the row and the rest of the screen's answers:

```rust
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletDetails {
    pub row: GemWalletRow,
    pub secret_kind: Option<GemWalletSecretKind>,
    pub address: Option<ChainAddress>,
}
```

Android's `WalletDetailsAggregateImpl(details)` and iOS's `WalletDetailViewModel.details` then read it, and neither keeps the `Wallet` for anything the record answers:

```kotlin
class WalletDetailsAggregateImpl(details: GemWalletDetails) : WalletDetailsAggregate {
    override val row: GemWalletRow = details.row
    override val address: ChainAddress? = details.address?.toPrimitives()
}
```

```swift
var address: WalletDetailAddress? {
    guard let account = details.address?.toPrimitives() else { return .none }
    return .account(SimpleAccount(chain: account.chain, address: account.address), link: ...)
}
```

`GemTransactionRow` and `GemTransactionDetailRows` are the same split for one value: the list row and the details record each carry the transaction's id, asset, type, direction, state and creation date, so neither screen needs the `TransactionExtended` beside it.

### A screen derives its record once and passes it down

A Core record is derived by crossing the FFI, so *where* a screen derives it decides how many times it crosses. The rule is one derivation per render, passed down:

```swift
public var body: some View {
    let details = model.details
    return List {
        ValueHeaderView(model: model.assetHeaderModel(details))
        if details.state.showsEarn { ... }
    }
    .navigationTitle(details.title)
}
```

A view model getter that derives the record itself — `var title: String { details.title }`, `var showEarnButton: Bool { details.state.showsEarn }` — looks free and is not: every getter the body reads crosses again, so one asset screen crossed a dozen times per pass. Getters that need the record take it as a parameter, and getters that are a one-line read of it are deleted: the view reads the record. Deriving inside an action (`onTogglePriceAlert`) is fine — that is one crossing per tap, not per frame.

Android gets this for free by collecting, not by reading: the record is derived once in the view model's flow and the composable reads the collected value.

```kotlin
val uiModel = assetInfo.map { info -> service.details(info.toInput()) }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)
```

Do not reach for a memo. A cache keyed on an `Equatable` input was tried on both of these screens and removed: it saved a crossing only when the body re-ran for an unrelated reason, it had to restate every input of the derivation to stay correct, and it hid the real problem — a screen that derives more than once per render.

### Sections, actions and destinations are records too

A row is not the only choice a screen makes, and the other three recur often enough to have the same answer.

**Which sections show, and which empty state.** A screen that splits one list into positions, pinned, recents and results decides that split from counts and whether a search is running. Left in the apps it becomes four booleans on each side that drift one at a time.

```rust
#[uniffi::export]
impl GemPerpetualMarketCounts {
    pub fn sections(&self, is_searching: bool, is_query_empty: bool) -> GemPerpetualMarketSections { ... }
}
```

**Which actions the screen or row offers.** Core returns the list of available actions as cases; the app renders each case as its own button. `GemStakeActionItem`, `GemHeaderButtonKind`, `GemAssetAction` and `GemFiatButtonAction` are this shape. An app that assembles the action list itself is deciding what the user is allowed to do, on its own, twice.

**What a tap means.** The destination of a row is Core's answer — `GemSelectRowAction`, `GemDelegationDestination`, `GemAcquireAssetFlow` — and performing it is the app's, with the app's own route type per [navigation values are app types](#navigation-values-are-app-types). Core says *open the validator*; the app decides that means pushing `DelegationValidator`.

**A limit comes with its answer.** When Core hands back a limit, it also answers what the limit implies, or each app invents the comparison. `GemWalletSearchLimits` returns `assets`, `perpetuals` and `nfts`, and both apps then wrote their own "is there more" check against different counts — iOS against the whole result, Android against the pinned and unpinned sum. A limit that only names a number is half an answer.

### A screen whose state changes is a session

A screen that only reads gets a record. A screen the user drives — typing an amount, choosing a provider, waiting on a quote — gets a **session**: an immutable record holding that screen's state, event methods that return a new session, and one method that derives everything the screen shows.

```rust
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatSession {
    pub quote_type: FiatQuoteType,
    pub buy: GemFiatOperation,
    pub sell: GemFiatOperation,
    pub available: GemBigUint,
}

#[uniffi::export]
impl GemFiatSession {
    pub fn on_amount_changed(&self, amount: String) -> GemFiatSession { ... }
    pub fn on_quote_results(&self, results: GemFiatQuotesResult) -> GemFiatSession { ... }
    pub fn view_state(&self, asset_price: Option<f64>, is_url_loading: bool) -> GemFiatViewState { ... }
}
```

The view model's whole job on an event becomes one line, and every decision the screen makes — which phase it is in, whether the button is enabled, what the amount check says — is one Rust test away instead of two app tests that can disagree.

```swift
var type: FiatQuoteType {
    get { session.type }
    set { session = session.onTypeChanged(quoteType: newValue.map()) }
}
```

```kotlin
fun setType(type: FiatQuoteType) {
    session.update { it.onTypeChanged(type.toGem()) }
}
```

This is as far as a view model should move into Core, and the limits are the point:

- **The session is pure.** No `await`, no store, no clock, no network. Work that needs those is a service call the app makes, and its outcome re-enters through an event. A session that could block is a service wearing the wrong name.
- **What the session cannot know is an argument to `view_state`, not stored state.** The asset price and whether a URL is opening are the app's facts, so they are parameters. That keeps the session a function of its own events and keeps its tests free of setup.
- **One `view_state` call returning one record.** Ten getters are ten crossings per render; one record is one, and the app can memoize it by comparing the session it came from.
- **A `Record`, not an `Object`.** Value semantics let the app diff old against new to decide whether to recompute, and let a Rust test assert a whole screen state as one literal.
- **One session per screen**, the same rule as [one Core service per screen](#7-at-most-one-core-service-on-ios-narrow-cases-on-android). A view model holding two sessions is describing two screens.

Core has no observation primitive and no lifecycle, which is why the reactive half stays in the app. The view model owns the task, the debounce, the cancellation and the navigation; the session owns the answers.

### A screen's state is one phase enum, never a bag of flags

`is_loading`, `error` and `data` describe eight combinations, half of them nonsense. Core collapses them into one enum with one variant per screen the user can actually see, and each app switches over it exhaustively.

```rust
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemChartPhase {
    Loading,
    Data { data: GemChartData },
    NoData,
    Failed { error: GemServiceError },
}

impl GemChartSession {
    pub fn view_state(&self) -> GemChartViewState {
        GemChartViewState {
            period: self.period,
            phase: self.phase(),
            is_refreshing: self.is_refreshing,
        }
    }
}
```

```swift
var chartState: StateViewType<ChartValuesViewModel> {
    switch session.viewState().phase {
    case .loading: .loading
    case let .data(data): ChartValuesViewModel(period: selectedPeriod, chartData: data).map { .data($0) } ?? .noData
    case .noData: .noData
    case let .failed(error): .error(error)
    }
}
```

```kotlin
val chartUIState = loaded.map { state ->
    ChartUIModel.State(
        period = state.period.toPrimitives(),
        chart = when (val phase = state.phase) {
            GemChartPhase.Loading -> StateViewType.Loading
            is GemChartPhase.Data -> StateViewType.Data(ChartUIModel(phase.data))
            GemChartPhase.NoData -> StateViewType.NoData
            is GemChartPhase.Failed -> StateViewType.Error
        },
    )
}
```

Four rules keep the collapse honest:

- **The phase is derived, never stored.** The session keeps the raw facts — the loaded chart, the last error, whether a first load is outstanding — and one private rule decides which phase they add up to. A stored phase is a second source of truth that drifts from the facts that produced it.
- **Empty is not a failure.** `NoData` is its own variant, so a series with one point renders the empty state instead of an error, and neither app has to guess from an `Option`.
- **A progress flag that coexists with content is a field, not a variant.** A refresh happens *while* data is on screen, so `is_refreshing` sits beside the phase; anything that replaces the screen is a variant.
- **Everything the phase needs is inside the session.** The chart session carries its display currency because the phase cannot be computed without it — so no caller has to supply a value, and nobody can supply the wrong one. A `view_state` that takes what the screen already asked Core for is a parameter the session should own.

The app switches and stops. No `if isLoading` ahead of the switch, no `default:` inside it: the exhaustiveness is what makes a new variant a compile error on both platforms instead of a blank screen on one.

### A number crosses as a value and a style, never as a string or a callback

The precision ladder, the adaptive rule and its constants (`0.99`, `1e-10`, `100_000`, `0.1`, `0.0001`), the fiat-pins-to-two-places rule and the dust threshold are decisions, and they live in Core: `GemCurrencyStyle::precision`, `GemValueStyle::precision`, `adaptive_precision`, `abbreviation_threshold` and `dust_threshold`. Both apps ask for the precision and render it with their own locale formatter. What is left is the numbers themselves: a row that carries a bare `f64` still leaves each app to pick the style.

Two mechanisms are tempting and both are wrong.

**Do not export a formatter as a foreign trait.** A `GemCurrencyFormatter` the apps implement would let Core call back for every number, and a view state with fifty rows and three numbers each becomes a hundred and fifty reverse crossings inside one call — the most expensive direction there is, against the rule above. It also breaks a real boundary: [`Formatters`](../ios/Packages/Formatters/) holds only locale formatting, which is why the formatters that read a Core rule live in `GemstonePrimitives`. And it makes a session impure, so a screen's state can no longer be asserted as one literal in a Rust test.

**Do not return a finished string either.** Core's formatter is not locale-aware, so a Core-formatted amount regresses every locale that groups or separates differently.

**A number crosses as its value plus the style that decides how precisely it reads.** Core owns the choice; the app owns the rendering, with one small renderer per platform and no crossing per number.

```rust
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNumberUnit {
    Currency { code: String },
    Symbol { symbol: String },
    Plain,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNumberDisplay {
    Number { precision: GemPrecision },
    Abbreviated,
    BelowThreshold { threshold: f64, places: u32 },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNumberNotation {
    Plain,
    Signed,
    Parenthesised,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemValueTone {
    Plain,
    Neutral,
    Positive,
    Negative,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFormattedNumber {
    pub value: f64,
    pub unit: GemNumberUnit,
    pub display: GemNumberDisplay,
    pub notation: GemNumberNotation,
    pub tone: GemValueTone,
}
```

The style is resolved when the record is built, not carried for the app to re-ask: `GemFormattedNumber::currency` and `GemFormattedNumber::amount` take the value and a `GemCurrencyStyle` or `GemValueStyle` and settle the precision, the abbreviation and the dust cut once. The app reads `display` and renders with `NumberFormatter` or `DecimalFormat`, so a view state with fifty rows costs no crossings at all.

**A number's sign, its enclosure and its tone are decisions too, and one field each.** `notation` says whether the rendered digits get a forced `+`, get wrapped in parentheses, or neither — three mutually exclusive cases, not two booleans the app can combine into a fourth that means nothing. `tone` says whether the number reads as plain text, as a signed quantity that happens to be zero, or as up or down: a price is `Plain` however it moves, a delta is coloured by its sign. Deriving the tone app-side from `value > 0` is how the two apps drifted into colouring a chart headline differently, so the record answers it and each app maps the four cases to its own palette, once, in its module's style mapper.

```swift
extension GemValueTone {
    public var color: Color {
        switch self {
        case .plain: Colors.black
        case .neutral: Colors.gray
        case .positive: Colors.green
        case .negative: Colors.red
        }
    }
}
```

```kotlin
@Composable
fun GemValueTone.color(): Color = when (this) {
    GemValueTone.PLAIN -> MaterialTheme.colorScheme.onSurface
    GemValueTone.NEUTRAL -> MaterialTheme.colorScheme.secondary
    GemValueTone.POSITIVE -> MaterialTheme.colorScheme.tertiary
    GemValueTone.NEGATIVE -> MaterialTheme.colorScheme.error
}
```

A row or view state carries `GemFormattedNumber` where it carries a bare `f64` today. The adaptive rule, the abbreviation threshold and the dust cut live once, in Core, with tests that fail if a constant moves.

### An app row model holds the Core record; it does not restate its fields

A row model that copies `url`, `host` and `isSelected` out of a Core record is a partial twin: the copy has to be maintained, and a field added in Core reaches the screen only after someone widens the copy. Hold the record and read through it.

The test is what the model carries that the record cannot, not how much they overlap. A screen input that holds two app assets and a max-amount flag and *projects* the Core request from them is not a twin, however many field names rhyme; a record that carries the same fields re-typed, so that both have to move together, is one whatever it is called.

```kotlin
data class NodeRowUiModel(
    val node: GemNodeSelection,
    val canDelete: Boolean = false,
    val statusState: GemNodeStatusState = GemNodeStatusState.Loading,
)
```

The model adds only what Core does not own — here whether the row can be deleted and the status of its last check. This is the same rule as [no hand-written twins](#6-where-derived-domain-answers-live), applied to the presentation layer, and it keeps the two apps' row models the same shape: iOS's `ChainNodeViewModel` holds `node: GemNodeSelection` for the same reason.

### A row model conforms to a UI protocol; it stores nothing but the row

A row model still earns its place when a list protocol demands one — it is the type that conforms. What it may not do is keep a second source of truth beside the record:

```swift
struct PriceAlertItemViewModel: ListAssetItemViewable {
    private let row: GemPriceAlertRow

    var name: String { row.title }
    var symbol: String? { row.symbol }
}
```

```kotlin
class PriceAlertDataAggregateImpl(
    private val row: GemPriceAlertRow,
) : PriceAlertDataAggregate {
    override val title: String = row.title
    override val titleBadge: String = row.symbol.uppercase()
}
```

One stored property, and every member reads through it. When a member cannot — a title, a symbol, an id for the image — that field belongs on the Core row, not on a second thing the model holds alongside it. Holding the domain object *and* the row is the twin this rule exists to prevent: the model then has two places to answer the same question, and the two apps answer it differently.

Three questions settle where a member goes, in order:

1. **Is it a decision?** Which label, which order, which style, whether a thing is shown — it goes in the Core row. Never recompute it from the domain object the row was built from.
2. **Is it a number?** It crosses as a `GemFormattedNumber` and renders through the one `text()` each app already has — prices, amounts and percentages alike. Never add a per-row text helper; if a number needs a new shape, give `GemNumberUnit` that shape once so every row gets it.
3. **Is it which slot a value lands in?** Whether a row shows a label on the left and a price on the right, or a price and a percentage, is a decision. Core names the slots — `prefix` and `suffix` carrying a text key, a number, or nothing — so neither app switches on the kind to lay the row out.
4. **Is it a platform value?** A `Color`, an `Image`, a Compose or SwiftUI value — these are the only things the apps still decide, and they go on a row extension beside the module's other Gemstone mappers, never inline in the row model. The model reads `row.directionColor`; it never switches on a Core enum itself.

A row model whose body is anything but `row.` lookups has taken back a decision Core had already made. If you are writing `switch row.kind` in an app, the row is missing a field.

```swift
public extension GemPriceAlertRow {
    var directionColor: Color { direction?.color ?? Colors.gray }
    var prefixText: String { prefix.text }
}
```

```kotlin
@Composable
internal fun GemPriceAlertText.string(): String = when (this) {
    is GemPriceAlertText.Empty -> EMPTY
    is GemPriceAlertText.Number -> value.text()
    is GemPriceAlertText.Label -> label.string()
}
```

**Two mapper files per module, never one per type.** Every Core value a module turns into an app value goes into one of exactly two files, and a new `SomeGemType+Module.swift` is always the wrong answer:

| | iOS | Android |
|---|---|---|
| Core key → localized text | `Gemstone+Localized.swift` | `localization/GemstoneText.kt` |
| Core enum → colour, image, or other platform value | `Gemstone+Style.swift` | the module's style mapper |

A number's text never belongs in either — it comes from `GemFormattedNumber.text()`, which both apps already have once.

### Keep the crossings few

A record crosses by copy. Every call carries its arguments and its result across the boundary, so the cost of this design is counted in calls, not in Rust work. Four rules keep it flat.

**Never call Core from a `body` or a `@Composable`.** Those re-run on every frame and every recomposition. A Core call belongs in the model, computed when the data changes. `ContactChainSelectScene` calls `chainService.getChains(query)` inside a composable and gets away with it only because it is wrapped in `remember`; without that wrapper it is a crossing per recomposition.

**One call per list, not one per row.** A rule that takes the whole list — `matching_assets(assets, query)` — costs one crossing; the same rule as `matches(asset, query)` costs one per item and copies each item twice. `transaction_row(transaction)` is the shape to grow out of: both apps call it once per transaction, sending a whole `TransactionExtended` in and a row back, so a page of 500 is 1000 record copies. Prefer `rows(items) -> Vec<Row>` for anything that can be long.

**Build rows once per data change, and memoize.** Android wraps row construction in a map keyed by the source value; iOS builds them inside the `ObservableQuery` that produced the rows. Neither rebuilds per render, and that is the requirement, not an optimization.

**Hand over the rule, not an answer per item.** When a decision compares each item against a small fixed set — three cases, a threshold, a set of ids — Core can hand the app the *boundaries* once and let the app compare locally, at zero further cost. A day header is today, yesterday, or a date, so Core answers what today and yesterday are and the app matches each section against those two values:

```rust
#[uniffi::export]
impl GemDayBoundaries {
    pub fn label(&self, day: GemDay) -> GemDayLabel {
        match day {
            day if day == self.today => GemDayLabel::Today,
            day if day == self.yesterday => GemDayLabel::Yesterday,
            _ => GemDayLabel::Date,
        }
    }
}
```

```swift
let boundaries = GemDayBoundaries.current          // one crossing per list build
switch boundaries.label(day: date.gemDay) {
case .today: Localized.Date.today
case .yesterday: Localized.Date.yesterday
case .date: Self.sectionFormatter.string(from: date)
}
```

```kotlin
private val boundaries = LocalDate.now().gemDay().boundaries()   // one crossing per list build

fun format(date: LocalDate, locale: Locale): String = when (boundaries.label(date.gemDay())) {
    GemDayLabel.TODAY -> todayLabel
    GemDayLabel.YESTERDAY -> yesterdayLabel
    GemDayLabel.DATE -> DateTimeFormatter.ofLocalizedDate(FormatStyle.LONG).withLocale(locale).format(date)
}
```

Core still owns what could drift — that yesterday is exactly one calendar day back, across month, year and leap boundaries, and that anything else is a plain date — while the per-section work is a value comparison. Prefer this to a call per item, and to a batch call, whenever the rule can be expressed as a small set of values the app can hold. Grouping stays platform work either way: Core has no device time zone.

**A crossing costs more on Android than on iOS.** iOS reaches Rust through a direct C call; Android goes through JNA, which is enough slower that a per-item crossing shows up in a list. Size the design for Android: batch per list, and where a rule is three lines that each platform's own framework already answers identically — a calendar's today/yesterday, a locale's grouping separator — leaving it in both apps is cheaper and no more likely to drift than paying a crossing per row to share it. That trade is only correct when the two implementations genuinely cannot disagree; write down which it is.

**Derive the view state; do not store it.** Android composes it declaratively — `combine(session, isUrlLoading, assetPrice) { session.viewState(...) }` — and iOS's equivalent is a computed property, because the inputs a screen does not own arrive from a database observation it cannot hook. Storing the result and updating it by hand goes stale the moment one of those inputs changes without a call site remembering. Derive on read, and keep the crossings down with `DerivedValue`, which recomputes only when a named `Equatable` input changes:

```swift
@ObservationIgnored private let derivedViewState = DerivedValue<ViewStateInput, GemFiatViewState>()

var viewState: GemFiatViewState {
    derivedViewState(ViewStateInput(session: session, assetPrice: priceUsdQuery.value, isUrlLoading: urlState.isLoading)) {
        $0.session.viewState(assetPrice: $0.assetPrice, isUrlLoading: $0.isUrlLoading)
    }
}
```

```kotlin
private val viewState = combine(session, isUrlLoading, assetPriceUsd) { session, isUrlLoading, priceUsd ->
    session.viewState(priceUsd, isUrlLoading)
}.stateIn(viewModelScope, SharingStarted.Eagerly, session.value.viewState(null, false))
```

The input struct names what the state depends on, so a new dependency is a compile-time edit rather than a forgotten refresh.

**Memoization needs equality.** A session or view state is memoized by comparing the value it came from, so the record must derive `PartialEq` — a record that does not silently never hits its cache. This is also why screen state is a `Record`: it copies once and every read afterwards is free, where a `uniffi::Object` crosses by handle but charges a crossing for every property read. Use an `Object` only when the state is large and read rarely, or when it owns something Rust-side.

### Field types

- Big-integer atomic quantities are `GemBigInt` / `GemBigUint`, never `String`. `String` moves the parse to every call site, and each one invents its own failure behaviour. The bindings type them too (`core/gemstone/uniffi.toml`): Kotlin sees `java.math.BigInteger`, Swift sees `BigInt` / `BigUInt`, so an app never parses a Core value and never `.toString()`s one to hand it back. The only parses left on the apps are of typeshare models and database columns, which are strings by generation.
- `amount` is for `f64`. `value` is for big integers. Do not mix them.
- Full domain words, per the shared [naming rule](../skills/engineering-principles.md#clean-code-principles).

## 4. The store trait is the app's only persistence obligation

Core declares what it needs; each app implements it over its own database. Nothing else about the app's storage crosses the boundary. Apps may also implement narrow foreign ports for OS-only capabilities such as secure storage, notifications and sockets.

```rust
// services/<feature>/store.rs
#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemPerpetualStore: Send + Sync {
    async fn save_perpetuals(&self, data: Vec<PerpetualData>) -> Result<(), GemServiceError>;
    async fn get_positions(&self, wallet_id: WalletId, provider: PerpetualProvider) -> Result<Vec<PerpetualPosition>, GemServiceError>;
}
```

An adapter maps reads and writes and nothing more — **no rules or mapping implementation inline**. Calling a named mapper or standard boundary conversion is expected; non-trivial mapping lives in a mapper file beside the adapter (`StoreModels.kt`, `nft/NftModels.kt`).

**Stores only write rows whose values differ.** A blanket write churns observers and hides real changes.

## 5. The app maps; it does not decide

### iOS

There is no app-side service wrapping a Core service. The view model holds its screen's Core service and calls it; each call is one line in, one mapping out.

```swift
func preload(request: ConfirmTransferRequest, selection: FeeSelection, feeAssetSelection: FeeAssetSelection) async throws -> ConfirmTransferPreload {
    try ConfirmTransferPreload(
        await service.preload(
            walletId: request.wallet.id.id,
            input: try request.confirmInput(),
            options: options(selection: selection, feeAssetSelection: feeAssetSelection),
        )
    )
}
```

Core → app mappings live in `GemstonePrimitives` as extensions. A mapping onto a *feature-internal* type stays in the feature — `GemstonePrimitives` cannot import a feature module, and reaching for one is the signal that the mapping belongs in the feature.

### Android

A case in `gemcore` `application/<area>/cases/`, implemented in `data/coordinators/<area>/`, injected by Hilt. An observed read returns a `Flow`; the case still asks Core for the decision on each emission:

```kotlin
override fun getTransactionDetails(id: TransactionId): Flow<TransactionDetailsAggregate?> = combine(
    getSession().filterNotNull(),
    getTransaction(id),
) { session, data -> Pair(session, data) }
    .mapNotNull { (session, data) ->
        data?.let { TransactionDetailsAggregateImpl(it, transactionDetailsService.detailRows(it.toGem()), session.currency) }
    }
    .flowOn(Dispatchers.IO)
```

One Core call answers the whole screen, so the case has nothing to assemble.

The store is the change trigger. Core is the decider. Core has no observation primitive, and that is the only reason the app watches its own tables.

### Navigation values are app types

A navigation stack matches a pushed value, or a route key, to its destination by exact type, and it does so silently: a push with no destination for its type does nothing, reports nothing, and still compiles. The push and the destination sit in different modules, so the route is a contract between two files that never see each other, and it has to be a type the app owns and changes deliberately. A generated transport record is not that type — its shape follows the FFI, so replacing one generated type with another under a screen breaks every link that pushed the old one while the build stays green. On both apps the matched type is the app's own — a `Primitives` value when the destination already takes one, otherwise an app input that carries the payload — and never the generated record itself. Map to it in the model rather than in the scene, and let the record stay the payload the screen reads once it is there. Exercising the link is the only thing that proves a route. The same line holds for which screen is showing: iOS pushes a `navigationDestination`, Android swaps an `AnimatedContent` over a page enum, and neither is a decision Core can hold without forcing one platform's idiom on the other. What crosses is what each page *needs* — the gates, the validity, the resolved value — not which one is on screen.

```swift
// iOS scene: the row pushes the app's validator, the selection maps Core's row
NavigationLink(value: validatorSelection.selectedValidator) {
    ValidatorView(model: ValidatorViewModel(row: validatorSelection.selected))
}

public extension SelectionState where T == GemValidatorRow {
    var selectedValidator: DelegationValidator {
        selected.validator.map()
    }
}

// iOS stack: the destination takes the same app type
.navigationDestination(for: DelegationValidator.self) { validator in
    ValidatorSelectScene(model: viewModelFactory.validatorSelectScene(currentValidator: validator, ...))
}
```

```swift
// an app input carries a payload that has no Primitives equivalent
public struct ConfirmTransferInput: Hashable {
    public let data: GemTransferData
}
```

```kotlin
// Android: a route key is a serializable record of primitives
@Serializable
data class AssetRoute(val assetId: AssetId) : NavKey
```

Pushing `GemValidatorRow` and matching `.navigationDestination(for: GemValidatorRow.self)` compiles and works until one of the two changes; pushing it against the destination above compiles and does nothing.

### A view never names a Core type

The boundary is the view, not the view model. A view model holds the Core record it was given — declaring a twin of it is a [separate violation](#6-where-derived-domain-answers-live) — and exposes platform values from it. What must not appear inside a SwiftUI `body` or a `@Composable` is the generated type itself: a view that names one is pinned to a transport shape it does not own, and the shape follows the FFI, so a field that moves in Core edits screens instead of models.

Two leaks account for most of it. A `switch` over a Core enum picking a localized string inside the view, and a Core record handed to a child view's initializer. Both move into the model unchanged — localization stays app-side, only the type stops crossing.

```swift
// scene
Text(model.errorText ?? "")

// model: the Core enum stops here
var errorText: String? {
    switch viewState.phase {
    case .invalidInput: Localized.Errors.invalidAmount
    case let .invalid(check): text(for: check)
    case .noInput, .loading, .ready, .noQuotes, .failed: nil
    }
}
```

```kotlin
// composable
FooterText(state.errorText)

// view model: the Core enum stops here
private fun errorText(phase: GemFiatQuotePhase): String? = when (phase) {
    GemFiatQuotePhase.InvalidInput -> context.getString(R.string.errors_invalid_amount)
    is GemFiatQuotePhase.Invalid -> checkText(phase.check)
    is GemFiatQuotePhase.Failed -> context.getString(R.string.errors_unknown_try_again)
    GemFiatQuotePhase.NoInput, GemFiatQuotePhase.NoQuotes, is GemFiatQuotePhase.Loading, GemFiatQuotePhase.Ready -> null
}
```

Naming a Core type is not the test; deciding from one is. A view that iterates a row key and hands each case to a component is doing what [§ 3](#3-return-one-record-that-answers-the-whole-question) asks — the key is the screen's contract, and the switch over it is exhaustive on purpose. Twelve of the seventy-five iOS scene files and just under half of the Android `presents/` files name a Core type for exactly that reason.

What to grep for is a view that answers a question instead of asking one: a `switch`/`when` over a Core type whose arms produce `Localized.` or `stringResource` — the localized text belongs to the module's mapper, not the body — or a Core record passed into a child view's initializer. On Android:

```
rg -tkotlin -U 'when \([^)]*\)\s*\{[^}]*(stringResource|R\.string)' android/features/*/presents --glob '!**/localization/**'
```

and its Swift equivalent over `Sources/Scenes/` and `Sources/Views/`, excluding the module's `Gemstone+Localized.swift`. A hit is a decision that has to move one layer down; everything else the type-name grep finds is the contract working.

### One mapper per module names every Core key it renders

A module resolves every Core key it shows in one file: `Sources/Types/Gemstone+Localized.swift` on iOS, `presents/localization/GemstoneText.kt` on Android, with `Gemstone+Style.swift` / `presents/style/GemstoneStyle.kt` beside them for the icon and colour a key picks. A view model, a scene or a composable that maps a key somewhere else has taken the module's vocabulary private, and the two apps drift one key at a time — `GemAmountTitle.rewards` read "Claim Rewards" on one app and "Rewards" on the other for exactly this reason, and the import screen labelled the same field two ways.

The mapper is the only place a `Localized.`/`R.string` is chosen from a Core variant, which is what makes the two apps comparable. `just check-mappers` parses each app's mapper files into variant → key, resolves both keys to their English text, and fails on every variant the two apps resolve differently. `just check-docs` does the same for the guidance: every link in `docs/`, `skills/` and the `AGENTS.md` files has to point at a file that exists, a heading that exists, and — when the label is a backticked name — a symbol that is still in that file. It only sees a variant while both mappers hold it, so a key mapped anywhere else is invisible to it.

Two files per module, no exceptions: if a screen needs a second phrasing of the same key — a tab title and a field label — both live in that one file under different names, the way Android's `tabStringRes` and `fieldStringRes` do.

### A UI state class translates the view state; it does not re-shape it

Compose screens collect a single state object, and the temptation is to declare it as a mirror of the Core view state — the same fields, re-typed. That is a twin with a different name: every Core field it restates has to be maintained in step, and the Core types it carries reach the composable anyway, so the [view boundary](#a-view-never-names-a-core-type) is not actually closed.

A UI state class is the **translation** of the Core view state into platform values. It holds no Core type. Every property is a `String`, a `Boolean`, a `@StringRes` id, or an app enum, and each one names what the view does with it rather than what Core called it.

```kotlin
data class FiatUiState(
    val isLoading: Boolean = false,
    val errorText: String? = null,
    @StringRes val actionTitle: Int = R.string.common_continue,
    val retries: Boolean = false,
    val buttonState: ButtonState = ButtonState.Disabled,
    val canSelectProvider: Boolean = false,
)
```

Compare it with what it replaced, which carried `GemFiatQuotePhase`, `GemFiatAmountCheck` and `GemFiatButtonAction` straight through: the composable then had to branch on all three, and a new Core case broke a screen instead of a model.

Two consequences worth stating:

- **Localization happens in the view model,** because that is where the branch over the Core enum now lives. An Android view model takes `@ApplicationContext` for this, as `NetworkAssetsViewModel` and `AssetsResultsViewModel` already do; a `@StringRes` id on the state is the alternative when no arguments are needed. iOS resolves `Localized.*` in the model for the same reason.
- **The tests improve.** A test that asserted `uiState.phase == GemFiatQuotePhase.Failed` was asserting a Core value the user never sees; asserting `uiState.errorText` and `uiState.retries` checks what the screen actually shows.

The same rule reads on iOS as: the view model exposes `String`, `Bool` and app enums; the Core record stays private behind them. A row model is the small case of this — it holds the Core record and exposes platform values from it — and a UI state class is the screen-sized one.

### A list row renders from one shared row model

Both apps draw a plain list row from one component model: `ListItemModel` in iOS `Components` and `android/ui`. It carries the finished title, the title tag, the extra line under the title, the value on the right with its extra line, the leading image and the info sheet, each with a style the view resolves to a font and colour. A sectioned screen returns `ListSection<T>` — an id, an optional title and the items — and the scene renders it with one call: `ForEach` over the sections on iOS, `listSections` on Android.

The view model builds the model and the scene only renders it: `ListItemView(model: model.listItem(for: row))` and `ListItem(model = row.model, listPosition = position)`. A row that also acts carries the app action beside the model — `SettingsRowUIModel(action, model)` — and the scene reads the action, never the row key. An address row carries the raw address, chain, name and explorer link and lets the scene format the address, because the address formatter is a composition local. A switch or a picker renders the same model with the control in the accessory slot; a dropdown picker keeps its options beside the model.

Three kinds of row stay outside the model on purpose. The row primitives themselves (`PropertyItem`, `LinkItem`, `ListItem`) are what the model renders through. Rich rows with their own layout — asset, wallet, chain, NFT, transaction, delegation, validator and swap-provider rows, and the network, validator and balance composites — have an iOS twin of their own (`ListAssetItemView` and friends) and keep it. Developer screens keep inline rows on both apps. Vector icons never reach a view model: Android carries `ListItemImage.Symbol` and resolves it to `AppIcons` in the image view. The toast a view model emits is the same `ToastMessage(title, image)` on both apps.

What this replaces is the row assembled in the body: `ListItemView(title:subtitle:imageStyle:)` with values read off the view model one by one, and `PropertyItem`/`LinkItem` calls with a title id, a value and an icon passed as separate arguments. Both restate the row's shape on every screen, and the two apps then drift a field at a time. Copy [`ContactsViewModel.listItemModel(for:)`](../ios/Features/Contacts/Sources/ViewModels/ContactsViewModel.swift) and [`ContactsViewModel.listItem`](../android/features/settings/contacts/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/contacts/viewmodels/ContactsViewModel.kt) for a row, [`PriceAlertViewModel`](../android/features/settings/price_alerts/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/price_alerts/viewmodels/PriceAlertViewModel.kt) for sections, and [`RecipientViewModel`](../android/features/recipient/viewmodels/src/main/kotlin/com/gemwallet/android/features/recipient/viewmodel/RecipientViewModel.kt) for sections whose rows format an address through Core.

### Never call Core from the main thread

The `flowOn` above is not decoration. A synchronous Core call such as `transactionDetailsService.detailRows` can read store callbacks that block on Room, and UniFFI polls the Rust future on the calling thread — so without it the read lands on main, where Room throws before any work happens.

The coordinator dispatches; it does not leave that to its caller. Whether a Core method touches a store is Core's business and can change without the call site noticing.

The one call that stays on the calling thread is a preference read a synchronous Core rule takes as an argument. `UserConfig` reads a locally stored fact and hands it to the rule; moving it off main would turn a synchronous answer into an asynchronous one for no gain. Every other Core call takes the dispatcher at its boundary.

```kotlin
// suspend: move the call
override suspend fun invoke(...): List<FiatQuote> = withContext(Dispatchers.IO) {
    fiatService.getQuotes(...).map { it.decodeJson<FiatQuote>() }
}

// Flow: flowOn after the operator that calls Core
override fun getTransactionDetails(id: TransactionId): Flow<TransactionDetailsAggregate?> = observed(id)
    .mapNotNull { data -> transactionDetailsService.detailRows(data.toGem()) ... }
    .flowOn(Dispatchers.IO)
```

## 6. Where derived domain answers live

The app uses Core's types. It does not declare a parallel record or enum of the same shape — that is two definitions of one thing, and every crossing pays for a two-way mapper. `TransferDataType` was that copy: eleven restated cases and 138 lines of mapping. It is deleted.

Choose the home from ownership first, then decide how it crosses FFI:

| Question | Home | Example |
|---|---|---|
| A field the value already carries, but the generated shape lacks a common accessor | thin app mapping extension | `inputType.asset` |
| A pure answer has one honest domain receiver; additional value arguments are allowed | method on the receiver | `bannerKey.identifier()`, `input.addAddress(addresses)` |
| A pure rule has no honest receiver | private rule called by the owning service | `selectable_fee_assets(...)` |
| The answer requires I/O, stored dependencies or platform ports | method on the service that owns the flow | `confirmService.preload(...)` |
| An app value must be encoded into a Core case | app mapping extension | `.stake(asset, stakeType)` |

**Never add a free exported function or a service wrapper — stateless or not — for an answer already owned by one local Core type.** `transaction_input_asset(input_type)` and `transferService.asset(inputType:)` both hid the natural receiver and were replaced by `inputType.transactionAsset()`; that is the shape to reach for. A method that ignores `self` is the same mistake even on a service with real dependencies: `confirmTransferService.simulationAssetIds(simulation:)` was a property of `SimulationResult`, not of an eight-dependency orchestrator, and moved there. Keep the owning service when the rule performs I/O, holds real dependencies, or combines inputs without a single honest receiver. A request record is an honest receiver when it contains the complete instruction: `GemContactAddressInput.add_address(addresses)` owns its replacement identifier and new address; `GemManageContactService.add_address(addresses, input)` would ignore every service dependency.

Do not manufacture a receiver by choosing the first parameter. The type is honest only when the answer is part of that type's meaning, the method uses `self`, and extra arguments are plain input values rather than stores, clients or services. For a repository-owned Rust type, prefer an inherent `impl Type`; do not create a one-method `TypeExt` trait to imitate Swift or Kotlin. Intrinsic structure belongs in the defining crate (`SimulationResult.asset_ids()`), while feature or product policy remains in Gemstone even when it consumes a primitives type.

### Ownership is not transport

A receiver method on a Gemstone-local UniFFI record or enum can cross FFI. Export it only when an app calls it. A TypeShare type defined in a repository-owned Core crate can own canonical inherent Rust behavior there, but TypeShare does not generate that method in Swift or Kotlin. A type merely declared through `#[uniffi::remote]` is defined elsewhere, so Gemstone cannot add an inherent receiver; use a private rule or adapter rather than inventing one. A thin app extension may expose a structural field projection when transport omits it, but it must not copy product policy.

When mobile needs behavior that cannot cross on its honest receiver, prefer folding the answer into an existing aggregate operation. If the UI genuinely needs a standalone pure projection, add it to an existing cohesive FFI adapter such as `GemSimulationFormatter`; do not put it on an I/O service whose dependencies it ignores and do not create a one-method object. This is how a TypeShare-only `SimulationResult.asset_ids()` should reach mobile without duplicating the rule.

A rule that answers for one value is a constructor on that value, not a method on an object with nothing in it: `GemSwapQuoteSummary::new(quote)` carries the minimum receive and the ETA of a quote, `GemSwapValue::price_impact(receive)` compares two priced amounts, `GemCustomFee::estimate` and `GemTransactionRow::new` do the same for fees and rows. The apps and their tests construct the value; nothing has to be mocked to reach a rule. `GemSwapQuoteService` carried those rules as a `new()` with no fields until it became the swap screen's real service — swap, balances, preferences and the price stream behind one object — and the rules moved onto their values.

A stateless exported object is acceptable only as a cohesive FFI codec or formatter when UniFFI cannot express an honest receiver or the operation spans several transport types. Name that role explicitly, give it no I/O dependencies, and delegate intrinsic behavior or feature policy to the owning receiver or private rule where possible. `GemSimulationFormatter` and `PriceAlertFormatter` are the current transport adapters. A one-call forwarding object is still a wrapper and should be removed.

`#[uniffi::export]` on an `impl` processes every function in that block regardless of Rust visibility. `pub(crate)` does not remove a method from generated bindings. Put only intended FFI methods in the exported block; move helpers to a separate unannotated `impl` and give them the narrowest Rust visibility. Derive `uniffi::Record`/`uniffi::Enum` only for types that actually cross FFI. After changing an exported member or type, regenerate bindings and build both apps — one platform may never have called the method the other still needs.

The generated code mapper crashing on an unknown `Chain` or `Currency` is not the same problem and is not wire handling: the app's enum is typeshare-generated from the same Core enum in the same build, so the two cannot disagree at runtime, and a crash there reports a broken generation rather than a server value. The one path that can reach it is a downgrade reading a store a newer build wrote; that is not a supported install path, and widening the mapper would hide the generation bug it exists to catch.

A list from the wire skips the entries this build cannot read rather than failing whole: `deserialize_known_entries` drops an element whose enum this build does not know and keeps the rest, so a currency added server-side costs one rate, not every price. Use it on any payload whose elements carry an enum the server can extend.

An app's persisted aggregate is not a typeshare candidate just because Core has one of the same name. `AssetData` was the standing example of the temptation: typeshare renders a `BigUint` as a `String`, so Core's ten-field `Balance` would reach both apps as text to parse; Android's `Balance<T>` is a generic container it instantiates with `BigInteger` and with `Double`, which typeshare cannot express; and the two apps' `AssetData` differ from each other — iOS carries price alerts, Android a wallet id — as well as from Core's. Both are persisted, which is where a twin belongs. Reopen a shape like this only if it keeps the atomic values numeric.

A generated mapper names its direction on both apps: `toPrimitives()` reads a Core value into the app's type and `toGem()` sends one back. An untyped `map()` on both sides made a diff unreadable and let a reviewer miss a crossing going the wrong way. A mapper between two app types keeps `map()` — the directional names mark the FFI boundary and nothing else.

Encoding members are scaffolding, not a pattern to copy. `core/bin/generate/remote_types.yml` lists what `just generate-models` maps: `remote` types get `#[uniffi::remote]` and structural mappers on both apps; `codes` are string-backed enums that cross as their code and get `Primitives.X(core:)` / `.rawValue` on iOS and `toX()` / `toGem()` on Android; `identifiers` are hand-written parsers the record mappers call by convention (`X(core:)` / `.identifier` on iOS, the `X(identifier)` constructor / `toIdentifier()` on Android). Before adding a `remote` type, verify that the generator can represent its full shape and inspect both generated mappers. It handles fieldless enums and records whose fields are scalars, `DateTime<Utc>`, other remote types, codes or identifiers, plain or wrapped in `Option` / `Vec`, whatever subdirectory of `primitives/src` declares them; a `#[typeshare(skip)]` field travels Core → app only and is filled with its empty value on the way back (scalars, `Option`, `Vec` and `String` have one; anything else fails generation). A data-carrying enum maps too when it keeps a twin, as long as each variant carries at most one unnamed payload — a twin renders a named or multi-field variant as a type of its own, which the generator will not invent. A type an app never looks inside does not need a model on either side: pass the wire text and let Core own the format. Never add a second app-side model or copy policy to avoid a gap in the generator; close the gap.

## 7. At most one Core service on iOS; narrow cases on Android

An iOS view model holds **at most one** Core service, named `service`, and it is **`private`**; a model that does not need Core holds none. Reuse the owning domain service when it already answers the screen. Add a screen-level service only when it genuinely composes collaborators or returns a cohesive screen result — never to satisfy a field-count rule. An Android view model holds the same Core service through its generated `GemFooServiceInterface` (`private val service`), plus the observed reads the screen watches as narrow application cases (a Room `Flow` behind `GetPriceAlerts`, `GetRecentAssets`, `SelectSearch`) and `GetSession`. A case that only forwards a Core call is migration debt: delete it and call the service. `SetPriceAlertsEnabled` over `set_enabled` and `SearchCustomToken` over `ensure_token_asset` were two such, both removed. A non-private service on iOS usually means the view is reaching through the model for a dependency.

**The observed read is each platform's own, and that asymmetry is the contract, not drift.** Core has no observation primitive, so the reactive half stays in the app — and the two apps do not have the same one to stay in. iOS holds `ObservableQuery<Request>` beside the service: an `@Observable` wrapper over a GRDB `ValueObservation` that restarts when its request changes, with `BindableQuery` as the one-method seam that lets the composition root hand it a `DatabaseQueue` without naming its generic. Android injects a narrow application case over a Room `Flow` (`GetPriceAlerts`, `GetRecentAssets`, `GetSession`) because Hilt already builds the DAO into it. Neither can adopt the other — SwiftUI has no `Flow` and Room has no property wrapper — so count an `ObservableQuery` beside a service the way you count a case beside one: an observed read, not a second owner.

**A value type that projects Core records holds no service, and that is the finished shape — not a gap.** The two apps count differently here and both are right: 156 iOS view models name no Core service where Android has 8, because an iOS row model is usually a `struct` with no observable state, no query and no async work, whose initializer takes the Core records it draws. `NetworkFeeSceneViewModel` takes `GemConfirmFeeSelection`, `GemFeeRateRows` and `GemFeeOptionItem` plus two callbacks and decides nothing; injecting a service into it would buy nothing to substitute. Android reaches the same place by injecting the service into a model its DI graph already builds. Judge a model by whether it *decides* anything the Core record does not already answer; a member count says nothing about ownership, and a service-less row model needs no migration.

This limit does not count explicit platform ports such as a signer, keystore, observation source or navigation builder. Those remain narrow injected dependencies; they do not decide shared product behavior.

A store is not one of those ports. A store is the database side of a Core service, and a view model that holds one has reached past the service into what the service owns — the same coupling a second service would be, by a shorter path. The developer screen was the last of these on iOS: it held `TransactionStore`, `AssetStore`, `StakeStore`, `BannerStore` and `PriceStore` beside `GemDeveloperServiceProtocol`, so the nine clear actions and the sample transaction table existed only on that platform. The operations moved onto `GemDeveloperStore` and the service exports them, which is what let Android offer the same actions from the service it already held:

```swift
DeveloperViewModel(walletId: walletId, service: developerService)
```

```kotlin
class DevelopViewModel @Inject constructor(
    private val service: GemDeveloperServiceInterface,
    val notificationsAvailable: NotificationsAvailable,
) : ViewModel()
```

A second service is never the way to reach a value the screen renders. When a view model needs an answer its own service does not hold, the fix is one of three, in order: the answer is a pure projection and becomes a function of the value it projects ([a row is projected from its value](#a-row-is-projected-from-its-value-never-fetched-from-a-service)); the screen's own service or session already receives the input and returns the answer alongside the rest of its view state; or the screen was drawn around the wrong service. Widening the constructor is not on the list, and neither is having the composition root call the other service and pass the result in — a factory line that reads `walletService.walletRow(...)` next to an unrelated service is the same coupling with a longer path.

When a real screen-level service is needed, name it for the screen it backs, not for the layer: `GemManageContactService` backs the add-and-edit screen. No `Scene` or `Facade` in the name. A `GemContactsService` that only forwarded calls to `GemContactService` was wrapper debt, not the pattern, and is deleted: the list screen holds the owning `GemContactService`. When a screen needs a cohesive answer from several Core owners, Core composes them:

```rust
#[derive(uniffi::Object)]
pub struct GemManageContactService {
    contacts: Arc<GemContactService>,
    addresses: Arc<GemAddressService>,
    names: Arc<GemNameService>,
    payments: Arc<GemPaymentService>,
}

#[uniffi::export]
impl GemManageContactService {
    #[uniffi::constructor]
    pub fn new(contacts: Arc<GemContactService>, addresses: Arc<GemAddressService>, names: Arc<GemNameService>, payments: Arc<GemPaymentService>) -> Self { ... }

    pub fn default_chain(&self) -> Chain { self.contacts.default_chain() }
    pub async fn save_contact(&self, input: GemContactInput) -> Result<Contact, GemServiceError> { ... }
    pub fn format_address(&self, address: String, chain: Chain, style: GemAddressFormatStyle) -> String { ... }
}
```

The pure address-list transformation stays on its request value: `input.add_address(addresses)`. It does not belong on this service because it uses none of the service's dependencies.

A sheet belongs to the screen that presents it and shares that screen's service. A screen you navigate *to* is a different screen with its own.

### Composition services are reached through the screen service

`GemExplorerService`, `GemDeeplinkService`, `GemSwapService`, `GemAssetConfigService`, `GemPriceService` are *composition* services: screen services hold them, and a screen reads their answers through its own service — the chart's token link is `GemChartService::token_url`, the confirm screen's sender link is `GemConfirmTransferService::address_url`, the asset scene's share link is `GemAssetDetailsService::deeplink_url`, its swap pair `swap_pair`, the confirm sheet's acquire flow `acquire_asset_flow`. A composition service's method is exported only while an app still calls it; once every screen reads it through its screen service, move the method to a plain `impl` block (the constructor stays exported because the composition root builds the object). The September 2026 sweep found the same answer reached three ways — iOS through the screen service, Android through the composition service from a Hilt-injected coordinator, and Android's Compose layer through a `CompositionLocal` — and each pair disagreed somewhere (the slippage default, the acquire-flow title). One route per answer, and it is the screen service's. A composition service therefore never reaches the Compose tree as a `CompositionLocal`; the navigation builder, the one caller that is not a screen, takes it as a parameter from the composition root.

On Android the Hilt module binds both the concrete class and the generated interface (`fun provideGemFooServiceInterface(service: GemFooService): GemFooServiceInterface = service`): Core constructors need the concrete type to compose, view models and coordinators take the interface.

### A service never hands out another service

`service.manageContact()` was the same reach-through as `model.nameService`, one level down: the caller depends on something it was not given. Every service is constructed in the composition root and injected. Returning `Arc<GemFooService>` from an exported service is migration debt, not an exception to this rule.

A **shared component** — `AddressInputViewModel`, `NetworkSelectorViewModel` — takes the Core service it needs by its own protocol: `AddressInputViewModel` and `NameRecordViewModel` take `any GemNameServiceProtocol` (`GemNameServiceInterface` on Android), and the parent view model receives that `nameService` as a plain constructor dependency beside its `service` and passes it down. The screen service does not forward name methods and the client does not declare a protocol intersection (`any GemFooServiceProtocol & AddressInputResolving`) or a builder closure to reach the component's dependency — both hide a second dependency inside the first. `NetworkSelectorViewModel` needs only the dependency-free `GemChainService` and builds it itself.

### The parent vends the child model, the view never reaches in

```swift
// wrong — the view assembles the child from the parent's internals
ManageContactAddressScene(
    model: ManageContactAddressViewModel(
        defaultChain: model.defaultChain,
        nameService: model.nameService,
        addressService: model.addressService,
        onComplete: model.onAddressComplete,
    ),
)

// right — the parent owns the wiring, the view asks for a model
ManageContactAddressScene(model: model.addressModel(mode: mode))
```

Where the child is a different screen with its own service, the parent cannot build it — feature modules cannot see the composition root. The app passes the builder in:

```swift
public func contactsScene(mode: ContactsViewModel.Mode = .list) -> ContactsViewModel {
    ContactsViewModel(service: contactsService, manageContact: manageContactScene, mode: mode)
}
```

Android does not hit this at all for a child of the same screen: one Hilt view model owns both pages and the composable switches on the page it reports, so there is no child model for a view to assemble.

```kotlin
AnimatedContent(targetState = uiState.page) { page ->
    when (page) {
        ManageContactPage.Form -> ManageContactScene(state = uiState, onAction = ...)
        ManageContactPage.Address -> uiState.addressInput?.let { input ->
            ManageContactAddressScene(input = input, onAction = ...)
        }
    }
}
```

The same applies to state: a view switching on the model's `mode` forces `mode` to be non-private. Name the decision on the model instead — `var rowAction: RowAction` — and the view switches on the answer, not the input.

### Depend on the generated abstraction, not the concrete object

On iOS, UniFFI generates a protocol for every exported object. `GemAddressServiceProtocol` exists; importing `class Gemstone.GemAddressService` at a consumer means that consumer cannot be substituted without relying on UniFFI's fragile no-handle test path. On Android the same holds for the generated `GemFooServiceInterface`: bind it in the Hilt module (`): GemReceiveServiceInterface = GemReceiveService(...)`) and inject the interface.

- **iOS consumers** (view models, components, validators) take `any GemFooServiceProtocol`.
- **Android consumers** take the generated interface, or the observed-read case, used by their layer.
- **The composition root** (`ServicesFactory`, `ViewModelFactory`) holds the concrete type — a UniFFI constructor needs it, and the root is the one place allowed to construct. Its fields are grouped by what they are: Core services, platform services, stores.

## 8. Services are injected, never constructed at a call site

A `GemFooService()` in a field initialiser or at file scope is a second instance the graph does not know about, and it is where an app-side variant creeps back in.

- **iOS** — an owner (a service with a store, a client, a stream, or anything the app needs from launch) is registered in `ServicesFactory`, exposed through an `@Entry` in `ios/Gem/Types/Environment.swift`, and passed into the view model. A screen service — one that only composes owners for a single screen (`GemAssetDetailsService`, `GemChartService`, `GemTransactionDetailsService`, `GemWalletHomeService`) — is built in the `ViewModelFactory.xxxScene(...)` that builds its view model, from the owners the factory already holds. It is never a field of `AppResolver.Services` and never an `@Entry`: that constructs it on every launch of an app that may never open the screen, and hands views a composition detail.
- **Android** — provided in a Hilt module, injected. A Compose scene reads one instance from a `CompositionLocal` provided at `MainActivity` (`LocalChainService`) only for the dependency-free config services; a screen's Core answers come from its view model's service, never from a `CompositionLocal` inside a feature composable (`LocalDeeplinkService` building the share link and `LocalAssetConfigService.acquireFlow` deciding the confirm button were reach-throughs and are gone). A non-`@Composable` helper takes an explicit parameter — a `CompositionLocal` cannot be read outside a composable. A screen service is a `@Provides` like any other — Hilt builds it when its view model first asks, so nothing is built at launch — and is never read from a `CompositionLocal`.
- **A value type or a namespace of statics** takes the service as a method parameter only when the answer genuinely requires that service's dependencies. A pure receiver-owned answer stays on the receiver according to § 6.

Dependency-free FFI transport adapters are the exception: `GemSimulationFormatter` and `PriceAlertFormatter` may be constructed locally because they have no state to substitute. Do not extend that exception to a service, store, client or a type whose behavior can cross on its honest receiver.

A fieldless Core rule object is the second exception. `GemAssetConfigService` and `GemConnectionService` carry no state, no store and no client, so a module-level lazy instance is not a second instance of anything — there is nothing to substitute and nothing to keep in step. Keep them there only while they back top-level extensions on a primitive that neither a composable nor a constructor can reach (`Chain.asset()`, `AssetId.icon()`, `ConnectionStatus.refreshInterval(kind)`); a caller that already has a view model asks its service instead. The same holds for the iOS `.shared` accessors in `Config.swift`: a leaf value model a view builds from a value — a row, a formatted address, a search predicate — has no constructor the composition root controls, so it reads the fieldless object directly. What is never acceptable is a flow parent reaching for one to hand to a child: the parent takes the child's vendor from the factory, the way `ViewModelFactory` vends `ImportWalletTypeViewModel` and `ManageContactViewModel`. The composable reading one still has to `remember` the answer: a Core call in a `@Composable` body runs on every recomposition.

Prefer the platform abstraction (`GemConfirmServiceProtocol` on iOS, `GemConfirmServiceInterface` or a case on Android) wherever a test needs substitution. Mocking the concrete UniFFI object is fragile: any unstubbed generated method can reach a native handle the mock does not have.

## 9. Errors

Use the shared `GemServiceError` for ordinary API, store and service failures. Add a feature error enum in `error.rs` only when the app needs structured domain data to render or branch without parsing a message:

```rust
pub enum GemConfirmError {
    ScanMemoRequired { symbol: String },
    BalanceMissing { asset_id: AssetId },
    Sign { error: GemSignerError, msg: String },
}
```

When a lower-level error has a canonical feature-level mapping, implement `From` once in `error.rs` and use `?`:

```rust
impl From<GemServiceError> for GemConfirmError {
    fn from(error: GemServiceError) -> Self {
        Self::Load { msg: error.to_string() }
    }
}
```

Use `map_err` only when the call site adds context or deliberately selects a non-default category, such as `Record`, or when a named mapper preserves structured `Offline`/`Network` gateway cases.

The app **localizes Core's error directly** — it does not translate it into a parallel app-side enum first. A duplicate taxonomy costs a mapping function, re-derives data Core already carries, and drifts. Classify Core's error where a screen needs to branch; do not re-wrap it.

### An error crosses to the app as a display, not as itself

Cases an error enum distinguishes for control flow are rarely the cases a screen distinguishes for presentation. When the two differ, give the error a `display()` returning a presentation enum, and let every app switch on that:

```rust
#[uniffi::export]
impl GemConfirmError {
    pub fn display(&self) -> GemConfirmErrorDisplay {
        match self {
            Self::InsufficientNetworkFee { asset, requirement } => match requirement {
                Some(requirement) => GemConfirmErrorDisplay::NetworkFeeRequired { .. },
                None => GemConfirmErrorDisplay::NetworkFeeMissing { .. },
            },
            ...
        }
    }
}
```

The collapse is the point. Variants that read the same to a user merge into one display case; a nested error, an optional payload or a `from`/`signer` pair the screen never shows stops reaching the app at all. Without it each app re-derives the same branch, and the two drift on the case nobody checked. Answer per-variant questions the apps would otherwise each answer — whether a case has a detail sheet, say — on the display enum, so the app reads the decision rather than repeating the list.

## 10. Tests

| Layer | What it tests | Where |
|---|---|---|
| Core | the pure rule or intrinsic receiver behavior | owning module, usually `rules.rs`; beside the type for intrinsic behavior |
| iOS | the mapping Core → app types | feature tests, substituting an I/O screen service when needed |
| Android | the wiring — that the case passes Core's answer through | module unit tests, substituting the case or service interface |

Neither app tests a rule that lives in Core. If an app test would fail when a Core rule flips, the rule is in the wrong place or the test is asserting the mock.

A Core test double for a store or a port lives in the owning folder's `testkit.rs` (`#[cfg(test)] pub(crate) mod testkit;`), named after the trait it implements — `MemoryPreferencesStore`, `MemoryWalletStore`, `MemoryConnectionStore`, `TestWalletConnectSigner` — so a service test composes the doubles of every folder it depends on instead of writing one struct that implements six traits. Cross-cutting doubles (`TestAlienProvider`) live in `gemstone/src/testkit.rs`. A double that exists to probe one behavior of one test (a store that counts writes or delays a read) stays inline with that test.

**A test never hand-rolls a double a testkit already ships.** A `struct` in a test module that implements `Client`, `Target` or a store trait is a stand-in nobody else uses: it drifts the moment the real trait grows a method, and it asserts the stand-in rather than the path the app takes. Take the double from the owning crate's `testkit` — enabled through that crate's `testkit` feature under `[dev-dependencies]`, never copied — and drive the real request through it:

```rust
let client = AlgorandClient::new(MockClient::new().with_post_with_headers(|path, body, headers| {
    assert_eq!(path, "/v2/transactions");
    assert_eq!(body, [0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(headers.get(CONTENT_TYPE).map(String::as_str), Some(ContentType::ApplicationXBinary.as_str()));
    Ok(br#"{"txId":"TXID"}"#.to_vec())
}));
```

`MockClient` encodes the body exactly as `ReqwestClient` and `RpcClient` do, so a handler that asserts bytes and headers is asserting the wire. References: [`gem_client::testkit`](../core/crates/gem_client/src/testkit.rs) and `mock_jsonrpc_client` for HTTP, [`gem_hypercore/src/testkit.rs`](../core/crates/gem_hypercore/src/testkit.rs) for a chain client, [`primitives/src/testkit/asset_mock.rs`](../core/crates/primitives/src/testkit/asset_mock.rs) and [`storage/src/testkit/scan_address_mock.rs`](../core/crates/storage/src/testkit/scan_address_mock.rs) for fixtures; call sites in [`gem_algorand/src/rpc/client.rs`](../core/crates/gem_algorand/src/rpc/client.rs) and [`gem_stellar/src/rpc/client.rs`](../core/crates/gem_stellar/src/rpc/client.rs).

**A mock exists once, beside its type.** Values, doubles, and units under test come from the mock in the testkit that owns the type — `Type::mock()` in Core, `static func mock` in the iOS `TestKit`, `mockType()` in Android `testFixtures` — never from a helper written in the test file, even when one test uses it. The test keeps its literal inputs and one-off overrides; a missing shape extends the existing mock. Placement and naming per platform: [Core tests](../core/skills/tests.md#testkit-mocks), [iOS testing](../ios/skills/testing.md#mocks), [Android testing](../android/skills/testing.md#shared-testkit).

### Do not test the same rule twice through a thicker stack

An app test that stands up a real Core service over a real store and then asserts *Core's decision* is a second copy of a Core test, paid for in database setup and simulator time. It fails for the same reasons the Core test does, and it goes stale in a different file.

What is worth an app test at that seam is the **store adapter** — that `GemstoneFooStore` maps Core's trait onto the app's table, with the right columns and the right round-trip. Assert what was written and read back, not which value Core chose to write.

```swift
// double layer — Core's rules.rs already asserts created-vs-imported
try await service.setupWallet(wallet: created.json())
#expect(try store.getBanner(id: "\(created.id.id)_onboarding")?.state == .active)

// worth keeping — the adapter and schema, which Core cannot reach
try store.addBanners([NewBanner(id: id, walletId: walletId, assetId: assetId, event: .stake, state: .active)])
#expect(try store.getBanner(id: id)?.state == .active)
```

```kotlin
// worth keeping — the query the adapter runs, which Core cannot reach
database.bannersDao().addBanners(listOf(warning.copy(id = "other-wallet", walletId = "wallet-2")))
val banners = database.bannersDao().observeAssetBanners("wallet-1", tokenId, assetId).first().map { it.toDTO() }

assertEquals(setOf(BannerEvent.AccountBlockedMultiSignature), banners.map { it.event }.toSet())
```

- **Never mock a dependency-free constructible service** (`GemChainService`, `GemAssetConfigService`, …). Construct the real one. An app test may substitute an I/O screen service to test mapping or state; the returned Core answer is then a stated premise, not a rule assertion.
- **Never fabricate I/O to reach a rule.** An offline provider, in-memory stores and empty rows stood up so a test can touch rules that use none of them is always the wrong answer. Pass the answer in from the caller, or mock the service and state the premise.

A mock's defaults should be the *usual* case. A mock that fails by default becomes a trap the moment another method starts depending on it.

## 11. Landing a change

1. Implement in Core with the rule test.
2. If a UniFFI signature, TypeShare model, `remote_types.yml` entry or mobile integration boundary changed, run `just generate` from the repo root. Internal Core changes that preserve those contracts do not require regeneration. Never generate against half-edited Core.
3. Wire both platforms.
4. **Delete the old path it replaces:** free function or wrapper, app call sites, obsolete mocks, duplicate tests and unused imports. A migration that leaves both paths has not migrated anything.
5. Build both apps when their generated interface or integration changed, then verify the affected suites (see `SERVICES.md` § Verification).
6. Search for the old symbol and review the diff for unused public API, redundant conversions and stale generated files.
7. If the change completes an item tracked in [TODO.md](TODO.md), delete that item's line in the same commit. Commit and publish only when authorized under [Task Workflow](../skills/task-workflow.md).

### When the platforms disagree

Check the documented contract, callers, and tests: a difference may be intentional, or a test may pin a bug. Explain the evidence for the shared behavior before changing it. Ask only when the product decision remains unresolved; a passing test alone does not choose the contract.

## 12. A client's requests are one enum; the client only sends

**REST clients own a request target; JSON-RPC clients own a request enum.** Keep paths and request-specific metadata in the target. Credentials, transport, envelope handling, and pagination belong in the client. The deliberate exceptions are listed below.

The enum is a § 1 rule for a request: inputs in, wire format out, no transport, no secret, no clock. The two references are [`TronGridTarget`](../core/crates/gem_tron/src/rpc/trongrid/target.rs) for REST (`FooTarget` in `rpc/target.rs`) and [`SolanaRpc`](../core/crates/gem_solana/src/jsonrpc.rs) for JSON-RPC (`FooRpc` in `jsonrpc.rs`, method constants in `method.rs`). For a direct GET/POST without shared client work, use [`AptosClient`](../core/crates/gem_aptos/src/rpc/client.rs); do not add a forwarding `send` helper.

Inspect the [target](../core/crates/gem_tron/src/rpc/trongrid/target.rs) and [client](../core/crates/gem_tron/src/rpc/trongrid/client.rs) together; keep examples linked to their implementation instead of maintaining a second code sample here.

Variant fields are named: `GetAccount(String)` does not say what the string is. No path or query string is a `const`; `path()` builds every one. The target implements `gem_client::Target` (`path()`, `headers()` when a request carries one, and `content_type()` when a body is not JSON); the client owns the transport, the credentials, the clock, the signature, the envelope and the pagination loop. A method is `self.client.get(target).await` or `self.client.post(target, &body).await`. For `gem_client::Target`, do not add a body enum or a second dispatch over GET/POST variants to unify these calls; the device client's `gem_jsonrpc::Target` has a separate contract below. A private helper exists only for shared work: credentials (`TronGridClient::send`), a 404 that is a value (`StellarClient::get_or_not_found`), an envelope (TON). A GET-only host needs only `path()`; `GemDeviceApiTarget` is the full shape with `method()`, `body()` and a signed header.

| Case | Shape | Reference |
|---|---|---|
| Path parameter | `format!` in `path()`. A chain or network segment is a field the target reads; a chain-to-slug map is a pure `fn` with its own test | `TronGridTarget`, Blockscout `/{chain_id}/…`, `alchemy_url` |
| Query string | Part of `path()`; a transport only ever sees a path. One or two fixed parameters are a `format!`; more, or any optional one, is a flat `Serialize` struct rendered by `build_path_with_query` (`None` omitted, values encoded, a slice of pairs for a repeated key). A client without a target yet calls `client.get(path).query(&query)`, which renders the same way | `GemApiTarget`, `CoinMarketsQuery`, `mayan::quote_path` |
| Optional parameter | An `Option` field of the query struct, omitted when `None`, never a second variant | `TransactionsQuery { limit, fingerprint: Option<String> }`, `PaymentsQuery { cursor, .. }` |
| Method and body | The method calls `post(target, &body)` with the body it has; a POST variant carries nothing the path does not need. `Client` speaks GET and POST; the device client builds `gem_jsonrpc::Target` with a `method()` for PUT and DELETE. A host that multiplexes on the body (HyperCore `/info`, Cardano GraphQL) has a constant path and a variant per query | `AptosClient::submit_transaction`, `GemDeviceApiTarget` |
| Raw body | `String` for `text/plain` and form-urlencoded, `Vec<u8>` for binary; the content type from the target's `content_type()`, always a `ContentType` variant, never a `Content-Type` string in `headers()`. A body-carrying request declares `application/json` by default, so only a non-JSON body overrides it | Bitcoin `sendtx`, Stellar, Algorand, Aptos BCS, `SendSupportImage` |
| Credentials | `fn headers(&self)` on the client, empty when the key is blank (`Option<String>` decided once at construction), passed as `.headers(self.headers())`; a key the host wants in the query is appended by `send` the same way. Transport default headers are backend-only: `RpcClient` has none | `JupiterClient`, `NearIntentsClient`, Blockscout `apikey` |
| Request header | On the target: API version, idempotency key | TON emulate, Flashnet |
| Signature | A pure `fn` in `auth.rs` over method, path, body, timestamp and nonce; the client reads the clock and merges the result last. A refreshed token sits behind an injected port | `okx::auth::sign`, `GoPlusProvider::sign`, `build_device_auth_header` |
| Envelope | Unwrapped once in `send`; a typed error body through `get_or_error::<_, ErrorResponse>`; a 404 that means "none" is a value (§ 3) | TON `ApiResult`, GoPlus `Response`, THORChain, Stellar `AccountResult` |
| Pagination | The cursor on the variant, the page size a `const`, the loop in the client with a page cap and a repeated-cursor guard; the loop takes a closure that builds the page's variant | `get_transaction_pages`, `AlchemyClient::get_nfts_by_owner` |
| JSON-RPC | `ToJsonRpcRequest` with constants from `method.rs` and typed parameter enums beside it; `batch_request` then `take_all`; the same enum posted to a path when a REST host has an RPC route | `SolanaRpc`, `EthereumRpc`, Chainflip broker |
| Construction | `new(client, key)` with `C: Client` already pointed at the host. Never `ReqwestClient::request` from a client: it bypasses `Client` and never works on the apps | `TronGridClient` |

**Tests.** A client test over `MockClient` or `mock_jsonrpc_client` asserts behaviour the wire shape does not show: an envelope's failure branch, the paths a pagination loop produced, the body and content type of a broadcast, a merged credential header. Do not test `path()` by copying its implementation into the expectation. A wire-contract regression test uses an independently specified request and exercises the real client through its mock transport.

**Deliberate exceptions.** Three things stay on raw `reqwest` because they are not REST clients: the off-chain NFT metadata fetch (an arbitrary HTTPS URL read under a byte cap with redirects off), the image downloader (binary bodies), and the egress node health probe (only the status matters). The OKX client sends the string it signed rather than a target, and the alien reqwest provider is the transport itself.

## 13. Shapes that were tried and reverted

These decisions explain the contracts above; they are not a migration checklist. Follow current subsystem and security contracts when historical examples differ. Repo-wide build and tooling choices are in [§ 14](#14-repo-wide-choices-that-are-not-obvious-from-the-code).

### Ownership before wrappers

Empty services and forwarding-only screen services added dependencies without owning behavior. Intrinsic answers belong on the value; pure rules without an honest receiver belong in the existing rules or mapper module. A real service composes I/O, dependencies, or a cohesive flow. The number of fields in a view model does not justify another service.

Shared child views cannot depend on whichever generated Core service their parent happens to hold. Making generated protocols inherit an app protocol or hiding the service in closures did not solve ownership. The parent computes the answer and passes values to the child. A dependency-free child is the intended result.

When services form a cycle, separate the responsibilities that caused it. Node operations once mixed runtime latency state, user node preferences, and chain configuration. Separating those owners removed both the cycle and app-side fallback logic. Passing one service through another or exposing its internal store preserves the cycle.

### Return decisions and carry whole values

Apps repeatedly drifted when Core returned ingredients for the same calculation: transaction headers and participants, destination visibility, stake actions, reward totals, balance rows, validation errors, and claim destinations. Return the cohesive decision from Core, then let each app format and navigate.

Pass a domain record when it already owns the needed fields. Breaking a delegation into chain, provider, state, and rewards made app callers supply inconsistent constants. Creating subset balance records added a mapper on both apps for each new rule. Reuse the canonical record without exposing unrelated mutable dependencies.

Input parsing must preserve the source's meaning: human input and machine strings need different parsers. Returning a coarse success flag or rounding before checking precision discards information the next layer needs. See [number parsing](#number-parsing-human-input-vs-machine-strings).

### Presentation choices are decisions

Which name a row shows, what sits under it, what trails it, and whether a badge repeats a value already in the title are decisions, not formatting. They belong in the record the screen already receives, as cases the app maps to widgets. Left to the apps they drift silently, because each one looks locally reasonable and nothing compares them.

Core picks the number, its units and the currency it is quoted in; the app renders it in the user's locale. A unit choice ("days and hours above a day, hours and minutes below it"), a conversion factor, and which of several available numbers a row displays are all decisions with one right answer. Formatting is the part that must stay platform: Core's value formatter is not locale-aware, so a Core-formatted amount regresses every locale that groups or separates differently.

A default argument that supplies an app's own answer hides that answer from review. Make the parameter required so every call site names where its answer comes from; the compiler then lists the places that were deciding on their own.

Count the copies before adding one. A rule that looks like two app-side duplicates is often three, with the third inside a Core crate that must agree with the other two. Search the workspace before introducing a shared answer, and make the existing owner the one that answers.

### State and lifecycle have an owner

Screen services are constructed with the screen. Putting them in launch-time environment entries built dependencies for screens the user never opened. A screen-scoped service may retain state that belongs to that screen, such as confirm preload and simulation; this does not justify a global cache or singleton mutable state.

Moving a store behind Core must preserve observable write order and failure behavior. Price alerts previously wrote locally before pushing to the API; reversing those steps delayed the UI and lost the local update on network failure. A refactor must preserve that behavior unless the task deliberately changes it.

Wallet creation, import, rename, deletion, and switching have lifecycle effects beyond a database write. Keep those effects in the owning Core flow. Splitting default balance insertion and asset refresh across two app observers created a race; the setup flow must complete its own work. The session-change reaction owns post-import setup instead of each import entry point.

Operations on the current wallet read the Core session through their owning service. Passing a nullable wallet through every app call site caused silent early returns and inconsistent account selection. Preserve explicit wallet identifiers for operations that intentionally target a different wallet; do not turn every operation into an implicit current-session operation.

### Async execution and SDK boundaries

Core futures progress while awaited. Work that must outlive a service call needs an explicit platform execution port, such as transaction tracking started after synchronization. Core decides what to track; the app supplies a lifetime and executor. Do not assume creating a future starts background work.

The app translates SDK events and forwards them; Core owns shared authorization, chain/account selection, routing, and replies. WalletConnect drifted when both apps reimplemented those decisions. Platform ports remain appropriate for signing, secure storage, authentication, observation, and navigation under the current security contract; removing a dependency must not remove an auth gate.

### Transfers and signed conventions

Apps should carry Core-built transfer and action records instead of reconstructing them between screens. Stake and perpetual flows accumulated app-specific switches for accounts, recipient fields, direction, max flags, and amounts. Core owns the conversion from a domain action to confirm data; navigation preserves that result.

The signer's convention must be explicit and tested. HyperCore reduce orders interpret direction as the position direction; one app flipped it and built the wrong reduce-only side. Compare the signer and its vectors before resolving an app disagreement. Simplifying a record must preserve every transaction-critical input.

### FFI has a maintenance cost

An exported method creates generated interfaces, wrappers, and mock requirements even when no app uses it. After replacing a path, check callers on both apps, including extension wrappers and implicit receiver calls. Match the receiver when methods share a name. Move methods used only inside Core to a plain `impl`; delete unused ones. Platform ports implemented by the apps are different: Core is their caller.

Types and conversions may become redundant after un-exporting a method. Keep `Gem` for the FFI surface and reuse the underlying domain type internally when the wrapper carries no distinct meaning. Do not expose an API solely so a mock can reconstruct the production result.

Typed atomic quantities must remain typed across bindings. A big-integer Core type still transported as an app string leaves parsing, defaults, and possible truncation in every consumer. TypeShare and UniFFI have different capabilities; use the existing supported bridge rather than adding a parallel app model or copying product policy.

### Tests follow the rule's owner

Move the behavioral test with the rule. An app test that recreates a Core decision in its mock can pass while Core is wrong. State the mock's answer as a premise and test the app's response; test the decision in Core. Test app store adapters for their mapping and persistence contract instead of repeating Core's business rule through a database.

Use a real dependency-free constructible service. For an I/O service, reuse the owning testkit double. Do not build fake networks, unrelated stores, or synchronization instrumentation just to reach a pure rule. Concurrency and ordering tests are appropriate when that ordering is itself the contract, not merely because the implementation uses concurrent calls.

### Chain families remain families

A family member keeps the family `ChainType`. Giving an EVM chain its own type forced exhaustive switches, wallet namespaces, and fee behavior to treat it as a new family. Use `ChainConfig` or the chain crate for actual per-chain differences. Follow the [New Chain Checklist](../core/skills/new-chain-checklist.md) for implementation.

## 14. Repo-wide choices that are not obvious from the code

Understand the rationale before changing one of these. Code-style exceptions belong in the platform skills, not here.

### Gemstone is bundled locally

Gemstone (the Rust-to-mobile bridge) is built and bundled from source rather than fetched as a prebuilt package. This ensures the mobile apps always link against the exact Core revision in the repo and avoids version drift between Core logic and mobile bindings.

### TypeShare + UniFFI for code generation

TypeShare generates shared model types; UniFFI generates FFI bindings. Both run from `just generate`. Two tools are used because TypeShare handles pure data models efficiently while UniFFI handles the full FFI bridge (functions, callbacks, async). Do not consolidate them.

### Android distribution channels come from one matrix

`android/gradle/channels.gradle.kts` defines every channel (google, universal, huawei, solana, samsung, emerald, fdroid) and what each one links: the push module (FCM or stub), the review module (Google or stub), the WalletConnect implementation (Reown or no-op), ProGuard rules, update URL, and ABIs. Channels exist to satisfy different store and partnership constraints, so do not remove one or assume Google-only distribution.

The active channel is resolved once per Gradle invocation by `selectChannel()`: `-Pchannel=<name>` first, then inference from an `assemble<Channel>`/`bundle<Channel>` task name, then `google`. `fdroid` is the only channel that drops Firebase, Google services, and real WalletConnect, and it is always built alone with `-Pchannel=fdroid` (`android/reproducible/fdroid/build.sh`). Add or change a channel by editing the matrix only. Do not reintroduce `System.getenv` feature reads or hand-written product flavors: the previous dual selector had to be hand-synced, and building the F-Droid flavor without the environment flag silently produced an APK with Firebase and WalletConnect linked, which the F-Droid scanner rejects.

### Rust build cache: kache locally, sccache in CI

Developer machines use kache as the `rustc` wrapper (see `core/skills/setup.md`); CI keeps sccache with the GitHub Actions cache backend (`RUSTC_WRAPPER: sccache` in the workflows, `mozilla-actions/sccache-action` in `.github/actions/setup-rust-ci`). kache dedups content-addressed artifacts across worktrees and clean rebuilds better than sccache does, but CI's cache is independent of any developer machine and kache has no shared remote configured for this repository. The migration was deliberately local-only so CI caching could not regress; switching CI needs a remote store (S3 bucket plus secrets) first. Do not change one side while touching the other.

### Core source lives in this repository

`core/` is tracked source in this repository, not a Git submodule. Changes to Core and the mobile apps should land together when shared behavior, generated models, or bindings need to stay aligned.

### Number parsing: human input vs machine strings

Amount strings come from two sources that must be parsed differently. Confusing them silently corrupts amounts on locales that group thousands with a dot (de, it, es, nl, pt-BR, da), where `"1.234"` means 1234, not 1.234. Pick the parser by the source of the string, never by convenience.

- **Human input** (text a person typed into a field) is parsed by Core. `NumberInput::normalize` and `NumberInput::value` (`core/crates/number_formatter/src/number_input.rs`) own the separator, grouping, leading-zero and Unicode-digit rules and are exported as `normalize_number_input` / `parse_number_input`. The apps pass the text and the device's decimal separator and nothing else: iOS `NumberInput.plain/.double/.value` (`ios/Packages/GemstonePrimitives/Sources/NumberInput.swift`), Android `String.plainInputNumber()` / `parseInputNumber()` / `parseInputValueOrNull(decimals:)` (`android/gemcore/src/main/kotlin/com/gemwallet/android/math/NumberParser.kt`). Never read typed text with `Decimal(string:)`, `Double(_:)` or `BigDecimal(_)`: those miss grouping separators and non-Latin digits, so an Arabic or Persian keyboard reads as no amount at all and `"1.234"` in a dot-grouping locale reads 1000x too low. Text with no digit normalizes to an empty string, which every caller treats as no amount.
- **Machine strings** (QR/payment-link amounts, API/exchange payloads, anything the app did not get from a keyboard) never reach the input parser: Core decodes them and hands the apps typed values. If one has to be read app-side, parse it locale-independently — a machine string always uses `.` as the decimal point.
- **Amounts Core formats for notifications** (`number_formatter::ValueFormatter`, `ValueStyle::Auto`, used by the daemon pusher, the staking rewards notifier and in-app notifications) keep two decimals above one and four significant digits below it, dust included: a push title has no room for `0.000040036032429186 ETH` (issue #1155), so it reads `0.00004003 ETH`. The app list formatters keep full precision for dust on purpose; that is a screen with room, not a title.

### iOS localization compiles to String Catalogs, accessors are generated in Core

The localization generator writes one `.xcstrings` String Catalog per table (app, InfoPlist, widget) instead of per-language `.lproj/*.strings` files, and generates the typed `Localized.swift`/`WidgetLocalized.swift` accessors itself. SwiftGen is not used. Two constraints drove this:

- Xcode 26 can generate catalog symbols natively, but they are internal to the owning target; `Localization` is a shared SPM package consumed across the app, so the public accessor surface must be generated by our tooling.
- The historical dotted iOS keys (`common.cancel`) cannot be derived from the underscore Fluent IDs (`common_maximum_value` vs `common.maximum_value`); the committed `Localizable.xcstrings` is the source of that mapping, which the generator reads back on every run. Do not delete the catalogs and regenerate from scratch — the dotted key mapping (and with it the `Localized.*` API) would be lost.
