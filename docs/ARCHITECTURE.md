# Feature architecture

Use the relevant sections when changing shared feature behavior or its app integration. These are current ownership contracts, not a mandate to migrate adjacent code. Resolve disagreements with source using [guidance precedence](../AGENTS.md#using-the-guidance).

## Find the Relevant Contract and Example

Read the contract and the named implementation, then the actual owner and callers being changed. Examples illustrate the named responsibility; snippets with `...` are abbreviated, not complete implementations. Check the current signature before copying one. An open TODO identifies existing debt, not a second endorsed pattern. Preserve the worked examples when updating guidance, correcting stale details in place.

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
| Async result freshness | [Session contract](#a-screen-whose-state-changes-is-a-session), [Performance](PERFORMANCE.md) | [Fiat session](../core/gemstone/src/services/fiat/session.rs), [retained confirmation](../core/gemstone/src/services/confirm/confirmation.rs) |
| Atomic writes and retries | [Store contract](#atomic-changes-concurrent-publication-and-query-contracts), [command outcomes](#a-command-names-its-commit-and-recovery-behavior) | [Native balance update](../ios/Packages/Store/Sources/Stores/BalanceStore.swift), [Android transaction runner](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/stores/PerpetualStore.kt); `BalanceStoreTests` and `BalancesDaoTest` are the paired adapter tests (MIG6) |
| Choose and complete a migration item | [Working one item](TODO.md#working-one-item), [screen coverage](TODO.md#screen-coverage-and-existing-infrastructure) | [Service map](#screen-services), [closing checks](../skills/quality-checks.md#closing-matrix) |

For a service change, follow the price-alert example through [Core](#service-example-price-alerts), [store adapters](#store-adapter-example-price-alerts), [construction](#construction-example-price-alerts) and [screen calls](#direct-service-calls-and-observed-reads). Use the [service map](#service-map) to find existing owners and callers. Read [§ 13](#13-shapes-that-were-tried-and-reverted) only for rejected-design rationale; subsystem contracts remain in their own documents.

## The one rule

**Core decides. The apps ask, render, and store.**

Shared business decisions belong in Core: filters, thresholds, gates, ordering and the meaning of displayed state. Platform rendering and [intentional feature differences](#intentional-platform-differences-and-compatibility) remain native.

Everything else is platform work: rendering, navigation, observation, secure storage, keychain and biometrics, and the SQL that stores rows.

Gemstone is the mobile entry point to shared business behavior and reuses the existing Core crates. A command calls the owning service; an observed database value feeds a Core projection; a user event updates a pure session and its derived state. Full migration does not require a new session for every screen, an FFI call for every label, or replacing Room and GRDB. Use the smallest existing owner and delete the app path it replaces.

Fix ownership violations in the path being changed under the shared [engineering principles](../skills/engineering-principles.md#fix-causes-not-symptoms). Record larger unrelated migrations in [TODO.md](TODO.md); these contracts do not authorize broad adjacent refactors.

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
pub fn selectable_fee_assets(assets: Vec<Asset>, balances: Vec<GemAssetBalance>, prices: Vec<AssetPrice>) -> Vec<GemFeeAsset> {
    balances
        .into_iter()
        .filter(|balance| balance.available > num_bigint::BigUint::from(0u32))
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

    let balances = vec![
        GemAssetBalance { asset_id: funded.id.clone(), ..GemAssetBalance::mock_with_available(1) },
        GemAssetBalance { asset_id: empty.id.clone(), ..GemAssetBalance::mock_with_available(0) },
    ];

    let selectable = selectable_fee_assets(vec![funded.clone(), empty.clone()], balances, vec![]);

    assert_eq!(selectable.iter().map(|fee| fee.asset.id.clone()).collect::<Vec<_>>(), vec![funded.id]);
}
```

The module is `pub(crate) mod rules`, so a `pub fn` inside it is still crate-private. Verify changed domain rules using the shared [test-intent rule](../skills/engineering-principles.md#tests).

## 2. The service orchestrates; it owns its store and depends on services

A service composes rules with I/O. It may hold its own feature store and narrow platform ports. For another domain, it depends on that domain's service, never its store — a store belongs to one owner, and reaching around that owner creates a second read path it cannot see.

Inspect the dependency graph before replacing a foreign-domain store with its service. Never introduce an `Arc` cycle to satisfy this rule; split out a narrow query service, invert the dependency, or redesign the ownership boundary first. `GemWalletService` already depends on `GemWalletSessionService`, so making the session service depend back on the wallet service would be worse than the store debt it replaces.

`GemWalletSessionService` is the narrow query service for wallets: it answers `get_wallets`, `get_wallet` and `require_wallet` and depends on nothing but its two stores, so balance, subscription, device and asset discovery read wallets through it rather than holding `GemWalletStore`. Three holdings are kept, each because the owner already depends on the holder: `GemWalletSessionService` and `GemAvatarService` hold `GemWalletStore` (`GemWalletService` holds both), and `GemStreamSubscriptionService` holds `GemBalanceStore` (`GemBalanceService` holds the subscription service). A kept holding is a named exception, not a precedent — record why it is kept where it is declared.

A store that is a platform port rather than a domain — `GemFileStore`, `GemWalletSessionStore`, `GemDevicePlatform`, `GemNotificationPermissions` — is not a foreign-domain store and several services may hold it.

```rust
// services/confirm/mod.rs
#[derive(uniffi::Object)]
pub struct GemConfirmService {
    gateway: Arc<GemGateway>,
    balance: Arc<GemBalanceService>,
    price: Arc<GemPriceService>,
    assets: Arc<GemAssetsService>,
    ...
}

impl GemConfirmService {
    pub async fn fee_assets(&self, wallet_id: WalletId, chain: Chain) -> Result<Vec<GemFeeAsset>, GemConfirmError> {
        let fee_asset_ids = chain_fee_asset_ids(chain);
        if fee_asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        let (assets, balances, prices) = futures::join!(
            self.assets.assets(fee_asset_ids.clone()),
            self.balance.balances(wallet_id, fee_asset_ids.clone()),
            self.price.prices(fee_asset_ids),
        );
        Ok(rules::selectable_fee_assets(assets?, balances?, prices?))
    }
}
```

`GemConfirmError` implements `From<GemServiceError>` once in `error.rs` (see [§ 9](#9-errors)), so the three reads use `?` and keep `Cancelled` and `Offline` instead of folding every failure into `Load`.

The method is thin: gather inputs, call the rule, return. It stays on the plain `impl` because the confirm screen reads fees through `GemConfirmTransferService`. Product or domain-decision branching belongs in `rules.rs`; I/O sequencing, error propagation and empty-work short circuits may remain in the service.

### Secure, fast, simple, powerful

Judge a change against those four. **Secure:** a secret, a signature, and a confirmation stay explicit and fail closed. **Fast:** the user waits only for work that must happen in order. **Simple:** one owner, on the path that already exists. **Powerful:** the shared rule covers the next chain, not a one-off beside it.

Independent reads are the usual way to stay fast. Two reads that do not use each other's result run together. Awaiting the first before starting the second only adds a round trip. Use `futures::try_join!` when each call returns `Result`, and `futures::join!` when a caller still matches a non-error outcome. The XRP preload is the small case: the sender account and whether the destination exists are separate calls, so [`get_transaction_preload`](../core/crates/gem_xrp/src/provider/preload.rs) starts both:

```rust
let (sender, destination_exists) = futures::try_join!(
    self.get_account_info_full(&input.sender_address),
    self.account_exists(destination),
)?;
```

A later step that needs an earlier result stays sequential. That ordering is the [staged load](#a-staged-load-names-what-each-stage-waits-for). Do not overlap a write, a signature, or a step that must fail closed before the next one starts.

**Sync reads require an already-held value.** `GemWalletSessionStore.get_current_wallet_id` and preferences are synchronous. `GemWalletStore.get_wallet` and `get_wallets` stay async because Room must read off main, even though GRDB can read synchronously. Await database reads through their owner; reuse observed values for rendering. Never add a blocking DAO query or `runBlocking` to make the platforms look alike.

### Service example: price alerts

[`GemPriceAlertService`](../core/gemstone/src/services/price_alert/mod.rs) shows a service combining an API client, preferences, its own store and a platform permission port:

```rust
#[derive(uniffi::Object)]
pub struct GemPriceAlertService {
    api: Arc<GemDeviceApiClient>,
    preferences: Arc<GemPreferencesService>,
    store: Arc<dyn GemPriceAlertStore>,
    permissions: Arc<dyn GemNotificationPermissions>,
}
```

Its refresh reads both sources, asks the rule for a diff, then commits that diff:

```rust
pub async fn sync(&self, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
    let remote = self.api.client.get_price_alerts(...).await.map_err(GemApiError::from)?;
    let local = self.store.get_price_alerts(asset_id).await?;
    let changes = rules::reconcile(local, remote);
    if changes.delete_ids.is_empty() && changes.alerts.is_empty() {
        return Ok(());
    }
    self.store.update_price_alerts(changes.alerts, changes.delete_ids).await
}
```

Continue with the [store adapters](#store-adapter-example-price-alerts), [construction](#construction-example-price-alerts) and [screen calls](#direct-service-calls-and-observed-reads) below.

Node selection illustrates this boundary: `GemChainSettingsService.check_node` owns URL validation, the network-id check and node status. `GemGateway`, `GemSwapper` and `GemSimulationService` take `GemNodeService` for the selected node. Gateway preferences hold gateway state such as HyperCore agent data, never a second node selection.

### No trivial exports

An export earns its place by making a decision. A function that looks up a constant for a variant, or wraps a value the app already holds so the app can ask for it back, is not a decision — it is a second spelling of a `match` the app will write anyway, plus an FFI crossing per call. An icon name per row key and a title per enum case are the two usual forms; both belong on the enum as data the screen record already carries, resolved once in the app's [mapper](#one-mapper-per-app-names-every-core-key-it-renders).

The test is what the caller could not have worked out: if the answer depends only on the variant, the variant is the answer and the app maps it. If it depends on state, configuration, a chain rule or several values at once, it is a decision and Core owns it.

The one exception is a [projection](#a-row-is-projected-from-its-value-never-fetched-from-a-service).

**A fixed value is a generated constant, not an export.** A debounce, a timeout, a limit or a fixed list of options takes no input, so asking Core for it is a crossing per read. It lives in [`constants.rs`](../core/gemstone/src/constants.rs), which Core's own code reads, and `just generate-models` writes it to both apps as `GemConstants`. A constant is a plain value: a number, a string, a duration (Swift `Duration`, Kotlin `kotlin.time.Duration`) or a list of enum cases (a TypeShare enum as the app's own type). A record is not a constant; its fields are, and whoever needs the record builds it. A generator test fails when a checked-in constants file is stale. A value that depends on an input (a chain, a locale, a wallet) or is built by a formatter stays a function.

**An export also loses its place when the last app caller goes**, because the generated interface, the wrapper and the mock requirement stay behind on both apps. `just check-ffi` reports every exported function and method of an exported `impl` that no app calls outside its tests, mocks and previews; a function Core still uses keeps its body and loses only the export, one nothing uses is deleted. The composition services are where this happens most, and `GemExplorerService` is the finished shape: only its constructor is exported, because the composition root builds the object, while `get_address_url` and `get_transaction_link` sit in a plain `impl` block that the screen services call — every screen reads its explorer link through [its own screen service](#composition-services-are-reached-through-the-screen-service).

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

**A WalletConnect simulation fails open too**, and was weighed as its own decision (D73, 2026-09-22). `GemSimulationService::simulate_send_transaction` returns an empty `SimulationResult` when no provider can answer, so a review of a request whose simulation failed looks the same as a review of a request that changes nothing. The alternative considered was carrying a "simulation unavailable" warning through the existing warning rows; it was not taken, so an outage never blocks signing and never adds a row. The validation warnings computed before the simulation are unaffected either way.

Before moving any stage earlier, audit what the stage actually does on every provider — "load" is not a promise of read-only. Every chain family but one answers `get_transaction_load` with RPC estimates and local arithmetic. HyperCore is the exception: its swap and perpetual path creates and persists an agent keypair in the secure store and writes approval-cache preferences, so starting it before the scan verdict would provision durable credentials for transactions the scanner then rejects. That single provider is why the gate stays where it is, and an implementation commit has to move the provisioning out of the load first — the overlap is safe for the read-only families only once each one is pinned as read-only.

The order itself is tested, not assumed: a malicious verdict must leave the chain unasked, and a clean verdict must let the load through.

## 3. Return one record that answers the whole question

A screen that needs five things should make one call, not five. Core assembles the answer.

```rust
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmLoad {
    pub sender: GemAccount,
    pub fee_asset: Asset,
    pub metadata: GemConfirmMetadata,
    pub fee_assets: Vec<GemFeeAsset>,
    pub simulation: GemConfirmSimulationState,
    pub address_name: Option<AddressName>,
    pub fee: Option<GemConfirmFee>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmFee {
    pub value: GemBigInt,
    pub additional_fees: Vec<GemFeeOptionItem>,
    pub selected_priority: FeePriority,
    pub amount: GemTransferAmountResult,
}
```

Three things these records get right:

**The record carries what the screen shows, not what Core signs with.** The signing input (gas, chain metadata such as UTXO lists, the raw simulation) stays in the Core object that owns the screen; the app gets display values, and asks that object for anything derived from the signing input, such as fee rate rows, instead of sending a record back.

**A recoverable failure is a value, not an error.** An unaffordable transfer still has a fee, fee rates and a simulation to render, so the fee's amount is an enum rather than collapsing the whole call:

```rust
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransferAmountResult {
    Amount { amount: GemTransferAmount },
    Error { error: GemConfirmError },
}
```

The error is the same `GemConfirmError` every other confirm failure uses, carrying the `Asset` it names and a `GemBalanceRequirement` (required, available, shortfall), so the app renders it the same way whether it came from the load or the send.

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
pub struct GemAssetRowStyle {
    pub title: GemAssetTitleStyle,
    pub shows_symbol: bool,
    pub subtitle: GemAssetSubtitleStyle,
    pub trailing: GemAssetTrailingStyle,
}
```

The screen asks its service for the style once and passes it down, so wallet, search, select and network-asset lists all render from one answer and a change to what a row means is one edit in Core. The style names the choices; a second projection resolves them against the asset, so neither app reads a title rule:

```rust
pub fn asset_row_text(asset: &Asset, style: GemAssetRowStyle) -> GemAssetRowText {
    let chain_asset = ChainAsset::from_chain(asset.chain());
    let title = match style.title {
        GemAssetTitleStyle::Asset => asset.name.clone(),
        GemAssetTitleStyle::CanonicalAsset => match asset.id.is_native() {
            true => chain_asset.asset.name.clone(),
            false => asset.name.clone(),
        },
        GemAssetTitleStyle::Network => chain_asset.network_name.clone(),
    };
    GemAssetRowText {
        symbol: (style.shows_symbol && title != asset.symbol).then(|| asset.symbol.clone()),
        network: (style.subtitle == GemAssetSubtitleStyle::Network && !asset.id.is_native()).then(|| chain_asset.network_name.clone()),
        title,
    }
}
```

The row model holds the record and reads it; `name` is `text.title` on both apps, `symbol` is `text.symbol`, and a network subtitle is `text.network` or nothing. Before this, a network title read "TON" on iOS and "Gram" on Android, and the symbol was hidden against the row's title on one app and the raw asset name on the other.

`GemValidatorRow`, `GemFiatQuoteRow`, `GemWalletRow` and `GemBalanceRow` are the same shape for their lists. A session is for a screen the user drives, with events and a derived view state; a projection of one value that answers the same way every time is a row. The thing to look for in a row model is a decision the record could carry: if both apps compute it, it belongs in the record, not in two view models.

### The record carries the finished value, not the ingredients

An app that receives a number and a flag has to decide what to print, and two apps decide differently: which locale groups a block height, whether a missing chain id is a dash or an empty cell, what a boolean sync flag looks like. None of that is a platform capability; it is the same answer computed twice.

So the row carries the value and names the outcome:

```rust
pub enum GemNodeCheckRow {
    ChainId { value: String },
    InSync { state: GemNodeSyncState },
    LatestBlock { value: GemFormattedNumber },
    Latency { milliseconds: u32 },
}
```

`ChainId` arrives printable, with the placeholder already substituted, and `LatestBlock` arrives as a [`GemFormattedNumber`](#a-number-crosses-as-a-value-and-a-style-never-as-a-string-or-a-callback) that each app renders through its one `text()`, so neither app carries a formatter or a placeholder for them. `InSync` carries a named state rather than a `bool`, because a boolean forces the app to decide what the state means. `Latency` stays a number because its text is a localized template with a number in it, and that template lives in the app's catalog.

The app is then a map over the rows, and the only thing it decides is which widget draws each one:

```swift
case let .ready(check): .data(check.rows().map { ListItemField(title: $0.title, value: $0.text) })
```

```kotlin
is GemAddNodePhase.Ready -> "" to phase.check.rows().map { it.uiModel(context) }
```

iOS draws the sync state as an emoji in the value column and Android as a tinted icon in the accessory slot. Which widget draws a case is layout; neither app decides what the state means.

Two things still map per platform, and only two: the localized label for each case, and the glyph or colour for each named outcome. Anything else in a row model — a formatter, a placeholder, a ternary over a flag — is a decision that belongs in the record.

The same holds for a value an SDK sends on the wire. Rejecting a WalletConnect proposal is one Core answer: `session_rejection` returns the finished rejection — the named reason, the CAIP-25 code, the message the dApp receives and whether the session is deleted — and each app maps only its own SDK's error type onto the named reason, which is the one thing Core cannot see.

```swift
let rejection = service.sessionRejection(reason: GemWalletConnectRejectionReason(from: error))
try await WalletKit.instance.rejectSession(proposalId: proposal.id, reason: RejectionReason(rejection.reason))
if rejection.deletesSession {
    try await service.deleteSession(sessionId: proposal.pairingTopic)
}
```

```kotlin
val rejection = walletConnectService.sessionRejection(reason)
walletConnectClient.rejectSession(proposal = proposal, rejection = rejection, onSuccess = {
    if (rejection.deletesSession) {
        scope.launch { walletConnectService.deleteSession(proposal.pairingTopic) }
    }
})
```

### A plain list is Core sections of one shared row, rendered by one builder per app

A row that shows a title with a value, an amount, a duration, a toned label, a date, a network, a link, an icon, an address to copy, a loading placeholder or an error is the same row on every screen. It crosses once, as [`models/list.rs`](../core/gemstone/src/models/list.rs):

```rust
pub enum GemListRow {
    Notice { title: GemListRowTitle, message: Option<GemLocalizedText>, kind: GemNoticeKind },
    Text { title: GemListRowTitle, value: String },
    Amount { title: GemListRowTitle, amount: GemFormattedNumber, info: Option<GemInfoTopic> },
    Duration { title: GemListRowTitle, parts: Vec<GemDurationPart>, info: Option<GemInfoTopic> },
    Label { title: GemListRowTitle, text: GemLocalizedText, tone: GemValueTone, info: Option<GemInfoTopic>, progress: bool },
    Date { title: GemListRowTitle, date: DateTime<Utc> },
    Network { title: GemListRowTitle, chain: Chain, name: String },
    Link { title: GemListRowTitle, value: Option<String>, icon: GemListRowIcon },
    Url { title: GemListRowTitle, value: Option<String>, icon: GemListRowIcon, url: String, target: GemUrlTarget },
    Toggle { title: GemListRowTitle, value: Option<String>, icon: GemListRowIcon, is_on: bool },
    Picker { title: GemListRowTitle, value: String, icon: GemListRowIcon },
    Icon { chain: Chain },
    Address { address: String, copy: GemCopy },
    Explorer { name: String, url: String },
    Loading,
    Error { error: GemServiceError },
    ...
}

pub struct GemListSection {
    pub title: GemListSectionTitle,
    pub footer: GemListSectionFooter,
    pub rows: Vec<GemListRow>,
}
```

Four things make this work, and all four are load-bearing:

1. **A screen's record returns `Vec<GemListSection>`** and declares no section or row type of its own. `GemAddressDetails::sections()` is the reference; a screen with no section header uses `GemListSectionTitle::None`.
2. **The row carries the finished value.** `Text` carries printed text, `Amount` carries a `GemFormattedNumber` each app renders through its existing formatter, and `Address` carries the [copy model](#one-copy-model-for-every-address-phrase-and-key) with the shortened display already computed. A row that owns its title answers it in Core (`GemBalanceRow::title()`).
3. **Titles are two shared enums**, `GemListRowTitle` and `GemListSectionTitle`, resolved in the one shared localization file per app — iOS `PrimitivesComponents/Extensions/Gemstone+Localized.swift`, Android `ui/localization/GemstoneText.kt`. A new title is a case there, never a per-feature mapper. A title that interpolates a value stays app-side (`Explorer` carries the explorer's name and the app composes "View on …").
4. **One builder renders it, so a screen adds no rendering.** iOS `GemListRowView(row:)` in `PrimitivesComponents` and Android `GemListRowView(row:, listPosition:)` plus `LazyListScope.gemListSections(sections:)` in `:ui` take the Core row and own everything it needs — the list-row container, the copy and its toast, opening the link, the loading spinner, and the info button. A row that has an explanation carries a `GemInfoTopic`, and `GemInfoTopic::sheet(platform)` composes its sheet — the title and description keys with their finished arguments, the image, and the action (a docs link, buy, acquire or continue) — so each app maps one `GemInfoSheet` and wires only the action closure. iOS reports the topic through `GemListRowView(row:onInfo:)` for the screen to present, and Android opens the `InfoSheetEntity` itself. A confirm error composes its sheet through `GemConfirmErrorInfo::sheet(platform)`, which keeps the topic that every row carries small. A screen is then:

```swift
ListSectionView(provider: model) { row in
    GemListRowView(row: row)
}
```

```kotlin
LazyColumn { gemListSections(sections) }
```

Rich rows stay outside: the transaction header, swap progress, asset, wallet and validator rows have their own layout on both apps and keep their own records ([a row that a screen only ever draws one way](#a-row-that-a-screen-only-ever-draws-one-way-keeps-its-shape-app-side)). The test is whether the row is a title with a value — if it is, it is a `GemListRow`. Adding a per-screen row enum, a per-feature title mapper or a second `switch` over the row in a scene is the regression this replaces.

### Three row families, and which one a list belongs to

Rows divide into three families. Getting the family right is most of the design work for a list.

**1. Plain rows — one shared `GemListRow`.** A title from a fixed vocabulary plus a value, an amount, a link, a web page, a switch, a picker, an icon, an address, a loading placeholder or an error. One type, one renderer per app, no per-screen code. This is the default; reach for it first.

A row that acts carries what the act needs, and Core decides it: `Url` says whether the page opens in the app or hands off to the system (`GemUrlTarget`), `Link` names an in-app destination, `Toggle` carries its state and `Picker` the selected value. What the app keeps is two maps keyed by the shared `GemListRowTitle` — one to a destination, one to an action — plus the callbacks the builder takes (`onSelect`, `onToggle`). A screen never re-titles or re-icons a row; a value only the device knows (an app version, a wallet count, the biometry name, a localized period) is passed *into* Core, which decides whether the row exists and what it holds.

**2. Shared rich rows — one record per family, reused by every list that draws that family.** An asset row and a transaction row are heavy (an icon with a badge, a title that may be the asset or its network, a subtitle that may be a price, a network name or a counterparty, a two-line trailing value) and they appear on many screens, so each is one record that all of those screens read:

| Family | Record | Lists that read it |
|---|---|---|
| Asset | [`GemAssetItemRow`](../core/gemstone/src/services/assets/model.rs) | wallet, network assets, search, select asset, buy, price alerts, perpetual markets and positions, fee assets |
| Transaction | [`GemTransactionRow`](../core/gemstone/src/services/transactions/model.rs) | activity, asset details, perpetual position |
| Wallet | [`GemWalletRow`](../core/gemstone/src/services/wallet/model.rs) | wallets list, wallet detail, confirm sender |
| Validator | [`GemValidatorRow`](../core/gemstone/src/services/stake/model.rs) | stake, earn, delegation |
| Balance | [`GemBalanceRow`](../core/gemstone/src/services/balance/model.rs) | asset details, address details |

A shared rich row lives with the service that owns its domain, not in `models/list.rs`, because its payload is that domain's type (`Asset`, `TransactionId`). The rich record does not contain a `GemListRow`: a plain row's title is a case of `GemListRowTitle`, while an asset or transaction row's title is data the row carries. A plain section can still hold one rich row, which is what `GemListRow::Wallet` does on confirm and sign-message. What a rich row reuses is the smaller shared pieces — `GemFormattedNumber` for an amount, `GemCopy` for a copyable value, `GemListRowTitle` for a labelled sub-field, `GemLoadState` for its list's state.

A `Gem…Row` record carries **one row's data**. Which fields a shared row shows on a given screen is a different decision and a different type, named `…Style` and never `Row`: [`GemAssetRowStyle`](../core/gemstone/src/services/assets/model.rs) says whether the asset row titles itself with the asset, its canonical name or its network, whether it repeats the symbol, and what its subtitle and trailing hold. The app keeps the store record (`AssetData` on iOS, `AssetInfo` on Android) and projects it through `asset_list_rows` with the style of the flow it is in (`GemSelectAssetFlow.row_style`), or `wallet_asset_rows` for the wallet lists, which applies the wallet style in Core. Both return [`GemAssetItemRow`](../core/gemstone/src/services/assets/model.rs): the icon, a title with an extra, a subtitle with an extra and a trailing `Value`, `Toggle`, `Copy` or `None`, each text a `GemLocalizedText` with its `GemValueTone`, plus `masks_balance` for the rows the balance privacy switch hides. Price alerts, perpetual markets, positions and orders being opened, and fee assets return the same record next to their identity, so each app draws every asset-like list with one renderer. Pin and the account address stay on the app record; the app never re-declares the style enums.

**3. Per-screen rich rows — one screen, one layout.** The transaction header, swap progress and the confirm recipient row are drawn one way on one screen, so the record stays with that screen ([a row that a screen only ever draws one way](#a-row-that-a-screen-only-ever-draws-one-way-keeps-its-shape-app-side)). If a second screen starts drawing it, it has become family 2 and moves.

The questions, in order: is the row a title with a value (family 1)? Does more than one list draw this shape (family 2)? Only then does it stay with its screen (family 3). A per-screen enum for a family-1 row, or a second copy of a family-2 record, is the regression to watch for.

### A screen's load state is one Core state, and a failed refresh keeps what is shown

The apps agree on what a screen in flight looks like: `StateViewType` on iOS and `StateViewType` in `ui-models` on Android both read `noData | loading | data | error`. When each case applies — whether a pull-to-refresh that fails wipes the rows the user is reading or leaves them — is a product decision, so it crosses as Core state.

[`models/state.rs`](../core/gemstone/src/models/state.rs) holds the shared pair. `GemLoadState` is the one enum every screen uses, with the same four cases as the apps' own:

```rust
pub enum GemLoadState {
    NoData,
    Loading,
    Data,
    Error { error: GemServiceError },
}
```

UniFFI has no generics, so the state cannot carry the payload across the FFI the way `StateViewType<T>` does; the record holds the state beside the value it loaded. `GemLoad<T>` is the Rust-side generic that keeps the two together and owns the transition, so no feature writes that rule again:

```rust
pub struct GemLoad<T> {
    pub state: GemLoadState,
    pub value: T,
}

impl<T: Clone + Default> GemLoad<T> {
    pub fn loading() -> Self
    pub fn data(&self, value: Result<T, GemServiceError>) -> Self
}
```

`data` is the decision: a fetch that succeeds replaces the value, a fetch that fails keeps a value already on screen, and only a screen with nothing to keep shows the error. A screen therefore hands its record back for the next load — `refresh(details)`, not `refresh(chain, address)` — so Core decides what survives a failure. Never re-derive the previous value from the sections the app is rendering: that is the same decision read backwards out of the UI.

A screen whose rows come from a store query rather than a Core record — the stake delegations, the asset transactions, the activity list — tells its refresh what it shows (`GemStakeService::refresh` takes the delegations, `GemAssetDetailsService::refresh` and `GemTransactionsService::refresh` take `has_transactions`), and Core answers with `GemLoadState::refreshed`. An `Error` state takes the place of the empty state and is drawn as the shared error row (`GemListRow::Error` on Android, `ListItemErrorView` on iOS); rows already on screen stay, with no error. Which failure earns that row is `load_error(state, has_rows)`, not a `case .error` each screen writes for itself.

`GemLoadState` is the only shape for these four answers. A screen that needs `Loading | Data | Failed` has not found a fifth case, it has restated this enum, and the four ways it crosses are all on `GemLoadState`: `of` builds it from a `Result`, `into_result` reads it back, `refreshed` folds a sync into what the screen shows, and `data` on `GemLoad<T>` makes the transition. Each app turns it into the state its views already take, once: `GemLoadState.stateViewType(_:)` on iOS, `loadError` plus the row builders on Android.

### One copy model for every address, phrase and key

Copying is the same three-part answer everywhere: the value that reaches the clipboard, the shortened value the toast shows, and what kind of secret it is. A per-call-site `isSensitive` flag is how a screen copies a private key without marking it sensitive, so the kind decides it.

```rust
// models/copy.rs
pub enum GemCopyKind {
    Address { chain: Chain },
    Plain,
    SecretPhrase,
    PrivateKey,
}

impl GemCopyKind {
    pub fn is_sensitive(&self) -> bool { ... }
}

pub struct GemCopy {
    pub kind: GemCopyKind,
    pub value: String,
    pub display: String,
}
```

Core shortens the address (`address_copy` calls `format_address`), so the toast text is not formatted twice. Each app maps `GemCopy` once — `GemCopy.copyModel` on iOS, `ClipboardManager.setCopy` on Android — and a row that offers copying carries the model rather than the pieces.

### A row that a screen only ever draws one way keeps its shape app-side

`GemAssetRowStyle` carries the layout because the same asset row is drawn four ways: the wallet list prices it, select-asset names its network, manage-tokens toggles it, receive copies it. The choice varies, so Core makes it once and both apps switch on `subtitle` and `trailing`.

The rows that do not vary keep their shape in the app. The transaction header and the swap progress row are drawn one way on one screen, so their records carry the finished values and the slot each value lands in stays in the view: a layout enum that every row carried would add redundant per-row payload and type surface without settling a variable decision — the cost [Keep the crossings few](#keep-the-crossings-few) warns about, paid for a decision no one is making twice. A row that draws another row's slots is not one of these: a perpetual market, a position, a price alert and a fee asset fill the asset row's slots, so they return `GemAssetItemRow` and share its renderer.

The test is whether the same row is drawn differently somewhere: if it is, the shape is a choice and belongs in the record; if it is not, it is layout and belongs in the view.

### A row is projected from its value, never fetched from a service

`walletRow(wallet)`, `walletRows(wallets)`, `secretPhraseRows(wordCount)`, `transactionRows(transactions)` and `emptyState(input)` are pure functions of the value, so they are exported as functions, not hung off a service. Reading a row must never require a service the screen does not otherwise have — that is what forces a second service into a view model, a row to be passed down as a constructor argument, or a factory to call `service.walletRow(...)` at the composition root: a projection dressed up as a dependency. This is the one exception to [no trivial exports](#no-trivial-exports): a projection has no owner to be a receiver on, because the value it projects is a remote record and Rust allows no inherent `impl` for it.

A view model that already owns the screen's service still asks that service for anything the *screen* decides. The line is whether the answer depends on state the service holds.

### The row carries the whole answer; the view model only reads it

Once Core owns the row, the app-side model has nothing left to decide. `GemWalletRow` carries `id`, `name`, `subtitle`, `placeholder`, `showsWatchBadge`, `isPinned`, `hasAvatar` and `imageUrl`, and each app maps it once, in a row extension, to the platform model its view takes: iOS `GemWalletRow.listItem` builds the `ListItemModel` that `WalletListItemView` draws, Android `GemWalletRow.uiModel(context)` builds the `WalletRowUIModel` that `WalletItem` draws. Where a model still stands between the row and the view, it is [the row and nothing else](#an-app-row-model-stores-the-row-and-nothing-else).

Two things a record cannot carry are a localized string and a bundled image asset, and both have one home per platform.

**Every Core case that becomes localized text maps in the app's mapper file** ([one mapper per app](#one-mapper-per-app-names-every-core-key-it-renders)): Core returns the case, the file returns the string. Never push the app's string catalog into Core through a foreign localizer — that trades a maintained translation for an FFI crossing per row. Text Core composes itself, such as a default wallet name, is a `GemLocalizedText` case (`WalletDefaultName { index }`) that the mapper renders like any other.

**A Core case that becomes a bundled image maps in the module's style mapper**, beside the colours: `GemEmptyStateImage.image` in iOS `Gemstone+Style.swift` and `GemEmptyStateImage.image()` in Android `style/GemstoneStyle.kt` are that mapping, one place per platform, read by every screen.

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

Android's `GemWalletDetails.uiModel()` and iOS's `WalletDetailViewModel.details` then read it, and neither keeps the `Wallet` for anything the record answers:

```kotlin
internal fun GemWalletDetails.uiModel() = WalletDetailsUIModel(
    walletId = WalletId(row.id),
    name = row.name,
    address = address?.toPrimitives(),
    addressExplorer = addressExplorer?.toPrimitives(),
)
```

```swift
var addressModel: AddressListItemViewModel? {
    guard let account = details.address?.toPrimitives(), let link = details.addressExplorer?.toPrimitives() else { return .none }
    return AddressListItemViewModel(title: Localized.Common.address, account: SimpleAccount(...), mode: .auto(addressStyle: .short), addressLink: link)
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
        if details.state.showsBanners { ... }
    }
    .navigationTitle(details.title)
}
```

A view model getter that derives the record itself — `var title: String { details.title }`, `var showsBanners: Bool { details.state.showsBanners }` — looks free and is not: every getter the body reads crosses again, a dozen crossings per pass on a screen like this one. Helpers that would derive the record again take the already-derived value instead. A getter that only reads a field from an existing record does not cross FFI; retain it when it provides a useful native model interface. Deriving inside an action (`onTogglePriceAlert`) is fine — that is one crossing per tap, not per frame.

Android gets this for free by collecting, not by reading: the record is derived once per emission in the view model's flow and the composable reads the collected value.

```kotlin
val uiModel = combine(chainAssetInfo, session, banners, priceAlerts, ::uiModel)
    .flowOn(ioDispatcher)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)
```

First derive once and pass the result down. Reuse a list projection until its source changes. Add memoization only for measured expensive repetition, with a complete input key and an owner that invalidates it; do not add a cache to hide repeated getters or maintain a second writable view state.

### Sections, actions and destinations are records too

A row is not the only choice a screen makes, and the other three recur often enough to have the same answer.

**Which sections show, and which empty state.** A screen that splits one list into positions, pinned, recents and results decides that split from counts and whether a search is running. Left in the apps it becomes four booleans on each side that drift one at a time.

```rust
#[uniffi::export]
impl GemPerpetualMarketSession {
    pub fn sections(&self, counts: GemPerpetualMarketCounts) -> Vec<GemPerpetualMarketSection> { ... }
}
```

**Which actions the screen or row offers.** Core returns the list of available actions as cases; the app renders each case as its own button. `GemStakeActionItem`, `GemHeaderButtonKind`, `GemAssetAction` and `GemFiatButtonAction` are this shape. An app that assembles the action list itself is deciding what the user is allowed to do, on its own, twice.

**What a tap means.** The destination of a row is Core's answer — `GemSelectRowAction`, `GemDelegationDestination`, `GemAcquireAssetFlow`, `GemBannerDestination` — and performing it is the app's, with the app's own route type per [navigation values are app types](#navigation-values-are-app-types). Core says *open the validator*; the app decides that means pushing `DelegationValidator`. A destination that opens a link carries the link, so a screen never has to pair a tap answer with a separate URL field or read one beside its own per-event switch: `GemBannerContent.destination` is that shape.

**A limit comes with its answer.** When Core hands back a limit, it also answers what the limit implies, or each app invents the comparison against a different count. `GemWalletSearchLimits` carries `assets`, `perpetuals` and `nfts` and answers `has_more_assets(count)`, `has_more_perpetuals(count)` and `has_more_nfts(count)`. A limit that only names a number is half an answer.

### A screen whose state changes is a session

A screen that only reads gets a record. Use a **session** when several user events share domain state and transition rules: an immutable record, event methods that return its next value, and one derived view state. A text field, one selection, navigation state or a secure-setting mirror alone does not need a session; retain a pure projection such as `GemAmountEntry` when it already answers the screen. The fiat example below demonstrates a flow that benefits from a session.

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
    set { session = session.onTypeChanged(quoteType: newValue.toGem()) }
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
- **One `view_state` call returning one record.** Ten getters are ten crossings per render; one record is one, [derived once and passed down](#a-screen-derives-its-record-once-and-passes-it-down).
- **A `Record`, not an `Object`.** Value semantics let a state flow skip an unchanged state and let a Rust test assert a whole screen state as one literal.
- **One session per screen**, the same rule as [one Core service per screen](#7-at-most-one-core-service-on-ios-narrow-cases-on-android). A view model holding two sessions is describing two screens.

Core has no observation primitive and no lifecycle, which is why the reactive half stays in the app. The view model owns the task, the debounce, the cancellation and the navigation; the session owns the answers.

**Async outcomes identify the request that produced them.** Core validates whether the request still applies before changing domain state, including failures. Native cancellation saves work but does not establish freshness. Reuse the owning request/input type; introduce a generation only if overlapping identical inputs require it. [Fiat result acceptance](../core/gemstone/src/services/fiat/session.rs) and [swap result acceptance](../core/gemstone/src/services/swap/session.rs) are the existing examples. The shape a service-backed load takes: the service returns a result carrying the request it answers, [`GemLoadState`](#a-screens-load-state-is-one-core-state-and-a-failed-refresh-keeps-what-is-shown) and the value it loaded — [`GemPortfolioResult`](../core/gemstone/src/services/portfolio/session.rs), [`GemRewardsResult`](../core/gemstone/src/services/rewards/model.rs) — and the session folds it with `on_result`, which drops an answer for a selection the screen has moved past and hands the rest to `GemLoad::data`. A per-screen `Loaded | Failed` enum beside that state is a second answer to a question `GemLoadState` already answers. The view model is then two lines and holds no cancellation flag of its own:

```swift
func refresh() async {
    let result = await service.refresh(walletId: selectedWallet.id.id)
    session = session.onResult(result: result)
}
```

Never pre-assign a freshly loading record and pass it back in as the shown state: that throws away the rows a failed refresh is supposed to keep. [Performance](PERFORMANCE.md) requires rejecting obsolete wallet, asset and quote results. `GemChartSession` carries its request identity the same way; AUD65 adds the currency event.

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
    pub fn view_state(&self, price: Option<AssetPrice>) -> GemChartViewState {
        GemChartViewState {
            period: self.period,
            phase: self.phase(price),
            is_refreshing: self.is_refreshing,
        }
    }
}
```

```swift
var chartState: StateViewType<ChartValuesViewModel> {
    switch session.viewState(price: currentPrice).phase {
    case .loading: .loading
    case let .data(data): .data(ChartValuesViewModel(period: selectedPeriod, chartData: data))
    case .noData: .noData
    case let .failed(error): .error(error)
    }
}
```

```kotlin
val chartUIState = combine(loaded, price) { session, price -> session.viewState(price) }.map { state ->
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

The Android excerpt shows the current phase mapping; its payload-free `StateViewType.Error` loses the error detail. VM141 carries the mapped error to the Android chart error UI. Preserve the error when implementing a new screen rather than copying that omission.

Four rules keep the collapse honest:

- **The phase has one source of truth.** A chart session derives its phase from the canonical loaded chart, last error and loading facts. A session that stores a canonical phase instead must not also store equivalent independent flags. Do not add a second representation of the same state.
- **Empty is not a failure.** `NoData` is its own variant, so a series with one point renders the empty state instead of an error, and neither app has to guess from an `Option`.
- **A progress flag that coexists with content is a field, not a variant.** A refresh happens *while* data is on screen, so `is_refreshing` sits beside the phase; anything that replaces the screen is a variant.
- **Everything the phase needs is inside the session.** The chart session carries its display currency because the phase cannot be computed without it, so no caller supplies a currency. The observed spot price is the one `view_state` argument: the session cannot read the store, and a price older than the last chart point leaves the header where it is. A `view_state` that takes what the screen already asked Core for is a parameter the session should own.

The app switches and stops. No `if isLoading` ahead of the switch, no `default:` inside it: the exhaustiveness is what makes a new variant a compile error on both platforms instead of a blank screen on one.

### A number crosses as a value and a style, never as a string or a callback

The precision ladder, the adaptive rule and its constants (`0.99`, `1e-10`, `100_000`, `0.1`, `0.0001`), the fiat-pins-to-two-places rule and the dust threshold are decisions, and they live in Core: `GemCurrencyStyle::precision`, `GemValueStyle::precision`, `adaptive_precision`, the styles' `abbreviates` and `GemValueStyle::is_dust`. Both apps take the display from `formatted_amount` and render it with their own locale formatter. What is left is the numbers themselves: a row that carries a bare `f64` still leaves each app to pick the style.

Two mechanisms are tempting and both are wrong.

**Do not export a formatter as a foreign trait.** A currency formatter the apps implement would let Core call back for every number, and a view state with fifty rows and three numbers each becomes a hundred and fifty reverse crossings inside one call — the most expensive direction there is, against the rule above. It also breaks a real boundary: [`Formatters`](../ios/Packages/Formatters/) holds only locale formatting, which is why the formatters that read a Core rule live in `GemstonePrimitives`. And it makes a session impure, so a screen's state can no longer be asserted as one literal in a Rust test.

**Do not return a finished string either.** Core's formatter is not locale-aware, so a Core-formatted amount regresses every locale that groups or separates differently.

**A number crosses as its value plus the style that decides how precisely it reads.** Core owns the choice; the app owns the rendering, with one small renderer per platform and no crossing per number.

```rust
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNumberUnit {
    Currency { code: String },
    Symbol { symbol: String },
    Percent,
    Plain,
    Multiplier,
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
    Warning,
    Negative,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFormattedNumber {
    pub value: f64,
    pub unit: GemNumberUnit,
    pub display: GemNumberDisplay,
    pub notation: GemNumberNotation,
    pub tone: GemValueTone,
    pub rounding: GemNumberRounding,
}
```

The style is resolved when the record is built, not carried for the app to re-ask: `GemFormattedNumber::currency` and `GemFormattedNumber::amount` take the value and a `GemCurrencyStyle` or `GemValueStyle` and settle the precision, the abbreviation and the dust cut once. The app reads `display` and renders with `NumberFormatter` or `DecimalFormat`, so rendering the returned numbers adds no further formatter crossings; producing the view state still crosses FFI once.

**A number's sign, its enclosure and its tone are decisions too, and one field each.** `notation` says whether the rendered digits get a forced `+`, get wrapped in parentheses, or neither — three mutually exclusive cases, not two booleans the app can combine into a fourth that means nothing. `tone` says whether the number reads as plain text, as a signed quantity that happens to be zero, as up or down, or as a warning: a price is `Plain` however it moves, a delta is coloured by its sign. Deriving the tone app-side from `value > 0` lets the two apps colour the same headline differently, so the record answers it and each app maps the cases to its own palette, once, in its module's style mapper.

```swift
extension GemValueTone {
    public var color: Color {
        switch self {
        case .plain: Colors.black
        case .neutral: Colors.gray
        case .positive: Colors.green
        case .warning: Colors.orange
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
    GemValueTone.WARNING -> pendingColor
    GemValueTone.NEGATIVE -> MaterialTheme.colorScheme.error
}
```

A row or view state carries `GemFormattedNumber`, never a bare `f64`. The adaptive rule, the abbreviation threshold and the dust cut live once, in Core, with tests that fail if a constant moves.

**Exact quantities retain exact transport.** `GemFormattedNumber` uses `f64`; it is suitable for approximate display quantities, not full-precision balances, approval limits or transaction-critical atomic amounts. Those remain `GemBigInt`/`GemBigUint` with decimals and the existing display policy, as R86 records. Do not introduce precision loss just to reuse a shared row. Counts, indices, timestamps and timeout inputs need a display wrapper only when they are actually rendered.

Currency symbols and placement come from the platform's locale renderer. Shared templates such as `GemAssetRate::text` take locale-rendered pieces and labels. Storage/input values (`GemPriceUpdate`, `GemAssetDetailsInput.price`) do not need display wrappers; existing finished text is not precedent for locale-sensitive quantities. Fiat suggestions and rewards counts already cross as `GemFormattedNumber`.

### An app row model stores the row and nothing else

A row model that copies `url`, `host` and `isSelected` out of a Core record is a partial twin: the copy has to be maintained, and a field added in Core reaches the screen only after someone widens the copy. Hold the record and read through it. A model still earns its place when it owns something the record cannot — formatting that depends on locale or user preference, a binding, a bundled asset, or a join to app-side data — but one stored property is the whole of it. No view protocol forces a model per record: a shared view takes the Core record, or the one `ListItemModel` or `ValueHeader` a mapper extension builds from it, so a model that would only pass fields through is not written.

The test is what the model carries that the record cannot, not how much they overlap. A screen input that holds two app assets and a max-amount flag and *projects* the Core request from them is not a twin, however many field names rhyme; a record that carries the same fields re-typed, so that both have to move together, is one whatever it is called. Holding the domain object *and* the row is the twin this rule exists to prevent: the model then has two places to answer the same question, and the two apps answer it differently.

```swift
struct ChainNodeViewModel {
    let row: GemNodeRow

    var url: String { row.node.url }
    var canDelete: Bool { row.canDelete }
}
```

`GemNodeRow` already carries the node, its title, its latest-block subtitle, its latency status and whether it can be deleted, so the model stores the row and every member reads through it. This is the same rule as [no hand-written twins](#6-where-derived-domain-answers-live), applied to the presentation layer. A view-facing UI model that turns the row into platform values for a composable (Android's `GemNodeRow.uiModel(context)`) is the [translation](#a-ui-state-class-translates-the-view-state-it-does-not-re-shape-it), not a twin: it holds no Core type.

A shared view reads the record through the same kind of extension:

```swift
extension GemPerpetualBalanceHeader {
    public var valueHeader: ValueHeader {
        ValueHeader(title: total.text(), subtitle: Localized.Wallet.availableBalance(available.text()), buttons: actions.headerButtons)
    }
}
```

```kotlin
class TransactionDataAggregateImpl(
    private val row: GemTransactionRow,
) : TransactionDataAggregate {
    override val id: TransactionId = TransactionId(row.id)
    override val title: GemTransactionTitle = row.title
    override val valueTone: GemValueTone = row.valueTone
}
```

When a member cannot read through the row — a title, a symbol, an id for the image — that field belongs on the Core row, not on a second thing the model holds alongside it.

Four questions settle where a member goes, in order:

1. **Is it a decision?** Which label, which order, which style, whether a thing is shown — it goes in the Core row. Never recompute it from the domain object the row was built from.
2. **Is it a displayed number?** Approximate quantities use `GemFormattedNumber` and the existing `text()` renderer. Exact atomic amounts retain their typed value, decimals and display policy as specified above. Reuse the corresponding renderer; add a shared number shape only for a real missing display requirement.
3. **Is it which slot a value lands in?** Whether a row shows a label on the left and a price on the right, or a price and a percentage, is a decision. Core names the slots — `GemAssetItemRow`'s `subtitle`, `subtitle_extra` and `trailing`, each a `GemLocalizedText` with its tone — so neither app switches on the kind to lay the row out.
4. **Is it a platform value?** A `Color`, an `Image`, a Compose or SwiftUI value — these are the only things the apps decide, and they go on an extension in the module's mapper files, never inline in the row model. The view reads `text.tone.color`; it never switches on a Core enum itself.

A row model whose body is anything but `row.` lookups has taken back a decision Core had already made. If you are writing `switch row.kind` in an app, the row is missing a field.

```swift
// Gemstone+Style.swift
extension GemValueTone {
    public var color: Color { ... }
}

// Gemstone+Localized.swift
extension GemLocalizedText {
    public var text: String {
        switch self {
        case let .priceAlertLabel(label): label.text
        ...
        }
    }
}
```

```kotlin
fun GemLocalizedText.string(context: Context): String = when (this) {
    is GemLocalizedText.PriceAlertLabel -> context.getString(label.stringRes())
    ...
}
```

Those extensions live in the app's [two mapper files](#one-mapper-per-app-names-every-core-key-it-renders), never in one file per type. A number's text never belongs in either — it comes from `GemFormattedNumber.text()`, which both apps already have once.

### Keep the crossings few

A record crosses by copy. Count calls and copied payloads, and measure the Rust work behind them. The rules below avoid unnecessary crossings without introducing a caching framework.

**Feature views do not call Core directly.** A body or composable can run repeatedly, so the model owns derivation and the view consumes the prepared result — [derived once and passed down](#a-screen-derives-its-record-once-and-passes-it-down). Do not introduce `remember` around a feature-level Core call as a substitute for the model boundary.

**One call per list, not one per row.** A rule that takes the whole list — `matching_assets(assets, query)` — costs one crossing; the same rule as `matches(asset, query)` costs one per item and copies each item twice. `transaction_rows(transactions)` builds a page of rows in one crossing, where `transaction_row(transaction)` per item would send a whole `TransactionExtended` in and a row back for each one — a page of 500 would be 1000 record copies. Prefer `rows(items) -> Vec<Row>` for anything that can be long.

**Reuse list rows until their source changes.** Use the existing observed-query/flow projection rather than rebuilding a page for every row render. A separate cache is warranted only when measured and keyed by all inputs; observation-owned values are often sufficient.

**Derive shared boundaries once, then label sections.** A day header is today, yesterday, or a date. The existing API constructs the boundaries once per list build and calls `label` once per section. Both `boundaries` and `label` are exported methods, so the label calls below still cross FFI; they are not free local comparisons. Keep this work per section, not per row:

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
boundaries.label(day: date.gemDay).title           // nil for .date, which the formatter prints
```

```kotlin
private val boundaries = LocalDate.now().gemDay().boundaries()   // one crossing per list build

fun format(date: LocalDate, locale: Locale): String = when (boundaries.label(date.gemDay())) {
    GemDayLabel.TODAY -> todayLabel
    GemDayLabel.YESTERDAY -> yesterdayLabel
    GemDayLabel.DATE -> DateTimeFormatter.ofLocalizedDate(FormatStyle.LONG).withLocale(locale).format(date)
}
```

Core owns the calendar-day boundary calculation and label policy. Each section label call copies those small values; native grouping still uses the device timezone. If profiling identifies section-label crossings as significant, batch them in the existing projection rather than adding a cache or duplicating policy speculatively.

**Measure crossings on both platforms.** iOS uses the C bridge and Android uses JNA; their overhead differs, so batch bounded list projections and measure on Android as well as iOS. Native locale separators and timezone grouping remain platform formatting. The day-label policy above stays in Core; do not duplicate it merely because its implementation is short.

**Derive the view state; do not store it.** Android composes it declaratively — `combine(session, isUrlLoading, assetPrice) { session.viewState(...) }` — and iOS's equivalent is a computed property, because the inputs a screen does not own arrive from a database observation it cannot hook. Storing the result and updating it by hand goes stale the moment one of those inputs changes without a call site remembering. Derive on read, and keep the crossings down by deriving [once per render and passing it down](#a-screen-derives-its-record-once-and-passes-it-down):

```swift
var viewState: GemFiatViewState {
    session.viewState(assetPrice: priceUsdQuery.value, isUrlLoading: urlState.isLoading)
}
```

```kotlin
private val viewState = combine(session, isUrlLoading, assetPriceUsd) { session, isUrlLoading, priceUsd ->
    session.viewState(priceUsd, isUrlLoading)
}.stateIn(viewModelScope, SharingStarted.Eagerly, session.value.viewState(null, false))
```

**Screen state is a record.** A `Record` copies once and every read afterwards is free, where a `uniffi::Object` crosses by handle but charges a crossing for every property read. Use an `Object` only when the state is large and read rarely, or when it owns something Rust-side. Screen state derives `PartialEq` so a Rust test can assert it as one literal; the generated Swift and Kotlin records compare by value either way.

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

**The native store speaks primitives only.** `ios/Packages/Store` and `android/data/services/store` never reference Gemstone, and a Core store trait is implemented only in the adapter layer (`GemstoneServices`, `data/services/gemstone`). Core types stop at the adapter, which maps them to primitives or store-owned types (`AssetsRequestFilter`) and passes Core values such as `transactionsListLimit()` in as parameters.

**Migrations fail loudly.** iOS `run()` only creates tables or recreates a cache; a column change belongs in `runChanges()`, which also runs on fresh installs, so each step checks what exists instead of `try?`. A cache of server data is dropped and recreated, not migrated. The Room version ships with its exported schema and a registered migration, and never falls back to a destructive migration.

### Choose the persistence owner

| What the service needs | Trait | Shape | iOS | Android |
| --- | --- | --- | --- | --- |
| rows in the database | one `Gem<Name>Store` per persistence owner ([example](../core/gemstone/src/services/price_alert/store.rs)) | database reads and writes use `async` where either platform requires it; only already-held in-memory point reads may be sync; every method returns `Result<_, GemServiceError>` | GRDB adapter under [`GemstoneServices/Sources/Stores/`](../ios/Packages/GemstoneServices/Sources/Stores/) | Room adapter under [`data/services/gemstone/.../stores`](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/stores/) |
| a value the user set | [`GemPreferencesStore`](../core/gemstone/src/services/preferences/store.rs) through `GemPreferencesService` | sync; `get` returns `Option<String>` and **cannot fail** | `GemstonePreferencesStore` over `UserDefaults` | `GemstonePreferencesStore` over `SharedPreferences` |
| the same, per wallet | `GemWalletPreferencesStore` through `GemWalletPreferencesService` | sync, keyed by `WalletId` | same file layout | same file layout |
| a secret | [`GemSecureStore`](../core/gemstone/src/services/preferences/store.rs) | sync; **every read can fail** | `GemstoneSecurePreferencesStore` over the Keychain | `TinkGemPreferences` over Tink |
| something only the OS can do | a foreign trait of its own (`GemNotificationPermissions`, `GemStreamConnection`) | whatever the platform needs | app class | app class |

Use one trait per persistence owner; closely related rows such as contacts and their addresses may share a trait. Preferences use named `const` keys and typed accessors on the existing owner; apps never duplicate raw keys. Existing keys such as `price_alerts_enabled` are valid. Preference reads are infallible; secure-store failures must propagate.

Store methods use `get_*` for reads, `is_*` for boolean reads, `set_*` for preferences, flags or sets, `save_*` for upserts, `add_*` for inserts that must not overwrite, `update_<items>(…, items, delete_ids)` for reconciliation, `delete_*` for removals and `clear*` for a whole scope.

### Store adapter example: price alerts

The iOS adapter lives under [`GemstoneServices/Sources/Stores/`](../ios/Packages/GemstoneServices/Sources/Stores/) and uses the generated `toPrimitives()` conversion:

```swift
public final class GemstonePriceAlertStore: GemPriceAlertStore, @unchecked Sendable {
    private let store: PriceAlertStore

    public init(store: PriceAlertStore) {
        self.store = store
    }

    public func updatePriceAlerts(alerts: [Gemstone.PriceAlert], deleteIds: [String]) async throws {
        try store.diffPriceAlerts(
            deleteIds: deleteIds,
            alerts: alerts.map { (id: PriceAlertFormatter.shared.alertId(alert: $0), alert: $0.toPrimitives()) },
        )
    }
}
```

The Android adapter lives under [`data/services/gemstone/.../stores`](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/stores/):

```kotlin
class GemstonePriceAlertStore(
    private val priceAlertsDao: PriceAlertsDao,
    private val priceAlertFormatter: PriceAlertFormatter,
) : GemPriceAlertStore {

    override suspend fun updatePriceAlerts(alerts: List<uniffi.gemstone.PriceAlert>, deleteIds: List<String>) {
        priceAlertsDao.update(alerts.map { it.toPrimitives().toRecord(priceAlertFormatter.alertId(it)) }, deleteIds)
    }
}
```

Both implement the same conflict, missing-row and changed-only write semantics. A platform difference in those semantics is a bug, not a rendering choice.

### Atomic changes, concurrent publication and query contracts

Core computes a complete domain change; one feature-store operation commits related fields and rows in a native transaction. Moving policy out of an adapter must preserve its existing atomicity. A conditional update carries its condition or expected state into the write, so another writer cannot invalidate a Core pre-read. Use focused store operations and the existing transaction runners, not a generic unit-of-work framework.

The existing hide/unpin adapters are the atomic-write example: [iOS balance updates](../ios/Packages/Store/Sources/Stores/BalanceStore.swift) and [Android asset configuration](../android/data/services/store/src/main/kotlin/com/gemwallet/android/data/service/store/database/AssetsDao.kt) update both fields together. Core sends one asset-configuration patch, so separate hide and unpin calls would weaken the contract. Address-name replacement stays conditional, and perpetual collateral clearing stays one transaction.

Atomicity of one batch does not order overlapping operations. The [balance publication contract](#publish-each-source-of-a-refresh-as-it-answers) is how stale responses and different balance-kind updates publish without overwriting newer data. Preserve explicit wallet/asset identity and independent requests; a global lock is not a default solution.

Observed queries stay native. Their product contract specifies wallet scope, inclusion, ordering, limits and missing rows; paired adapter/query fixtures verify those semantics. Do not solve query parity by copying an unbounded wallet across FFI and sorting it on every emission. `BalanceStoreTests` and `BalancesDaoTest` are those contract tests, through the real GRDB/Room adapters (MIG6).

### Publish each source of a refresh as it answers

A refresh that asks several sources at once — [`GemBalanceService.update`](../core/gemstone/src/services/balance/mod.rs) asks every chain of a wallet concurrently, and on each chain the coin, staking, token and earn balances separately — writes each chain as soon as that chain has answered, so the fastest network shows first ([product/wallet.md](product/wallet.md)). The contract each app's observers rely on:

- **One write per chain, and it is atomic.** Both adapters write a chain's batch inside a single database transaction, so an observed query never sees a chain half updated.
- **A source that fails holds nothing back.** On a chain, the components that answered are written even when another failed; across chains, one chain's failure never delays another's write. The first failure in request order is returned after every chain has finished, so the caller can report it without discarding good data. `published_balances` owns the component split and is tested on its own.
- **A source that fails leaves its rows as they were.** There is no "unknown" state: the previous values stay and stay visible, so a total computed while one chain is offline is a total of older values for that chain, not a total missing it.
- **The wallet is named, not implied.** Every write is keyed by the `WalletId` the refresh was asked for, so a response that lands after the user switched wallets writes the wallet it belongs to and never the one on screen.
- **Only rows whose values differ are written.** The refresh reads the stored rows, folds its updates onto them by kind — a stake answer does not clear a coin's available balance — and drops the rows that come back equal.
- **Overlapping refreshes publish in order, one wallet at a time.** Each refresh takes a sequence number when its fetch starts and holds that wallet's publication lane for its read-fold-write, so a concurrent coin and stake answer cannot lose each other and a response that arrives after a newer one for the same asset and kind is dropped instead of written. Other wallets and the price socket lane stay concurrent.

A write per chain notifies the observers once per chain, and a total read between two writes mixes the new values of one chain with the older values of another for that moment; the owner chose that over holding every chain back for the slowest.

## 5. The app maps; it does not decide

### iOS

There is no app-side service wrapping a Core service. The view model holds its screen's Core service and calls it; each call is one line in, one mapping out.

```swift
func refresh() async {
    details = await service.refresh(details: details)
}
```

Core → app mappings live in `GemstonePrimitives` as extensions. A mapping onto a *feature-internal* type stays in the feature — `GemstonePrimitives` cannot import a feature module, and reaching for one is the signal that the mapping belongs in the feature.

### Android

Commands and point reads call the generated service interface directly. For an observed read, keep a narrow case in `gemcore` `application/<area>/cases/`, implemented in `data/coordinators/<area>/` and injected by Hilt. Its `Flow` feeds each relevant emission to the Core projection:

```kotlin
override fun getTransactionDetails(id: TransactionId): Flow<TransactionDetailsAggregate?> = combine(
    getSession().filterNotNull(),
    getTransaction(id),
) { session, data -> Pair(session, data) }
    .mapNotNull { (session, data) ->
        data?.let {
            TransactionDetailsAggregateImpl(
                transactionDetailsService.detailRows(it.toGem(), session.wallet.type.toGem()),
                session.currency,
            )
        }
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
        selected.validator.toPrimitives()
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

The boundary is where a feature interprets domain data. Its model holds the Core record and exposes prepared platform values; declaring a domain twin is a [separate violation](#6-where-derived-domain-answers-live). Feature views must not call Core, select localized wording from domain cases or reconstruct domain decisions. Shared renderers deliberately accept Core row records, and exhaustive dispatch of an opaque row key to its existing renderer/model is allowed. A type name alone is not a reason to add a wrapper.

Two common leaks are a view switching on a Core enum to pick localized text, and a child feature view receiving a raw domain record that it interprets itself. Move that translation into the model; direct pass-through to a shared row renderer remains intentional.

```swift
// scene
CurrencyInputValidationView(text: $model.amount, error: model.amountError, config: model.currencyInputConfig)

// model: the Core enum stops here
var amountError: (any Error)? {
    let state = viewState
    return switch state.phase {
    case .noInput, .loading, .noQuotes, .failed: nil
    case .invalidInput: state.phase.inputErrorText.map(AnyError.init)
    case let .invalid(check): check.errorText(locale: locale).map(AnyError.init)
    case .ready: state.amountCheck.errorText(locale: locale).map(AnyError.init)
    }
}
```

```kotlin
// composable
error = uiState.amountError ?: ""

// ui/localization/GemstoneText.kt, called from the view model: the Core enum stops here
fun GemFiatViewState.amountErrorText(context: Context): String? = when (val phase = phase) {
    GemFiatQuotePhase.InvalidInput -> context.getString(R.string.errors_invalid_amount)
    is GemFiatQuotePhase.Invalid -> phase.check.string(context)
    GemFiatQuotePhase.Ready -> amountCheck.string(context)
    GemFiatQuotePhase.NoInput, is GemFiatQuotePhase.Loading, GemFiatQuotePhase.NoQuotes, is GemFiatQuotePhase.Failed -> null
}
```

Naming a Core type is not the test; deciding from one is. A view that iterates a row key and hands each case to a component is doing what [§ 3](#3-return-one-record-that-answers-the-whole-question) asks — the key is the screen's contract, and the switch over it is exhaustive on purpose. Some scene and `presents/` files name a Core type for exactly that reason.

The shapes that close a leak are a row model with an app kind or destination where a view would switch on a Core row key, a model the view model vends where a Core record would pass through to a child view, and a closure typed by the view model where a view would declare a Core-typed callback; navigation payloads are the app's inputs (`ConfirmTransferInput`, `AmountInput`).

What to grep for is a view that interprets a domain answer outside its mapper: a `switch`/`when` over a Core type whose arms produce `Localized.` or `stringResource` — the localized text belongs to the app's mapper, not the body — or a Core record passed into a child view's initializer. On Android:

```
rg -tkotlin -U 'when \([^)]*\)\s*\{[^}]*(stringResource|R\.string)' android/features --glob '**/presents/**'
```

and its Swift equivalent over `Sources/Scenes/` and `Sources/Views/`. The pattern also matches switches over app enums; a hit over a Core type is a decision that has to move one layer down, and everything else the type-name grep finds is the contract working.

### One mapper per app names every Core key it renders

Every Core value an app turns into a platform value goes into one of exactly two files per app, and a new `SomeGemType+Module.swift` or a feature-level mapper is always the wrong answer:

| | iOS | Android |
|---|---|---|
| Core key → localized text | `PrimitivesComponents/Sources/Extensions/Gemstone+Localized.swift` | `ui/src/main/kotlin/.../ui/localization/GemstoneText.kt` |
| Core enum → colour, image, or other platform value | `PrimitivesComponents/Sources/Extensions/Gemstone+Style.swift` | `ui/src/main/kotlin/.../ui/style/GemstoneStyle.kt` |

Every feature already imports `PrimitivesComponents` or depends on `:ui`, so a feature reads the shared mapping and never writes its own; the members are public. One file per app means one Core enum has one mapping per app: a feature that needs a second phrasing of the same key — a tab title and a field label — adds it to that file under a different name, the way Android's `tabStringRes` and `fieldStringRes` do. A view model, a scene or a composable that maps a key somewhere else has taken the app's vocabulary private, and the two apps drift one key at a time: the same title reads "Claim Rewards" on one app and "Rewards" on the other, or one field gets two labels. A service that turns a Core failure into an `Error` is not a presentation mapper and stays with the service (`GemWalletConnectFailure+WalletConnectorService.swift`).

The mapper is the only place a `Localized.`/`R.string` is chosen from a Core variant, which is what makes the two apps comparable. `just check-mappers` reads the two localization files into variant → key, resolves both keys to their English text, fails on every variant the two apps resolve differently, and fails on any other `Gemstone+Localized.swift`, `Gemstone+Style.swift`, `GemstoneText.kt` or `GemstoneStyle.kt`. It only sees a variant while both mappers hold it, so a key mapped anywhere else is invisible to it. `just check-docs` does the same for the guidance: every link in `docs/`, `skills/` and the `AGENTS.md` files has to point at a file that exists, a heading that exists, and — when the label is a backticked name — a symbol that is still in that file.

The mapper check compares variants found in both mapper files; it does not prove coverage or domain parity. `just check-boundaries` is the structural gate (MIG5): a rule joins it when a regex can decide it exactly and stays a review lead when it cannot. Treat a census hit as a review lead and explicitly exempt shared renderers, native ports, DI and legitimate child dependencies.

Chain and asset icons come from Core: `ChainConfig.icon_chain` supplies the chain logo; `GemAssetConfigService::asset_icon` chooses the asset image and badge. An EVM layer 2's native coin uses Ethereum's image only when it is ETH; the family alone does not decide its gas coin.

### A UI state class translates the view state; it does not re-shape it

Compose screens collect a single state object, and the temptation is to declare it as a mirror of the Core view state — the same fields, re-typed. That is a twin with a different name: every Core field it restates has to be maintained in step, and the Core types it carries reach the composable anyway, so the [view boundary](#a-view-never-names-a-core-type) is not actually closed.

A UI state class is the **translation** of the Core view state into platform values. It holds no Core type. Every property is a `String`, a `Boolean`, a `@StringRes` id, or an app enum, and each one names what the view does with it rather than what Core called it.

```kotlin
data class FiatUiState(
    val isLoading: Boolean = false,
    val amountError: String? = null,
    val quotesMessage: String? = null,
    @StringRes val actionTitle: Int = R.string.common_continue,
    val retries: Boolean = false,
    val buttonState: ButtonState = ButtonState.Disabled,
    val canSelectProvider: Boolean = false,
)
```

A state that carried `GemFiatQuotePhase`, `GemFiatAmountCheck` and `GemFiatButtonAction` straight through would make the composable branch on all three, and a new Core case would break a screen instead of a model.

Two consequences worth stating:

- **Localization happens in the view model layer,** because that is where the branch over the Core enum lives. An Android view model takes `@ApplicationContext` and hands it to the module's mapper (`FiatViewModel` calls `amountErrorText(context)`); a `@StringRes` id on the state is the alternative when no arguments are needed. iOS resolves `Localized.*` in the model or its mapper for the same reason.
- **The tests improve.** A test that asserts `uiState.phase == GemFiatQuotePhase.Failed` asserts a Core value the user never sees; asserting `uiState.quotesMessage` and `uiState.retries` checks what the screen actually shows.

The same rule reads on iOS as: the view model exposes `String`, `Bool` and app enums; the Core record stays private behind them. A row model is the small case of this — it holds the Core record and exposes platform values from it — and a UI state class is the screen-sized one.

### A list row renders from one shared row model

Both apps draw a plain list row from one component model: `ListItemModel` in iOS `Components` and `android/ui`. It carries the finished title, the title tag, the extra line under the title, the value on the right with its extra line, the leading image and the info sheet, each with a style the view resolves to a font and colour. A sectioned screen returns `ListSection<T>` — an id, an optional title and the items — and the scene renders it with one call: `ForEach` over the sections on iOS, `listSections` on Android.

The view model builds the model and the scene only renders it: `ListItemView(model: model.listItem(for: row))` and `ListItem(model = row.model, listPosition = position)`. A row that also acts carries the app action beside the model — `AcquireOptionUIModel(action, model)` — and the scene reads the action, never the row key. An address row carries the prepared display/copy information and explorer link; the existing shared address renderer may use its dedicated adapter, but feature scenes do not call an address service or reconstruct shortening rules. A switch or a picker renders the same model with the control in the accessory slot; a dropdown picker keeps its options beside the model. Every model row draws its title in the regular body weight on both apps (iOS `.body`, Android `bodyLarge`) whether or not it carries an image or an extra line; a heavier title belongs to a rich composite, never to the shared renderer. Line limits match as well: the title is one line and the line under it wraps unless the row sets `titleLineLimit` or `titleExtraLineLimit`, the same fields on both apps. Moving a row onto the model starts from its iOS twin: a twin that is a `ListItemModel` moves onto the model, a twin that is a composite (the error and info notices, for example) keeps the matching composite on Android or moves to a shared `GemListRow` variant on both apps; before and after, compare the old view's text styles, line limits and slots with the renderer path it lands on, on a device. A trailing image — the wallet icon beside a confirm value — is an accessory, not the model's image (Android's `GemListRowView` draws it through `DataBadgeChevron`), so the model's `image` stays the leading slot on both apps.

Three kinds of row stay outside the model on purpose. The row primitives themselves (`PropertyItem`, `LinkItem`, `ListItem`) are what the model renders through. Rich rows with their own layout stay composites on both apps, named in pairs so a new screen reuses the pair instead of rebuilding it: asset — iOS `ListAssetItemView`, Android `AssetListItem`; recent assets — iOS `RecentsSceneViewModel.listItem`, Android `AssetListItem`; wallet — `WalletListItemView`, `WalletItem`; chain — `ChainView`, `ChainItem`; NFT — `GridPosterView`, `NftListItem`; transaction — `TransactionView`, `TransactionItem`; delegation — iOS `DelegationViewModel.listItem`, Android `DelegationItem`; validator — `ValidatorView`, `ValidatorItem`; swap provider — the provider row of iOS `SwapDetailsView`, Android `SwapProviderListItemView`; network — iOS `ListItemImageView` with the chain image, Android `PropertyNetworkItem`; balance — iOS `AssetBalanceView`, Android `PropertyAssetBalanceItem`. None becomes a `ListItemModel` variant, because each carries what the plain model does not: a semibold title, a badge, a status tag or a two-line trailing value. The iOS recents and delegation rows reach that weight through the row primitive's explicit title style, which is the rich family's iOS form rather than a plain model row, so the regular-weight rule above does not apply to them. Developer screens keep inline rows on both apps. Vector icons never reach a view model: Android carries `ListItemImage.Symbol` and resolves it to `AppIcons` in the image view. The toast a view model emits is the same `ToastMessage(title, image)` on both apps.

What this replaces is the row assembled in the body: `ListItemView(title:subtitle:imageStyle:)` with values read off the view model one by one, and `PropertyItem`/`LinkItem` calls with a title id, a value and an icon passed as separate arguments. Both restate the row's shape on every screen, and the two apps then drift a field at a time. Copy [`ContactsViewModel.listItemModel(for:)`](../ios/Features/Contacts/Sources/ViewModels/ContactsViewModel.swift) and [`ContactsViewModel.listItem`](../android/features/settings/contacts/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/contacts/viewmodels/ContactsViewModel.kt) for a row, [`PriceAlertViewModel`](../android/features/settings/price_alerts/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/price_alerts/viewmodels/PriceAlertViewModel.kt) for sections, and [`RecipientViewModel`](../android/features/recipient/viewmodels/src/main/kotlin/com/gemwallet/android/features/recipient/viewmodel/RecipientViewModel.kt) for sections whose rows format an address through Core.

The NFT list, fiat transaction row and curated asset list use title left, secondary value trailing and chevron last. iOS `ListItemModel.subtitle` names that trailing slot, not a second title line. The NFT verified badge belongs to the grid (`GemNftRow.is_verified`), not the list; details use the collection status already held.

### Dispatch Core work that can block

The `flowOn` above is not decoration. A synchronous Core call such as `transactionDetailsService.detailRows` can read store callbacks that block on Room, and UniFFI polls the Rust future on the calling thread — so without it the read lands on main, where Room throws before any work happens.

The coordinator dispatches; it does not leave that to its caller. Whether a Core method touches a store is Core's business and can change without the call site noticing.

A view model is the one caller that does not name the dispatcher itself. Its work belongs to `viewModelScope`, which delivers every emission back on main, so the dispatcher has to be something a test can replace: it arrives as an injected `@IoDispatcher` parameter. See [Code Style](../android/skills/code-style.md).

Cheap pure projections and synchronous in-memory/preference reads can stay on the calling thread; this is how the session and one-derivation-per-render examples work. Calls that can touch a database, network or blocking platform port take the appropriate dispatcher at the boundary. Do not infer that a call is non-blocking merely because its FFI signature is async.

```kotlin
// suspend: move the call
override suspend fun setCurrentWallet(walletId: WalletId) = withContext(Dispatchers.IO) {
    walletSessionService.setCurrentWalletId(walletId.id)
}

// Flow: flowOn after the operator that calls Core
override fun getTransactionDetails(id: TransactionId): Flow<TransactionDetailsAggregate?> = observed(id)
    .mapNotNull { (session, data) -> transactionDetailsService.detailRows(data.toGem(), session.wallet.type.toGem()) ... }
    .flowOn(Dispatchers.IO)
```

## 6. Where derived domain answers live

The app uses Core's types. It does not declare a parallel record or enum of the same shape — that is two definitions of one thing, and every crossing pays for a two-way mapper. The confirm flow carries Core's `GemTransferData` through navigation (`ConfirmTransferInput`) for this reason, not an app enum restating its cases.

Choose the home from ownership first, then decide how it crosses FFI:

| Question | Home | Example |
|---|---|---|
| A field the value already carries, but the generated shape lacks a common accessor | thin app mapping extension | `inputType.stakeType` (Android) |
| A pure answer has one honest domain receiver; additional value arguments are allowed | method on the receiver | `bannerKey.identifier()`, `input.addAddress(addresses)` |
| A pure rule has no honest receiver | private rule called by the owning service | `selectable_fee_assets(...)` |
| The answer requires I/O, stored dependencies or platform ports | method on the service that owns the flow | `confirmService.preload(...)` |
| An app value must be encoded into a Core case | app mapping extension | `.stake(asset, stakeType)` |

**Never add a free exported function or a service wrapper — stateless or not — for an answer already owned by one local Core type.** The transfer record answers for itself: `GemTransferData` exports `input_asset()`, `fee_asset()`, `header_kind()` and `title()`, and the apps call `transfer.feeAsset()` rather than asking a service or a free function. A method that ignores `self` is the same mistake even on a service with real dependencies: a simulation's asset ids are `SimulationResult.asset_ids()`, not a method on an eight-dependency orchestrator. Keep the owning service when the rule performs I/O, holds real dependencies, or combines inputs without a single honest receiver. A request record is an honest receiver when it contains the complete instruction: `GemContactAddressInput.add_address(addresses)` owns its replacement identifier and new address; `GemContactEditorService.add_address(addresses, input)` would ignore every service dependency.

Do not manufacture a receiver by choosing the first parameter. The type is honest only when the answer is part of that type's meaning, the method uses `self`, and extra arguments are plain input values rather than stores, clients or services. For a repository-owned Rust type, prefer an inherent `impl Type`; do not create a one-method extension trait to imitate Swift or Kotlin. Intrinsic structure belongs in the defining crate (`SimulationResult.asset_ids()`), while feature or product policy remains in Gemstone even when it consumes a primitives type.

### Ownership is not transport

A receiver method on a Gemstone-local UniFFI record or enum can cross FFI. Export it only when an app calls it. A TypeShare type defined in a repository-owned Core crate can own canonical inherent Rust behavior there, but TypeShare does not generate that method in Swift or Kotlin. A type merely declared through `#[uniffi::remote]` is defined elsewhere, so Gemstone cannot add an inherent receiver; use a private rule or adapter rather than inventing one. A thin app extension may expose a structural field projection when transport omits it, but it must not copy product policy.

When mobile needs behavior that cannot cross on its honest receiver, prefer folding the answer into an existing aggregate operation. If the UI genuinely needs a standalone pure projection, add it to an existing cohesive FFI adapter such as `GemSimulationFormatter`; do not put it on an I/O service whose dependencies it ignores and do not create a one-method object. `SimulationResult.asset_ids()` is the folded case: it lives in `primitives`, and the confirm and WalletConnect flows call it inside their own loads (`ensure_simulation_assets`), so no app needs it and no app copies it.

A rule that answers for one value lives on that value or is a projection of it, not a method on an object with nothing in it: `swap_quote_summary(quote, from_asset, to_asset)` carries the minimum receive and the rate of a quote, `GemSwapValue::price_impact(receive)` compares two priced amounts, `GemCustomFee::estimate` builds a custom fee and `transaction_rows(transactions)` builds rows. The apps and their tests construct the value; nothing has to be mocked to reach a rule. `GemSwapQuoteService` is the swap screen's real service — swap, balances, preferences, the price stream and the session behind one object — and none of those rules sits on it.

A stateless exported object is acceptable only as a cohesive FFI codec or formatter when UniFFI cannot express an honest receiver or the operation spans several transport types. Name that role explicitly, give it no I/O dependencies, and delegate intrinsic behavior or feature policy to the owning receiver or private rule where possible. `GemSimulationFormatter` and `PriceAlertFormatter` are the current transport adapters. A one-call forwarding object is still a wrapper and should be removed.

`#[uniffi::export]` on an `impl` processes every function in that block regardless of Rust visibility. `pub(crate)` does not remove a method from generated bindings. Put only intended FFI methods in the exported block; move helpers to a separate unannotated `impl` and give them the narrowest Rust visibility. Derive `uniffi::Record`/`uniffi::Enum` only for types that actually cross FFI. After changing an exported member or type, regenerate bindings and build both apps — one platform may never have called the method the other still needs.

The generated code mapper crashing on an unknown `Chain` or `Currency` is not the same problem and is not wire handling: the app's enum is typeshare-generated from the same Core enum in the same build, so the two cannot disagree at runtime, and a crash there reports a broken generation rather than a server value. The one path that can reach it is a downgrade reading a store a newer build wrote; that is not a supported install path, and widening the mapper would hide the generation bug it exists to catch.

Use tolerant entry decoding only where the payload contract permits independent entries to be skipped. The price/rate stream uses `deserialize_known_entries`, so one unknown currency does not discard all prices. This is not a blanket rule for transaction, wallet recovery or authorization payloads: required inputs must remain complete and fail closed under the [security contract](../skills/security.md).

An app's persisted aggregate is not a typeshare candidate just because Core has one of the same name. `AssetData` is the standing example: typeshare renders a `BigUint` as a `String`, so the ten big-integer fields of Core's `Balance` would reach both apps as text to parse; Android's `Balance<T>` is a generic container it instantiates with `BigInteger` and with `Double`, which typeshare cannot express; and the two apps' `AssetData` differ from each other — iOS carries price alerts, Android a wallet id — as well as from Core's. Both are persisted, which is where a twin belongs. Revisit a shape like this only if it keeps the atomic values numeric.

A generated mapper names its direction on both apps: `toPrimitives()` reads a Core value into the app's type and `toGem()` sends one back. An untyped `map()` on both sides made a diff unreadable and let a reviewer miss a crossing going the wrong way. A mapper between two app types keeps `map()` — the directional names mark the FFI boundary and nothing else.

Encoding members are scaffolding, not a pattern to copy. `core/bin/generate/remote_types.yml` lists what `just generate-models` maps: `remote` types get `#[uniffi::remote]` and structural mappers on both apps; `codes` are string-backed enums that cross as their code and get `Primitives.X(core:)` / `.rawValue` on iOS and `toX()` / `toGem()` on Android; `identifiers` are hand-written parsers the record mappers call by convention (`X(core:)` / `.identifier` on iOS, the `X(identifier)` constructor / `toIdentifier()` on Android). Before adding a `remote` type, verify that the generator can represent its full shape and inspect both generated mappers. It handles fieldless enums and records whose fields are scalars, `DateTime<Utc>`, other remote types, codes or identifiers, plain or wrapped in `Option` / `Vec`, whatever subdirectory of `primitives/src` declares them; a `#[typeshare(skip)]` field travels Core → app only and is filled with its empty value on the way back (scalars, `Option`, `Vec` and `String` have one; anything else fails generation). A data-carrying enum maps too when it keeps a twin, as long as each variant carries at most one unnamed payload — a twin renders a named or multi-field variant as a type of its own, which the generator will not invent. A type an app never looks inside does not need a model on either side: pass the wire text and let Core own the format. Never add a second app-side model or copy policy to avoid a gap in the generator; close the gap.

### Identifiers and structural records at the boundary

`Account`, `Wallet`, `SimulationResult`, `Delegation`, `StakeType` and `RedelegateData` already cross as structural remote records in `core/bin/generate/remote_types.yml`; use their generated mappers. `GemJsonValue` remains a JSON string custom type. Do not serialize a structural record just to cross FFI.

Rust signatures use domain types (`WalletId`, `AssetId`, `Chain`, `Currency`). The generated representation differs: `Currency` is a remote enum; `Chain` crosses as its string code; identifiers such as `AssetId`, `WalletId` and `TransactionId` cross as their stored strings. Map at the boundary with the generated conversions; do not replace typed Rust parameters with bare strings.

The identifier string matches native database storage. Mapping the binding directly to an app wrapper would also break Android's dependency direction: its primitives live in `:gemcore`, which depends on the standalone `:gemstone` artifact. Swift's ability to name `Primitives.WalletId` through UniFFI does not make that bridge symmetric. Revisit only if the module boundary changes.

## 7. At most one Core service on iOS; narrow cases on Android

An iOS view model holds **at most one** Core service, named `service`, and it is **`private`**; a model that does not need Core holds none. Reuse the owning domain service when it already answers the screen. Add a screen-level service only when it genuinely composes collaborators or returns a cohesive screen result — never to satisfy a field-count rule. An Android view model holds the same Core service through its generated `GemFooServiceInterface` (`private val service`), plus the observed reads the screen watches as narrow application cases (a Room `Flow` behind `GetPriceAlerts`, `GetTransactions`, `GetWalletAssets`) and `GetSession`. A case that only forwards a Core call — a setter over one service method, a lookup over another — is migration debt: delete it and call the service. A non-private service on iOS usually means the view is reaching through the model for a dependency. A second Core service added so a screen can answer one question is the same mistake from the other side: if the owning service can forward the call, it should, and a one-line forward on it is not a duplicate reader. `show_perpetuals` is the worked example: the preferences service keeps the rule, and the asset selection, perpetual and portfolio services each forward it to the screens they own, so no screen reads the preference itself. The observation seam still registers the setting — `ObservablePreferences.isPerpetualEnabled` on iOS, `UserConfig.isPerpetualEnabled()` on Android — and the answer comes from the service.

**Native observation is intentional.** iOS holds `ObservableQuery<Request>` over GRDB `ValueObservation`, with `BindableQuery` as the database-injection seam. Android injects a narrow case returning a Room `Flow`. Both are observation dependencies beside the service; neither is a second business owner.

**A value model may need no service.** `NetworkFeeSceneViewModel` renders `GemConfirmFeeSelection`, `GemFeeRateRows` and `GemFeeOptionItem` with callbacks. Injecting a service into a model that only reads those answers adds no behavior. Judge ownership by decisions, not member counts.

This limit does not count explicit platform ports such as a signer, keystore, observation source or navigation builder. Those remain narrow injected dependencies; they do not decide shared product behavior.

A store is not one of those ports. A store is the database side of a Core service, and a view model that holds one has reached past the service into what the service owns — the same coupling a second service would be, by a shorter path, and the operations it reaches for then exist on one platform only. The developer screen's clear actions are `GemDeveloperStore` operations its service exports, so both apps offer them from the one service each view model holds:

```swift
DeveloperViewModel(walletId: walletId, service: developerService, devicePlatform: devicePlatform)
```

```kotlin
class DevelopViewModel @Inject constructor(
    private val service: GemDeveloperServiceInterface,
    private val getSession: GetSession,
    val notificationsAvailable: NotificationsAvailable,
) : ViewModel()
```

A second service is never the way to reach a value the screen renders. When a view model needs an answer its own service does not hold, the fix is one of three, in order: the answer is a pure projection and becomes a function of the value it projects ([a row is projected from its value](#a-row-is-projected-from-its-value-never-fetched-from-a-service)); the screen's own service or session already receives the input and returns the answer alongside the rest of its view state; or the screen was drawn around the wrong service. Widening the constructor is not on the list, and neither is having the composition root call the other service and pass the result in — a factory line that reads `walletService.walletRow(...)` next to an unrelated service is the same coupling with a longer path.

When a real screen-level service is needed, name it for the screen it backs, not for the layer: `GemContactEditorService` backs the add-and-edit screen. No `Scene` or `Facade` in the name. A service that only forwards calls to an owner is wrapper debt, not the pattern: the contacts list screen holds the owning `GemContactService`. When a screen needs a cohesive answer from several Core owners, Core composes them:

```rust
#[derive(uniffi::Object)]
pub struct GemContactEditorService {
    contacts: Arc<GemContactService>,
    addresses: Arc<GemAddressService>,
    payments: Arc<GemPaymentService>,
}

#[uniffi::export]
impl GemContactEditorService {
    #[uniffi::constructor]
    pub fn new(contacts: Arc<GemContactService>, addresses: Arc<GemAddressService>, payments: Arc<GemPaymentService>) -> Self { ... }

    pub fn scanned_address(&self, input: String) -> GemContactScannedAddress { ... }
    pub fn default_chain(&self) -> Chain { self.contacts.default_chain() }
    pub async fn save_contact(&self, input: GemContactInput) -> Result<Contact, GemServiceError> { ... }
    pub fn format_address(&self, address: String, chain: Chain, style: GemAddressFormatStyle) -> String { ... }
}
```

The pure address-list transformation stays on its request value: `input.add_address(addresses)`. It does not belong on this service because it uses none of the service's dependencies.

A sheet belongs to the screen that presents it and shares that screen's service. A screen you navigate *to* is a different screen with its own.

### Direct service calls and observed reads

On iOS, the factory injects the generated protocol into the view model alongside native observation:

```swift
@Observable
@MainActor
public final class SupportChatSceneViewModel {
    private let service: any GemSupportServiceProtocol
    private let typing: ObservableSupportTyping

    public init(service: any GemSupportServiceProtocol, typing: ObservableSupportTyping) {
        self.service = service
        self.typing = typing
    }
}
```

`ViewModelFactory.supportChatScene()` supplies these dependencies; feature packages never read app-level environment service keys. A class implementing a Core store or platform trait is an adapter. A class that merely re-exposes Core calls is a redundant wrapper.

Android likewise calls the service directly for commands and point reads. [`PriceAlertViewModel`](../android/features/settings/price_alerts/viewmodels/src/main/kotlin/com/gemwallet/android/features/settings/price_alerts/viewmodels/PriceAlertViewModel.kt) rereads persisted state after the command, including on failure:

```kotlin
private val alertsEnabled = MutableStateFlow(service.isEnabled())

fun togglePriceAlerts(enable: Boolean) = viewModelScope.launch(ioDispatcher) {
    runCatchingCancellable { service.setEnabled(enable) }
        .onFailure { Log.e(TAG, "setting price alerts enabled failed", it) }
    alertsEnabled.update { service.isEnabled() }
}
```

[`GetPriceAlertsImpl`](../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/pricealerts/GetPriceAlertsImpl.kt) separately observes through `GemstonePriceAlertStore`; it does not wrap commands. Core classifies each alert through `alert_kind` and `PriceAlertFormatter`; this observer selects and groups the stored rows.

For a real platform-only concern, iOS uses a feature service in `Features/<Feature>/Sources/Services/`, constructed by the app and injected. Android uses a case in `gemcore` `application/<area>/cases/`, implemented in `data/coordinators/<area>/`. Neither path bypasses a Core persistence owner. Recent activity commands belong to `GemRecentActivityService`; the native stores supply persistence and observation.

### Composition services are reached through the screen service

`GemExplorerService`, `GemDeeplinkService`, `GemSwapService`, `GemAssetConfigService`, `GemPriceService` are *composition* services: screen services hold them, and a screen reads their answers through its own service — the chart's contract link rides in `GemChartService::sections`, the confirm screen's sender link is `GemConfirmTransferService::address_url`, the asset screen's share link, token link and swap pair ride in its `GemAssetDetails` record (`share_url`, `token_link`, `swap_pair`), and the confirm sheet's acquire flow is `GemConfirmation::acquire_asset_flow`. A composition service's method is exported only while an app still calls it ([no trivial exports](#no-trivial-exports)). One route per answer, and it is the screen service's: an answer also reached through a Hilt-injected coordinator or a `CompositionLocal` is a second route, and two routes disagree somewhere (a slippage default, an acquire-flow title). A composition service therefore reaches the Compose tree only at the root: the navigation builder, the one caller that is not a screen, reads `LocalDeeplinkService` and `LocalAssetsService` once in `rememberWalletNavigationState` and hands them to `WalletNavigator` as parameters; no feature composable reads one.

On Android the Hilt module binds both the concrete class and the generated interface (`fun provideGemFooServiceInterface(service: GemFooService): GemFooServiceInterface = service`): Core constructors need the concrete type to compose, view models and coordinators take the interface.

### A service never hands out another service

A service method that returns another service (`service.contactEditor()`) is the same reach-through as `model.nameService`, one level down: the caller depends on something it was not given. Every service is constructed in the composition root and injected. Returning `Arc<GemFooService>` from an exported service is migration debt, not an exception to this rule.

A **shared component** — `AddressInputViewModel`, `NetworkSelectorViewModel` — takes the Core service it needs by its own protocol: `AddressInputViewModel` and `NameRecordViewModel` take `any GemNameServiceProtocol` (`GemNameServiceInterface` on Android), and the parent view model receives that `nameService` as a plain constructor dependency beside its `service` and passes it down. The screen service does not forward name methods and the client does not declare a protocol intersection (`any GemFooServiceProtocol & AddressInputResolving`) or a builder closure to reach the component's dependency — both hide a second dependency inside the first. `NetworkSelectorViewModel` needs only the dependency-free `GemChainService` and reads `GemChainService.shared` itself ([the fieldless exception](#8-services-are-injected-never-constructed-at-a-call-site)).

### The parent vends the child model, the view never reaches in

```swift
// wrong — the view assembles the child from the parent's internals
ContactAddressEditorScene(
    model: ContactAddressEditorViewModel(
        defaultChain: model.defaultChain,
        nameService: model.nameService,
        addressService: model.addressService,
        onComplete: model.onAddressComplete,
    ),
)

// right — the parent owns the wiring, the view asks for a model
ContactAddressEditorScene(model: model.addressModel(mode: mode))
```

Where the child is a different screen with its own service, the parent cannot build it — feature modules cannot see the composition root. The app passes the builder in:

```swift
public func contactsScene(mode: ContactsViewModel.Mode = .list) -> ContactsViewModel {
    ContactsViewModel(service: contactService, contactEditor: contactEditorScene, mode: mode)
}
```

Android does not hit this at all for a child of the same screen: one Hilt view model owns both pages and the composable switches on the page it reports, so there is no child model for a view to assemble.

```kotlin
AnimatedContent(targetState = uiState.page) { page ->
    when (page) {
        ContactEditorPage.Form -> ContactEditorScene(state = uiState, onAction = ...)
        ContactEditorPage.Address -> uiState.addressInput?.let { input ->
            ContactAddressEditorScene(input = input, onAction = ...)
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

A `GemFooService()` in a field initialiser or at file scope is a second instance the graph does not know about, and it is where an app-side variant creeps back in. `just check-boundaries` rejects a new one: it reads which services Core declares with no fields, and holds every other one to the composition roots named below.

- **iOS** — an owner (a service with a store, a client, a stream, or anything the app needs from launch) is registered in `ServicesFactory`, exposed through an `@Entry` in `ios/Gem/Types/Environment.swift`, and passed into the view model. A screen service — one that only composes owners for a single screen (`GemAssetDetailsService`, `GemChartService`, `GemTransactionDetailsService`, `GemWalletHomeService`) — is built in the `ViewModelFactory.xxxScene(...)` that builds its view model, from the owners the factory already holds. It is never a field of `AppResolver.Services` and never an `@Entry`: that constructs it on every launch of an app that may never open the screen, and hands views a composition detail.
- **Android** — provided in a Hilt module, injected. A Compose scene reads one instance from a `CompositionLocal` provided at `MainActivity` (`LocalChainService`, `LocalAddressService`) only for the dependency-free config services; a screen's Core answers come from its view model's service, never from a `CompositionLocal` inside a feature composable (a share link or a confirm button's acquire flow built there is a reach-through). A non-`@Composable` helper takes an explicit parameter — a `CompositionLocal` cannot be read outside a composable. Hilt scope and the requesting graph determine lifetime; `@Provides` alone guarantees neither screen scope nor deferred startup. Stateless composition may be shared. Mutable per-screen state belongs to a screen-lifetime instance or operation object, such as `GemConfirmation`, rather than an app singleton. Feature composables do not read screen services from a `CompositionLocal`.
- **A value type or a namespace of statics** takes the service as a method parameter only when the answer genuinely requires that service's dependencies. A pure receiver-owned answer stays on the receiver according to § 6.

Dependency-free FFI transport adapters are the exception: `GemSimulationFormatter` and `PriceAlertFormatter` may be constructed locally because they have no state to substitute. Do not extend that exception to a service, store, client or a type whose behavior can cross on its honest receiver.

A fieldless Core rule object is the second exception. `GemAssetConfigService` and `GemConnectionService` carry no state, no store and no client, so a module-level lazy instance is not a second instance of anything — there is nothing to substitute and nothing to keep in step. Keep them there only while they back top-level extensions on a primitive that neither a composable nor a constructor can reach (`Chain.asset()`, `AssetId.icon()`, `ConnectionStatus.refreshInterval(kind)`); a caller that already has a view model asks its service instead. The same holds for the iOS `.shared` accessors in `Config.swift`: a leaf value model a view builds from a value — a row, a formatted address, a search predicate — has no constructor the composition root controls, so it reads the fieldless object directly. `ImportWalletTypeViewModel` is that shape: it reads `GemChainService.shared` itself, so its parent vends it with no argument. What is never acceptable is a flow parent reaching for one to hand to a child: the parent takes the child's vendor from the factory, the way `ViewModelFactory` hands `ContactsViewModel` its `ContactEditorViewModel` builder. The composable reading one still [may not call it from its body](#keep-the-crossings-few).

Prefer the [generated abstraction](#depend-on-the-generated-abstraction-not-the-concrete-object) wherever a test needs substitution: any unstubbed method on a mocked concrete UniFFI object can reach a native handle the mock does not have.

The keystore is the one dependency a composition root does not pass around. `GemKeystore` unlocks a wallet's secrets given a password, so an app that holds one can sign without the service that decides whether signing is allowed. Each platform's keystore layer — [`GemstoneServices`](../ios/Packages/GemstoneServices) on iOS, [`data:services:gemstone`](../android/data/services/gemstone) on Android — builds the four services that need it (`GemWalletService`, `GemSwapService`, `GemSignMessageService`, `GemAuthService`) and hands out those, never the keystore. On iOS they come from [`LocalKeystore+Services.swift`](../ios/Packages/GemstoneServices/Sources/Keystore/LocalKeystore+Services.swift) and `gemKeystore` is `package`, so `ios/Gem` cannot name it; on Android they come from [`KeystoreModule`](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/di/KeystoreModule.kt), the only module that injects the Hilt binding. `just check-boundaries` rejects a `GemKeystore` outside those two layers.

### Construction example: price alerts

The shared owner is constructed in [`ServicesFactory.swift`](../ios/Gem/Services/ServicesFactory.swift) and passed through [`ViewModelFactory.swift`](../ios/Gem/Services/ViewModelFactory.swift):

```swift
let gemstonePriceAlertStore = GemstonePriceAlertStore(store: stores.priceAlertStore)
let priceAlertService = Gemstone.GemPriceAlertService(
    api: deviceApiClient,
    preferences: preferencesService,
    store: gemstonePriceAlertStore,
    permissions: notificationPermissions,
)
```

Android's [`PriceAlertsModule`](../android/data/services/gemstone/src/main/kotlin/com/gemwallet/android/data/services/gemstone/di/PriceAlertsModule.kt) binds the adapters, concrete service for Core composition and interface for app consumers:

```kotlin
@Singleton @Provides
fun provideGemstonePriceAlertStore(priceAlertsDao: PriceAlertsDao, priceAlertFormatter: PriceAlertFormatter): GemstonePriceAlertStore =
    GemstonePriceAlertStore(priceAlertsDao, priceAlertFormatter)

@Singleton @Provides
fun provideGemPriceAlertStore(store: GemstonePriceAlertStore): GemPriceAlertStore = store

@Singleton @Provides
fun provideGemPriceAlertService(
    apiClient: GemDeviceApiClient,
    preferencesService: GemPreferencesService,
    store: GemPriceAlertStore,
    notificationPermissions: GemNotificationPermissions,
): GemPriceAlertService = GemPriceAlertService(
    api = apiClient,
    preferences = preferencesService,
    store = store,
    permissions = notificationPermissions,
)

@Provides
fun provideGemPriceAlertServiceInterface(service: GemPriceAlertService): GemPriceAlertServiceInterface = service
```

## 9. Errors

Use the shared [`GemServiceError`](../core/gemstone/src/services/error.rs) for ordinary API, store and service failures. Propagate required failures as typed errors or through the existing step-failure mechanism; only an explicit subsystem policy permits a fallback. Add a feature error enum in `error.rs` only when the app needs structured domain data to render or branch without parsing a message:

```rust
pub enum GemConfirmError {
    ScanMemoRequired { symbol: String },
    BalanceMissing { asset_id: AssetId },
    Sign { error: GemSignerError, chain: Chain, msg: String },
    ...
}
```

When a lower-level error has a canonical feature-level mapping, implement `From` once in `error.rs` and use `?`:

```rust
impl From<GemServiceError> for GemConfirmError {
    fn from(error: GemServiceError) -> Self {
        match error {
            GemServiceError::Cancelled => Self::Cancelled,
            GemServiceError::Offline => Self::Offline,
            error => Self::Load { msg: error.to_string() },
        }
    }
}
```

Use `map_err` only when the call site adds context or deliberately selects a non-default category, such as `Record`, or when a named mapper preserves structured `Offline`/`Network` gateway cases.

The app **localizes Core's error directly** — it does not translate it into a parallel app-side enum first. A duplicate taxonomy costs a mapping function, re-derives data Core already carries, and drifts. Classify Core's error where a screen needs to branch; do not re-wrap it.

### A command names its commit and recovery behavior

State what has been saved when a later step fails, whether the action can be retried, and how reconciliation repairs partial progress. Preserve intentional local-first behavior. A matching preference or unchanged row must not skip an unfinished required effect. Return existing structured outcomes/step failures where available; keep successful work and expose required failures consistently.

Wallet price requests use USD. `GemPriceService` applies the selected fiat rate once and preserves the USD price for later conversion; never pass server-converted prices into that path. The independent widget pricing path may request its display currency directly.

[Price updates](../core/gemstone/src/services/price/mod.rs) use one `GemPriceStore.save_rates(rates, conversion)` transaction: Core selects the optional current-currency conversion, and each adapter commits it with the changed rates. A failed repricing rolls back the rates so an identical-rate retry still applies. [Price-alert commands](../core/gemstone/src/services/price_alert/mod.rs) write locally first: an API refusal rolls the added or deleted row back, and `set_enabled` only stores the preference. Device registration reads that preference when it looks for changes, so an identical toggle has no unfinished device effect to retry. Add a durable queue only if a later recovery requirement needs it.

Cancellation after an irreversible effect does not prove that the effect did not happen. Security exceptions, including scanner fail-open, remain explicit subsystem policy rather than incidental error handling.

`GemAmountError::display` also decides whether an error is shown: zero amount is silent on both apps, and an asset is named `Name (SYMBOL)` unless its name already equals its symbol.

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

Test each domain rule at its Core owner. App tests verify mapping, wiring, navigation and real adapter contracts; meaningful integration coverage is not redundant simply because it crosses Core. Delete an app test when it only restates the same rule through a mock, not merely because a Core change could make it fail.

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

**Reusable mocks live beside their types.** Values and reusable doubles come from the testkit that owns the type — `Type::mock()` in Core, `static func mock` in the iOS `TestKit`, `mockType()` in Android `testFixtures` — rather than a duplicate helper in each test file. Literal inputs, one-off overrides and behavior probes stay in the test; reusable missing shapes extend the existing mock. Placement and naming per platform: [Core tests](../core/skills/tests.md#testkit-mocks), [iOS testing](../ios/skills/testing.md#mocks), [Android testing](../android/skills/testing.md#shared-testkit).

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

For consistency changes, deterministically delay or fail the relevant step and assert request applicability, ordering, partial progress and retry. Paired native tests must exercise actual GRDB/Room transactions for rollback, conditional conflicts, wallet scoping and observer-visible batches; a fake DAO cannot prove atomicity. Reuse the existing testkits and equivalent fixtures (MIG6).

## 11. Landing a change

Follow [Task Workflow](../skills/task-workflow.md) and the [Quality Checks closing matrix](../skills/quality-checks.md#closing-matrix). Implement shared behavior in Core, regenerate when its mobile contract changes, then wire both apps. Internal Core changes do not require regeneration.

A migration is complete when the replaced app logic, forwarding wrappers and obsolete exports are removed, meaningful tests follow their owners, and the applicable checks pass. Search both apps, extensions, nested fields and tests before deleting an export. Delete completed items from [TODO.md](TODO.md) in the implementing commit; commit or publish only when authorized.

### When the platforms disagree

Check the documented contract, callers, and tests: a difference may be intentional, or a test may pin a bug. Explain the evidence for the shared behavior before changing it. Ask only when the product decision remains unresolved; a passing test alone does not choose the contract.

## 12. A client's requests are one enum; the client only sends

**REST clients own a request target; JSON-RPC clients own a request enum.** Keep paths and request-specific metadata in the target. Credentials, transport, envelope handling, and pagination belong in the client. The deliberate exceptions are listed below.

The enum is a § 1 rule for a request: inputs in, wire format out, no transport, no secret, no clock. The two references are [`TronGridTarget`](../core/crates/gem_tron/src/rpc/trongrid/target.rs) for REST (`FooTarget` in `rpc/target.rs`) and [`SolanaRpc`](../core/crates/gem_solana/src/jsonrpc.rs) for JSON-RPC (`FooRpc` in `jsonrpc.rs`, method constants in `method.rs`). For a direct GET/POST without shared client work, use [`AptosClient`](../core/crates/gem_aptos/src/rpc/client.rs); do not add a forwarding `send` helper.

Inspect the [target](../core/crates/gem_tron/src/rpc/trongrid/target.rs) and [client](../core/crates/gem_tron/src/rpc/trongrid/client.rs) together; keep examples linked to their implementation instead of maintaining a second code sample here.

Variant fields are named: `GetAccount(String)` does not say what the string is. No path or query string is a `const`; `path()` builds every one. The target implements `gem_client::Target` (`path()`, `headers()` when a request carries one, and `content_type()` when a body is not JSON); the client owns the transport, the credentials, the clock, the signature, the envelope and the pagination loop. A method is `self.client.get(target).await` or `self.client.post(target, &body).await`. For `gem_client::Target`, do not add a body enum or a second dispatch over GET/POST variants to unify these calls; the device client's `gem_jsonrpc::Target` has a separate contract below. A private helper exists only for shared work: credentials (`TronGridClient::send`), a 404 that is a value (`StellarClient::get_or_not_found`), an envelope (`SnsClient::get_result`). A GET-only host needs only `path()`; `GemDeviceApiTarget` is the full shape with `method()`, `body()` and a signed header.

| Case | Shape | Reference |
|---|---|---|
| Path parameter | `format!` in `path()`. A chain or network segment is a field the target reads; a chain-to-slug map is a pure `fn` with its own test | `TronGridTarget`, Blockscout `/{chain_id}/…`, `alchemy_url` |
| Query string | Part of `path()`; a transport only ever sees a path. One or two fixed parameters are a `format!`; more, or any optional one, is a flat `Serialize` struct rendered by `build_path_with_query` (`None` omitted, values encoded, a slice of pairs for a repeated key). A client without a target yet calls `client.get(path).query(&query)`, which renders the same way | `GemApiTarget`, `CoinMarketsQuery`, Mayan `QuoteQuery` |
| Optional parameter | An `Option` field of the query struct, omitted when `None`, never a second variant | `TransactionsQuery { limit, fingerprint: Option<String> }`, `PaymentsQuery { cursor, .. }` |
| Method and body | The method calls `post(target, &body)` with the body it has; a POST variant carries nothing the path does not need. `Client` speaks GET and POST; the device client builds `gem_jsonrpc::Target` with a `method()` for PUT and DELETE. A host that multiplexes on the body (HyperCore `/info`, Cardano GraphQL) has a constant path and a variant per query | `AptosClient::submit_transaction`, `GemDeviceApiTarget` |
| Raw body | `String` for `text/plain` and form-urlencoded, `Vec<u8>` for binary; the content type from the target's `content_type()`, always a `ContentType` variant, never a `Content-Type` string in `headers()`. A body-carrying request declares `application/json` by default, so only a non-JSON body overrides it | Bitcoin `sendtx`, Stellar, Algorand, Aptos BCS, `SendSupportImage` |
| Credentials | `fn headers(&self)` on the client, empty when the key is blank (`Option<String>` decided once at construction), passed as `.headers(self.headers())`; a key the host wants in the query is appended by `send` the same way. Transport default headers are backend-only: `RpcClient` has none | `JupiterClient`, `NearIntentsClient`, Blockscout `apikey` |
| Request header | On the target: API version, idempotency key | TON emulate |
| Signature | A pure `fn` in `auth.rs` over method, path, body, timestamp and nonce; the client reads the clock and merges the result last. A refreshed token sits behind an injected port | `okx::auth::sign`, `GoPlusProvider::sign`, `build_device_auth_header` |
| Envelope | Unwrapped once in a private helper; a typed error body through `get_or_error::<_, ErrorResponse>`; a 404 that means "none" is a value (§ 3) | SNS `Response`, GoPlus `Response`, Mayan and THORChain `get_or_error`, Stellar `AccountResult` |
| Pagination | The cursor on the variant, the page size a `const`, the loop in the client with a page cap and a repeated-cursor guard; the loop takes a closure that builds the page's variant | `get_transaction_pages`, `AlchemyClient::get_nfts_by_owner` |
| JSON-RPC | `ToJsonRpcRequest` with constants from `method.rs` and typed parameter enums beside it; `batch_request` then `take_all`; the same enum posted to a path when a REST host has an RPC route | `SolanaRpc`, `EthereumRpc`, Chainflip broker |
| Construction | `new(client, key)` with `C: Client` already pointed at the host. Never `ReqwestClient::request` from a client: it bypasses `Client` and never works on the apps | `TronGridClient` |

**Tests.** A client test over `MockClient` or `mock_jsonrpc_client` asserts behaviour the wire shape does not show: an envelope's failure branch, the paths a pagination loop produced, the body and content type of a broadcast, a merged credential header. Do not test `path()` by copying its implementation into the expectation. A wire-contract regression test uses an independently specified request and exercises the real client through its mock transport.

**Deliberate exceptions.** Three things stay on raw `reqwest` because they are not REST clients: the off-chain NFT metadata fetch (an arbitrary HTTPS URL read under a byte cap with redirects off), the image downloader (binary bodies), and the egress node health probe (only the status matters). The OKX client sends the string it signed rather than a target, and the alien reqwest provider is the transport itself.

## 13. Shapes that were tried and reverted

These decisions explain the contracts above; they are not a migration checklist. Follow current subsystem and security contracts when an example here differs. Repo-wide build and tooling choices are in [§ 14](#14-repo-wide-choices-that-are-not-obvious-from-the-code).

### Ownership before wrappers

Forwarding-only services previously added dependencies without owning behavior. Use the [ownership matrix](#6-where-derived-domain-answers-live); a member-count rule does not justify a new service.

Shared child views cannot depend on whichever generated Core service their parent happens to hold. Making generated protocols inherit an app protocol or hiding the service in closures does not solve ownership. The parent computes the answer and passes values to the child. A dependency-free child is the intended result.

When services form a cycle, separate the responsibilities that caused it. Node runtime latency state, user node preferences and chain configuration are three owners; one service holding all three creates both a cycle and app-side fallback logic. Passing one service through another or exposing its internal store preserves the cycle.

### Return decisions and carry whole values

Returning ingredients made the apps independently derive transaction headers, stake actions and reward totals. [Screen records](#3-return-one-record-that-answers-the-whole-question) return those decisions together.

Pass a domain record when it already owns the needed fields. Breaking a delegation into chain, provider, state, and rewards makes app callers supply inconsistent constants, and a subset balance record adds a mapper on both apps for each new rule. Reuse the canonical record without exposing unrelated mutable dependencies.

Input parsing must preserve the source's meaning: human input and machine strings need different parsers. Returning a coarse success flag or rounding before checking precision discards information the next layer needs. See [number parsing](#number-parsing-human-input-vs-machine-strings).

### Presentation choices are decisions

Independent row composition drifted in titles, units, badges and selected values. The [row contract](#a-list-row-is-a-record-of-choices) and [number contract](#a-number-crosses-as-a-value-and-a-style-never-as-a-string-or-a-callback) keep those choices in Core.

A default argument that supplies an app's own answer hides that answer from review. Make the parameter required so every call site names where its answer comes from; the compiler then lists the places that were deciding on their own.

Count the copies before adding one. A rule that looks like two app-side duplicates is often three, with the third inside a Core crate that must agree with the other two. Search the workspace before introducing a shared answer, and make the existing owner the one that answers.

### State and lifecycle have an owner

Shared mutable screen state caused lifetime conflicts. Follow the [construction contract](#8-services-are-injected-never-constructed-at-a-call-site); a retained confirmation belongs to its operation.

Moving a store behind Core must preserve observable write order and failure behavior. Price alerts write locally before pushing to the API; reversing those steps delays the UI and loses the local update on a network failure. A refactor must preserve that behavior unless the task deliberately changes it.

Wallet creation, import, rename, deletion and switching have lifecycle effects beyond a database write. The owning Core workflow defines required setup and activation order, while native observers trigger lifecycle work. Do not split required setup between independent observers or declare an import successful before its required work completes. `GemWalletService::import_wallet` defines that order once (C53).

Operations on the current wallet read the Core session through their owning service. A nullable wallet passed through every app call site causes silent early returns and inconsistent account selection. Preserve explicit wallet identifiers for operations that intentionally target a different wallet; do not turn every operation into an implicit current-session operation.

### Async execution and SDK boundaries

Core futures progress while awaited; Gemstone has no async runtime of its own. Confirmation broadcasts, stores the pending transaction and returns its hashes. The `GemTransactionStatusService` foreign port schedules `GemTransactionStateService::track` off-thread; awaiting that poll would keep confirmation spinning until finality. Core decides what to track; the app supplies a lifetime and executor. Do not assume creating a future starts background work.

The app translates SDK events and forwards them; Core owns shared authorization, chain/account selection, routing, and replies, because two apps that each reimplement those decisions drift. Platform ports remain appropriate for signing, secure storage, authentication, observation, and navigation under the current security contract; removing a dependency must not remove an auth gate.

### Transfers and signed conventions

Apps carry Core-built transfer and action records instead of reconstructing them between screens; reconstruction grows app-specific switches for accounts, recipient fields, direction, max flags, and amounts. Core owns the conversion from a domain action to confirm data; navigation preserves that result.

The signer's convention must be explicit and tested. HyperCore reduce orders interpret direction as the position direction; flipping it builds the wrong reduce-only side. Compare the signer and its vectors before resolving an app disagreement. Simplifying a record must preserve every transaction-critical input.

### FFI has a maintenance cost

After replacing a path, check callers on both apps, including extension wrappers and implicit receiver calls, and match the receiver when methods share a name; what the replaced export then owes is in [no trivial exports](#no-trivial-exports). Platform ports implemented by the apps are different: Core is their caller.

Types and conversions may become redundant after un-exporting a method. Keep `Gem` for the FFI surface and reuse the underlying domain type internally when the wrapper carries no distinct meaning. Do not expose an API solely so a mock can reconstruct the production result.

Use the existing [typed field transport](#field-types) and [generation contract](#ownership-is-not-transport) when removing redundant wrappers.

### Tests follow the rule's owner

App tests that mocked the decision could pass while Core was wrong. Follow [§ 10](#10-tests); test ordering when it is the contract, not merely an implementation detail.

### Chain families remain families

A family member keeps the family `ChainType`. Giving an EVM chain its own type forces exhaustive switches, wallet namespaces, and fee behavior to treat it as a new family. Use `ChainConfig` or the chain crate for actual per-chain differences. Follow the [New Chain Checklist](../core/skills/new-chain-checklist.md) for implementation.

## 14. Repo-wide choices that are not obvious from the code

Understand the rationale before changing one of these. Code-style exceptions belong in the platform skills, not here.

### Gemstone is bundled locally

Gemstone (the Rust-to-mobile bridge) is built and bundled from source rather than fetched as a prebuilt package. This ensures the mobile apps always link against the exact Core revision in the repo and avoids version drift between Core logic and mobile bindings.

### TypeShare + UniFFI for code generation

TypeShare generates shared model types; UniFFI generates FFI bindings. Both run from `just generate`. Two tools are used because TypeShare handles pure data models efficiently while UniFFI handles the full FFI bridge (functions, callbacks, async). Do not consolidate them.

### Android distribution channels come from one matrix

`android/gradle/channels.gradle.kts` defines every channel (google, universal, huawei, solana, samsung, emerald, fdroid) and what each one links: the push module (FCM or stub), the review module (Google or stub), the WalletConnect implementation (Reown or no-op), ProGuard rules, update URL, and ABIs. Channels exist to satisfy different store and partnership constraints, so do not remove one or assume Google-only distribution.

The active channel is resolved once per Gradle invocation by `selectChannel()`: `-Pchannel=<name>` first, then inference from an `assemble<Channel>`/`bundle<Channel>` task name, then `google`. `fdroid` is the only channel that drops Firebase, Google services, and real WalletConnect, and it is always built alone with `-Pchannel=fdroid` (`android/reproducible/fdroid/build.sh`). Add or change a channel by editing the matrix only. Do not reintroduce `System.getenv` feature reads or hand-written product flavors: a second selector has to be hand-synced with the matrix, and an F-Droid build that misses its flag silently links Firebase and WalletConnect, which the F-Droid scanner rejects.

### Rust build cache: kache locally, sccache in CI

Developer machines use kache as the `rustc` wrapper (see `core/skills/setup.md`); CI keeps sccache with the GitHub Actions cache backend (`RUSTC_WRAPPER: sccache` in the workflows, `mozilla-actions/sccache-action` in `.github/actions/setup-rust-ci`). kache dedups content-addressed artifacts across worktrees and clean rebuilds better than sccache does, but CI's cache is independent of any developer machine and kache has no shared remote configured for this repository. The split is deliberate so CI caching cannot regress; switching CI needs a remote store (S3 bucket plus secrets) first. Do not change one side while touching the other.

### Core source lives in this repository

`core/` is tracked source in this repository, not a Git submodule. Changes to Core and the mobile apps should land together when shared behavior, generated models, or bindings need to stay aligned.

### Number parsing: human input vs machine strings

Amount strings come from two sources that must be parsed differently. Confusing them silently corrupts amounts on locales that group thousands with a dot (de, it, es, nl, pt-BR, da), where `"1.234"` means 1234, not 1.234. Pick the parser by the source of the string, never by convenience.

- **Human input** (text a person typed into a field) is parsed by Core. `GemNumberFormat` (`core/gemstone/src/services/amount/model.rs`) carries the device's decimal separator, and its `plain` and `value` methods own the separator, grouping, leading-zero and Unicode-digit rules (`plain_number` and `value_from_input` in the amount rules). The apps pass the text and the device's decimal separator and nothing else: iOS `NumberInput.plain/.double/.value` (`ios/Packages/GemstonePrimitives/Sources/NumberInput.swift`), Android `String.plainInputNumber()` / `parseInputNumber()` (`android/gemcore/src/main/kotlin/com/gemwallet/android/math/NumberParser.kt`). A typed value that feeds a Core rule goes to that rule as text with the number format, and the rule parses it (`GemCustomFee::estimate` takes the typed rate), so no app holds a parse whose failure it has to swallow. Never read typed text with `Decimal(string:)`, `Double(_:)` or `BigDecimal(_)`: those miss grouping separators and non-Latin digits, so an Arabic or Persian keyboard reads as no amount at all and `"1.234"` in a dot-grouping locale reads 1000x too low. Text with no digit normalizes to an empty string, which every caller treats as no amount.
- **Machine strings** (QR/payment-link amounts, API/exchange payloads, anything the app did not get from a keyboard) never reach the input parser: Core decodes them and hands the apps typed values. If one has to be read app-side, parse it locale-independently — a machine string always uses `.` as the decimal point.
- **Writing into a field.** Text in an editable number field is human input the moment it lands there, so whatever the app puts into one — a payment-link amount, the max, a suggestion — is written in the device's format. Core hands over a number (an atomic `GemBigInt`, as `GemAmountInput.prefill` carries, or an `f64`), and the app renders it with `GemNumberFormat`'s `input_text` or `value_text` (iOS `NumberInput.format()`, Android `numberFormat()`). A machine string never goes into a field as is, and Core returns text meant for a field only when it took the separator (`GemAutocloseSession::input_text`). A `"0.001"` placed in a comma-decimal field reads back as 1: #727 fixed the scan-to-confirm parse, but the amount-screen prefill wrote the raw string until it crossed as a number.
- **Amounts Core formats for notifications** (`number_formatter::ValueFormatter`, `ValueStyle::Auto`, used by the daemon pusher, the staking rewards notifier and in-app notifications) keep two decimals above one and four significant digits below it, dust included: a push title has no room for `0.000040036032429186 ETH` (issue #1155), so it reads `0.00004003 ETH`. The app list formatters keep full precision for dust on purpose; that is a screen with room, not a title.

### iOS localization compiles to String Catalogs, accessors are generated in Core

The localization generator writes one `.xcstrings` String Catalog per table (app, InfoPlist, widget) instead of per-language `.lproj/*.strings` files, and generates the typed `Localized.swift`/`WidgetLocalized.swift` accessors itself. SwiftGen is not used. Two constraints drove this:

- Xcode 26 can generate catalog symbols natively, but they are internal to the owning target; `Localization` is a shared SPM package consumed across the app, so the public accessor surface must be generated by our tooling.
- The historical dotted iOS keys (`common.cancel`) cannot be derived from the underscore Fluent IDs (`common_maximum_value` vs `common.maximum_value`); the committed `Localizable.xcstrings` is the source of that mapping, which the generator reads back on every run. Do not delete the catalogs and regenerate from scratch — the dotted key mapping (and with it the `Localized.*` API) would be lost.

## Service map

### Screen services

The table locates the existing owners and consumers; it is not proof that a screen has completed migration. The session column names selected [state records](#a-screen-whose-state-changes-is-a-session); a dash means no session is recorded here. [TODO.md](TODO.md#screen-coverage-and-existing-infrastructure) records remaining work. Check both callers before changing a holder: pure value models, native observed reads, platform ports and parent-to-child dependencies are intentional, and a one-sided call is only an audit lead.

| Core service | Session | iOS | Android |
| --- | --- | --- | --- |
| `GemAddAssetService` | — | `AddAssetSceneViewModel` | `AddAssetViewModel` |
| `GemAddressDetailsService` | — | `AddressDetailsSceneViewModel` | `AddressDetailsViewModel` |
| `GemAmountService` | — | `AmountSceneViewModel` and its providers | `AmountViewModel`, `AmountPerpetualProvider` |
| `GemAppUpdateService` | — | `AboutUsViewModel` | `AppUpdateCoordinator` (adds the Play vs universal-APK delivery channel) |
| `GemAssetDetailsService` | — | `AssetSceneViewModel` | `AssetDetailsViewModel` |
| `GemAssetSelectionService` | — | `SelectAssetViewModel`, `WalletSearchSceneViewModel`, `AssetsResultsSceneViewModel` | `BaseAssetSelectViewModel` and its subclasses |
| `GemChainService` | — | `ChainListSettingsViewModel` (chain picker) | `ContactChainSelectViewModel`, `SelectImportTypeViewModel`, `AddAssetViewModel` |
| `GemChainSettingsService` | — | `ChainSettingsSceneViewModel`, `AddNodeSceneViewModel` | `NetworksViewModel`, `AddNodeViewModel` |
| `GemChartService` | `GemChartSession` | `ChartSceneViewModel` | `ChartViewModel` |
| `GemCollectibleService` | — | `CollectibleViewModel`, `ReportNftViewModel` | `NftDetailsViewModel` (+ `GetNftAssetDetails` observed read) |
| `GemConfirmTransferService` | `GemConfirmation` (one confirmation in flight; it loads and executes, so it is not a session) | `ConfirmTransferSceneViewModel` (holds the `GemConfirmation` the factory opens) | `ConfirmViewModel` |
| `GemContactService` | — | `ContactsViewModel` | `ContactsViewModel` |
| `GemCurrencyService` | — | `CurrencySceneViewModel` | `CurrenciesViewModel` (+ session currency cases) |
| `GemDeviceService` | — | `RootSceneViewModel`, `AppLifecycleService`, `CurrencySceneViewModel` | `DeviceObserverService`, `DevicePushSettings` |
| `GemDeveloperService` | — | `DeveloperViewModel` | `DevelopViewModel` |
| `GemFiatQuoteService` | `GemFiatSession` | `FiatSceneViewModel` | `FiatViewModel` |
| `GemContactEditorService` | — | `ContactEditorViewModel` (+ `nameService`) | `ContactEditorViewModel` (+ `GemNameServiceInterface`) |
| `GemNftService` | — | `CollectionsViewModel` | `NftListViewModels` |
| `GemNotificationService` | — | `InAppNotificationsViewModel` | `InAppNotificationsViewModel` |
| `GemNotificationsService` | — | `NotificationsViewModel` | `DevicePushSettings` (the push cases `SettingsViewModel` calls) |
| `GemPerpetualDetailsService` | — | `PerpetualSceneViewModel` | `PerpetualDetailsViewModel` |
| `GemPerpetualService` | — | `PerpetualsSceneViewModel` (+ recent activity) | `PerpetualMarketViewModel` (+ recent activity) |
| `GemPortfolioService` | — | `PortfolioSceneViewModel` | `PortfolioChartViewModel` |
| `GemPriceAlertService` | — | `PriceAlertsSceneViewModel`, `SetPriceAlertViewModel` | `PriceAlertViewModel`, `PriceAlertTargetViewModel` |
| `GemReceiveService` | — | `ReceiveViewModel` | `ReceiveViewModel` |
| `GemRecentActivityService` | — | `RecentsSceneViewModel`, and `RecentAssetsModel` vended by `SelectAssetViewModel` and `PerpetualsSceneViewModel` | `RecentsSheetViewModel` |
| `GemRecipientService` | — | `RecipientSceneViewModel` (+ `nameService`) | `RecipientViewModel` (+ `GemNameServiceInterface`) |
| `GemRewardsService` | — | `RewardsViewModel`, `CreateRewardsCodeViewModel`, `RedeemRewardsCodeViewModel` | `ReferralViewModel` |
| `GemServiceStatus` | — | `ServiceStatusViewModel` | `ServiceStatusViewModel` |
| `GemSettingsService` | — | `SettingsViewModel`, `PreferencesViewModel`, `SecurityViewModel` | `SettingsViewModel`, `PreferencesViewModel`, `SecurityViewModel` |
| `GemSignMessageService` | — | `SignMessageSceneViewModel` | `WCRequestViewModel` |
| `GemStakeService` | — | `StakeSceneViewModel`, `DelegationSceneViewModel`, `EarnSceneViewModel` | `StakeViewModel`, `DelegationViewModel`, `EarnViewModel` |
| `GemSupportService` | — | `SupportChatSceneViewModel` | `SupportChatSceneViewModel` |
| `GemSwapQuoteService` | `GemSwapSession` | `SwapSceneViewModel` | `SwapViewModel` |
| `GemTransactionDetailsService` | — | `TransactionSceneViewModel` | `GetTransactionDetailsImpl` (observed read + links) |
| `GemTransactionsService` | — | `TransactionsViewModel` | `TransactionsViewModel` |
| `GemWalletConnectService` | — | `WalletConnectorService`, `ConnectionsViewModel` | `WCRequestViewModel`, `ProposalSceneViewModel`, `WCAuthViewModel`, `ConnectionsViewModel`, `ConnectionViewModel` |
| `GemWalletHomeService` | — | `WalletSceneViewModel`, `NetworkAssetsSceneViewModel` | `AssetsViewModel`, `NetworkAssetsViewModel` |
| `GemWalletService` | — | onboarding and manage-wallet view models, and `WalletImageViewModel` for the avatar (`WalletDetailViewModel` exports the secret through `export_secret`) | `CreateWalletViewModel`, `ImportViewModel`, `WalletsViewModel`, `WalletViewModel` (`rename`), `WalletSecretDataViewModel` (`export_secret`), `WalletImageViewModel`, wallet cases |
| `GemWalletSessionService` | — | `RootSceneViewModel`, `NavigationRouter` | `SessionCoordinator` (+ the services it composes) |
| `GemWidgetService` | — | — (the iOS widget never links Gemstone; see [the iOS project overview](../ios/skills/project-overview.md)) | `WidgetCoinUIModel` and `WidgetPriceSyncWorker` through `WidgetEntryPoint` |

### Composition and lifecycle services

These primarily serve Core composition or native lifecycle integration. Reuse them through the owning screen service where they supply a domain answer. Dependency-free rule objects, launch hosts and explicit platform/component ports retain [§ 7](#7-at-most-one-core-service-on-ios-narrow-cases-on-android)'s exceptions; judge the responsibility and actual calls rather than the number of fields.

| Core service | Held by |
| --- | --- |
| `GemBalanceService` | composed by `app_start`, `asset_discovery`, `assets`, `confirm`, `fiat`, `perpetual`, `receive`, `rewards`, `stream`, `swap`, `transaction_state`, `wallet_home` |
| `GemAssetsService` | composed by `app_start`, `assets`, `balance`, `confirm`, `fiat`, `receive`, `search`, `transaction_state`, `transactions`, `wallet_connect` |
| `GemExplorerService` | composed by `address_details`, `assets`, `chart`, `confirm`, `nft`, `node`, `stake`, `transactions`, `wallet`, `wallet_connect` |
| `GemPriceService` | composed by `assets`, `chart`, `confirm`, `currency`, `perpetual`, `portfolio`, `search`, `stream` |
| `GemStreamSubscriptionService` | composed by `assets`, `balance`, `stream`, `swap` |
| `GemSwapService` | composed by `assets` and `swap` |
| `GemSimulationService` | composed by `confirm` and `wallet_connect` |
| `GemScanService` | composed by `confirm` |
| `GemSearchService` | composed by `assets` |
| `GemFiatService` | composed by `fiat` and `stream` |
| `GemAssetDiscoveryService` | composed by `wallet_home` |
| `GemAvatarService` | composed by `nft` and `wallet` |
| `GemBannerService` | composed by `app_start`, `assets` and `wallet_home` |
| `GemAuthService` | composed by `rewards` |
| `GemConfigService` | composed by `app_start` and `app_update` |
| `GemWalletConfigurationService` | composed by `app_start` |
| `GemDeviceKeyService` | composed by `auth` and the device signer |
| `GemSubscriptionService` | composed by `device` |
| `GemAppStartService` | iOS `OnstartService`, Android `MainViewModel` — launch orchestration, not a screen |
| `GemConnectionService` | iOS `ConnectionStatusObserver`, Android `RefreshInterval` |
| `GemPerpetualStreamService` | `HyperliquidObserverService` on both apps |
| `GemPushNotificationService` | iOS `NavigationRouter`, Android notification routing |
| `GemTransactionStateService` | composed by `confirm`; tracked off-thread by the `TransactionStatusService` port on both apps |
| `GemSecurityService` | iOS `BiometryAuthenticationService` (used by `LockSceneViewModel`), Android `LockTimer` |

`GemNameService` is not a row in this table: it is the [shared-component dependency](#a-service-never-hands-out-another-service) that `AddressInputViewModel` and `NameRecordViewModel` take, and a parent passes it down beside its own service.

These searches locate possible concrete consumers or multiple owners. Inspect the calls before changing anything; exclude composition roots, previews, pure rule objects and dependencies passed only to children. A hit alone does not justify a wrapper.

```
rg -o "(let|var) \w+: (any )?Gem\w+Service(Protocol)?" ios/Features --glob '*ViewModel.swift'   # >1 per file, or no Protocol
rg -o "val \w+: Gem\w+Service\b" android/features --glob '*ViewModel*.kt'                       # concrete class
```

### App services

Native hosts and adapters retain platform work while delegating shared behavior to Core:

| Service | Notes |
| --- | --- |
| [`AppService/RateService`](../ios/Packages/FeatureServices/AppService/RateService.swift) | App Store review prompt |
| [`AppService/AppLifecycleService`](../ios/Packages/FeatureServices/AppService/AppLifecycleService.swift) | Scene phase orchestration of observers |
| [`AppService/OnstartService`](../ios/Packages/FeatureServices/AppService/OnstartService.swift) | OS security checks, URL cache and launch orchestration |
| [`ConnectionStatusService`](../ios/Packages/FeatureServices/ConnectionStatusService) | Connectivity |
| [`StreamService`](../ios/Packages/FeatureServices/StreamService) | Native socket observation and cancellation; `GemStreamService` owns session preparation, subscriptions, and currency selection on both apps |
| [`WalletConnectorService`](../ios/Packages/FeatureServices/WalletConnectorService) | Reown/WalletConnect SDK integration |
| [`SystemServices`](../ios/Packages/SystemServices) | Connectivity, image gallery, local store |

Both apps consume the same Core decisions, but may reach them through different integration surfaces. The exceptions below explain those differences.

### Intentional platform differences and compatibility

These choices explain apparent parity gaps. They do not authorize copying shared decisions into the apps. Remaining migrations live in [TODO.md](TODO.md).

| Area | Contract |
|---|---|
| Authentication | Privacy lock is iOS-only; WalletConnect one-click auth is Android-only. Android gates secret reads at each call site, while iOS gates the secret read itself. A new Android caller must request authentication. Wallet auth uses the Ethereum signature scheme (`AUTH_CHAIN`) on every chain; rejecting other schemes is intentional. |
| Autoclose | One app enables confirmation on a pending change and displays validation after tapping; the other enables only a buildable change. Both consume the same Core outcome from `GemAutocloseSession`, including the Android open-position sheet through `AmountPerpetualProvider`. |
| Refresh | Wallet home receives socket prices and refreshes on pull; it intentionally has no interval timer. Socket reconnect delay is capped at 30 seconds. `debugLog` and stream diagnostic logging compile out in release. |
| One-sided features | iOS support-image previews use `image_file`; Android uses post-search `sync_assets` and invalid-mnemonic highlighting. Developer tools may differ (`deeplink_url` on iOS, `platform_store` on Android). Add the counterpart only when the feature is required. |
| Equivalent integration | Both apps choose the collectible receive network through `GemSelectAssetType::ReceiveCollection`. Payment prefills reach iOS through `GemAmountTransfer::prefilled_amount` and Android through Core-built `GemRecipientNext::Amount` carried in navigation. Perpetual banners use native navigation on each app; both observable preference adapters call `GemPreferencesService.set_perpetual_enabled`. |
| Platform authentication outcomes | iOS uses `GemAuthPromptOutcome.is_cancelled`; Android uses `retry_delay_milliseconds`. They ask different questions of the same Core result. |
| Notifications | Android's adapter retains application context for permission status and system settings (`FLAG_ACTIVITY_NEW_TASK`); the permission request itself runs through the activity collector. |
| Hidden features | Earn exists on both apps behind `EARN_OFFERED` in [`config/stake.rs`](../core/gemstone/src/config/stake.rs). Its flag-disabled screens and services remain live code. |
| Compatibility cleanup | Keep Android's config-store auth fallback in `TinkGemPreferences` and the dated iOS `FileMigrator` moves until install-base evidence allows removal. The singular devices transaction route is dated after 2026-11-15; the bare `zh` locale supports installed clients. A date alone is not proof that removal is safe. |
| Build and styling exceptions | iOS styles swap-again only on iOS 26; its Gemstone package retains Swift 5 language mode until `GemstoneFFI` is Swift 6 clean. Android disables selected lint tasks for UniFFI-generated Kotlin. |
| Stream diagnostics | Status sections, concurrent endpoint checks and latency outcomes come from `GemServiceStatus`. The existing `GemStreamConnection` port supplies iOS ping/pong timing and Android’s latest WebSocket upgrade timing (updated on reconnect); native apps render shared latency rows. |
| Provider limitation | The TON verified-collection allowlist stays hardcoded until an authoritative source is available. |

Different export usage is not evidence of duplicated policy. Android already gets fee assets and swap quotes from Core records, observes the current wallet through its session store and checks releases through `check`. iOS receives rejection errors from `process_request`, projects `AssetBasic` from an existing `AssetFull`, and reads connection status through its component extension; only Android needs `chain_from_caip2`. Check the actual path before adding calls for symmetry.

A test, a mock or a preview is not a caller: an export only they read is trimmed, and the fixture is built literally or from the list form (`just check-ffi` reports the rest). The exception is a test double that has to answer as Core does, such as the confirmation doubles that ask `GemConfirmScreen` for their button; the check names those. Nested fields are callers too, and the keystore `preview_import` fixtures must migrate under X172 before its export is removed.
