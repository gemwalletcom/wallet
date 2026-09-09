# Feature architecture

Use the relevant sections when changing shared feature behavior or its app integration. These are current ownership contracts, not a mandate to migrate adjacent code. Resolve disagreements with source using [guidance precedence](../AGENTS.md#using-the-guidance).

## Find the Relevant Contract and Example

Read the contract and the named implementation, then the actual owner and callers being changed. Examples illustrate the named responsibility, not every convention in their file; do not copy unrelated scaffolding or existing violations.

| Task | Contract | Source to inspect |
|---|---|---|
| Pure feature rule | [§ 1](#1-rules-are-pure-and-have-a-test-that-flips), [§ 6](#6-where-derived-domain-answers-live) | [`price_alert/rules.rs`](../core/gemstone/src/services/price_alert/rules.rs), including its tests |
| Service orchestration and store port | [§ 2](#2-the-service-orchestrates-it-owns-its-store-and-depends-on-services), [§ 4](#4-the-store-trait-is-the-apps-only-persistence-obligation) | [`price_alert/mod.rs`](../core/gemstone/src/services/price_alert/mod.rs), [`store.rs`](../core/gemstone/src/services/price_alert/store.rs) |
| App mapping, dependency ownership, or construction | [§ 5](#5-the-app-maps-it-does-not-decide), [§ 7](#7-at-most-one-core-service-on-ios-narrow-cases-on-android), [§ 8](#8-services-are-injected-never-constructed-at-a-call-site) | The changed screen's view model and its factory/Hilt provider; follow the examples in those sections |
| Loading UI | Shared [reuse rule](../skills/engineering-principles.md#clean-code-principles) | Current screen state first; [`LoadingView.swift`](../ios/Packages/Components/Sources/LoadingView.swift), [`LoadingScene.kt`](../android/ui/src/main/kotlin/com/gemwallet/android/ui/components/screen/LoadingScene.kt) |
| REST or JSON-RPC client | [§ 12](#12-a-clients-requests-are-one-enum-the-client-only-sends) | [`AptosClient`](../core/crates/gem_aptos/src/rpc/client.rs) for direct sends, [`TronGridClient`](../core/crates/gem_tron/src/rpc/trongrid/client.rs) for shared credentials, [`SolanaRpc`](../core/crates/gem_solana/src/jsonrpc.rs) for RPC |
| Tests and fixtures | [§ 10](#10-tests) and the platform testing guide | The owner's existing tests, [`gem_client/testkit.rs`](../core/crates/gem_client/src/testkit.rs) for wire behavior |

Read [§ 13](#13-shapes-that-were-tried-and-reverted) only when the task needs the rationale for a rejected design. Subsystem-specific contracts remain in their own documents.

## The one rule

**Core decides. The apps ask, render, and store.**

A rule is anything that could produce a different answer on one platform than the other: a filter, a threshold, a gate, an ordering, a mapping from state to what the user sees. If iOS and Android could ever disagree about it, it belongs in Core.

Everything else is platform work: rendering, navigation, observation, secure storage, keychain and biometrics, and the SQL that stores rows.

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

A rule is a function or receiver method that takes values and returns an answer. It performs no
I/O, holds no service dependency and does not read the clock. Pass time in as a value when it is
part of the decision. Pure does not mean publicly exported: use the narrowest Rust visibility and
shape the FFI-facing API according to § 6.

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

Its test follows Core's `test_<function_name>` convention, covers the meaningful cases together,
and fails if the rule flips:

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

A service composes rules with I/O. It may hold its own feature store and narrow platform ports.
For another domain, it depends on that domain's service, never its store — a store belongs to one
owner, and reaching around that owner creates a second read path it cannot see.

Inspect the dependency graph before replacing a foreign-domain store with its service. Never
introduce an `Arc` cycle to satisfy this rule; split out a narrow query service, invert the
dependency, or redesign the ownership boundary first. `GemWalletService` already depends on
`GemWalletSessionService`, so making the session service depend back on the wallet service would
be worse than the store debt it replaces.

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

`GemConfirmError` implements `From<GemServiceError>` once in `error.rs`, so the three reads use
`?` without repeating the same `Load` conversion.

The method is thin: gather inputs, call the rule, return. Product or domain-decision branching
belongs in `rules.rs`; I/O sequencing, error propagation and empty-work short circuits may remain
in the service.

**Point reads should be synchronous.** `GemWalletStore.get_wallet` is a sync trait method, so
`GemWalletSessionService` answers a session lookup without `await`. Do the same for any single-row
read — an `async` point read pushes the caller back to the store, which is how the confirm screen
ended up reading `AssetStore` directly for two years.

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

The error is the same `GemConfirmError` every other confirm failure uses, carrying the `Asset`
it names and the required/available values, so the app renders it the same way whether it came
from the preload or the send.

**State that travels together is one type.** An approval is either an exact amount or unlimited — never a string plus a boolean the caller has to reassemble:

```rust
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemApprovalValue {
    Exact { value: GemBigUint },
    Unlimited,
}
```

### Field types

- Big-integer atomic quantities are `GemBigInt` / `GemBigUint`, never `String`. `String` moves the parse to every call site, and each one invents its own failure behaviour. The bindings type them too (`core/gemstone/uniffi.toml`): Kotlin sees `java.math.BigInteger`, Swift sees `BigInt` / `BigUInt`, so an app never parses a Core value and never `.toString()`s one to hand it back. The only parses left on the apps are of typeshare models and database columns, which are strings by generation.
- `amount` is for `f64`. `value` is for big integers. Do not mix them.
- Full domain words, per the shared [naming rule](../skills/engineering-principles.md#clean-code-principles).

## 4. The store trait is the app's only persistence obligation

Core declares what it needs; each app implements it over its own database. Nothing else about the
app's storage crosses the boundary. Apps may also implement narrow foreign ports for OS-only
capabilities such as secure storage, notifications and sockets.

```rust
// services/<feature>/store.rs
#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemPerpetualStore: Send + Sync {
    async fn save_perpetuals(&self, data: Vec<PerpetualData>) -> Result<(), GemServiceError>;
    async fn get_positions(&self, wallet_id: WalletId, provider: PerpetualProvider) -> Result<Vec<PerpetualPosition>, GemServiceError>;
}
```

An adapter maps reads and writes and nothing more — **no rules or mapping implementation inline**.
Calling a named mapper or standard boundary conversion is expected; non-trivial mapping lives in a
mapper file beside the adapter (`StoreModels.kt`, `nft/NftModels.kt`).

**Stores only write rows whose values differ.** A blanket write churns observers and hides real changes.

## 5. The app maps; it does not decide

### iOS

There is no app-side service wrapping a Core service. The view model holds its screen's Core
service and calls it; each call is one line in, one mapping out.

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

### Never call Core from the main thread

The `flowOn` above is not decoration. A synchronous Core call such as
`transactionDetailsService.detailRows` can read store callbacks that block on Room, and UniFFI
polls the Rust future on the calling thread — so without it the read lands on main, where Room
throws before any work happens.

The coordinator dispatches; it does not leave that to its caller. Whether a Core method touches a
store is Core's business and can change without the call site noticing.

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

The app uses Core's types. It does not declare a parallel record or enum of the same shape — that
is two definitions of one thing, and every crossing pays for a two-way mapper. `TransferDataType`
was that copy: eleven restated cases and 138 lines of mapping. It is deleted.

Choose the home from ownership first, then decide how it crosses FFI:

| Question | Home | Example |
|---|---|---|
| A field the value already carries, but the generated shape lacks a common accessor | thin app mapping extension | `inputType.asset` |
| A pure answer has one honest domain receiver; additional value arguments are allowed | method on the receiver | `bannerKey.identifier()`, `input.addAddress(addresses)` |
| A pure rule has no honest receiver | private rule called by the owning service | `selectable_fee_assets(...)` |
| The answer requires I/O, stored dependencies or platform ports | method on the service that owns the flow | `confirmService.preload(...)` |
| An app value must be encoded into a Core case | app mapping extension | `.stake(asset, stakeType)` |

**Never add a free exported function or a service wrapper — stateless or not — for an answer
already owned by one local Core type.** `transaction_input_asset(input_type)` and
`transferService.asset(inputType:)` both hide the natural receiver; use
`inputType.transactionAsset()`. A method that ignores `self` is the same mistake even on a
service with real dependencies: `confirmTransferService.simulationAssetIds(simulation:)` is a
property of `SimulationResult`, not of an eight-dependency orchestrator. Keep the owning service
when the rule performs I/O, holds real dependencies, or combines inputs without a single honest
receiver. A request record is an honest receiver when it contains the complete instruction:
`GemContactAddressInput.add_address(addresses)` owns its replacement identifier and new address;
`GemManageContactService.add_address(addresses, input)` would ignore every service dependency.

Do not manufacture a receiver by choosing the first parameter. The type is honest only when the
answer is part of that type's meaning, the method uses `self`, and extra arguments are plain input
values rather than stores, clients or services. For a repository-owned Rust type, prefer an
inherent `impl Type`; do not create a one-method `TypeExt` trait to imitate Swift or Kotlin.
Intrinsic structure belongs in the defining crate (`SimulationResult.asset_ids()`), while
feature or product policy remains in Gemstone even when it consumes a primitives type.

### Ownership is not transport

A receiver method on a Gemstone-local UniFFI record or enum can cross FFI. Export it only when an
app calls it. A TypeShare type defined in a repository-owned Core crate can own canonical inherent
Rust behavior there, but TypeShare does not generate that method in Swift or Kotlin. A type merely
declared through `#[uniffi::remote]` is defined elsewhere, so Gemstone cannot add an inherent
receiver; use a private rule or adapter rather than inventing one. A thin app extension may expose
a structural field projection when transport omits it, but it must not copy product policy.

When mobile needs behavior that cannot cross on its honest receiver, prefer folding the answer
into an existing aggregate operation. If the UI genuinely needs a standalone pure projection,
add it to an existing cohesive FFI adapter such as `GemSimulationFormatter`; do not put it on an
I/O service whose dependencies it ignores and do not create a one-method object. This is how a
TypeShare-only `SimulationResult.asset_ids()` should reach mobile without duplicating the rule.

A rule that answers for one value is a constructor on that value, not a method on an object with
nothing in it: `GemSwapQuoteSummary::new(quote)` carries the minimum receive and the ETA of a
quote, `GemSwapValue::price_impact(receive)` compares two priced amounts, `GemCustomFee::estimate`
and `GemTransactionRow::new` do the same for fees and rows. The apps and their tests construct
the value; nothing has to be mocked to reach a rule. `GemSwapQuoteService` carried those rules as
a `new()` with no fields until it became the swap screen's real service — swap, balances,
preferences and the price stream behind one object — and the rules moved onto their values.

A stateless exported object is acceptable only as a cohesive FFI codec or formatter when UniFFI
cannot express an honest receiver or the operation spans several transport types. Name that role
explicitly, give it no I/O dependencies, and delegate intrinsic behavior or feature policy to the
owning receiver or private rule where possible. `GemSimulationFormatter` and
`PriceAlertFormatter` are the current transport adapters. A one-call forwarding object is still a
wrapper and should be removed.

`#[uniffi::export]` on an `impl` processes every function in that block regardless of Rust
visibility. `pub(crate)` does not remove a method from generated bindings. Put only intended FFI
methods in the exported block; move helpers to a separate unannotated `impl` and give them the
narrowest Rust visibility. Derive `uniffi::Record`/`uniffi::Enum` only for types that actually cross
FFI. After changing an exported member or type, regenerate bindings and build both apps — one
platform may never have called the method the other still needs.

Encoding members are scaffolding, not a pattern to copy. `core/bin/generate/remote_types.yml`
lists what `just generate-models` maps: `remote` types get `#[uniffi::remote]` and structural
mappers on both apps; `codes` are string-backed enums that cross as their code and get
`Primitives.X(core:)` / `.rawValue` on iOS and `toX()` / `toGem()` on Android; `identifiers` are
hand-written parsers the record mappers call by convention (`X(core:)` / `.identifier` on iOS,
the `X(identifier)` constructor / `toIdentifier()` on Android). Before adding a `remote` type,
verify that the generator can represent its full shape and inspect both generated mappers. It
handles fieldless enums and records whose fields are scalars, `DateTime<Utc>`, other remote
types, codes or identifiers, plain or wrapped in `Option` / `Vec`, whatever subdirectory of
`primitives/src` declares them; a `#[typeshare(skip)]` field
travels Core → app only and is filled with its empty value on the way back (scalars, `Option`,
`Vec` and `String` have one; anything else fails generation). `StakeType` has data-carrying
variants, so adding it mechanically would produce an incomplete mapping. Keep a single JSON bridge at the
boundary until the generator supports the type; never add a second app-side model or copy policy
to avoid that bridge.

## 7. At most one Core service on iOS; narrow cases on Android

An iOS view model holds **at most one** Core service, named `service`, and it is **`private`**;
a model that does not need Core holds none. Reuse the owning domain service when it already answers
the screen. Add a screen-level service only when it genuinely composes collaborators or returns a
cohesive screen result — never to satisfy a field-count rule. An Android view model holds the
same Core service through its generated `GemFooServiceInterface` (`private val service`), plus the
observed reads the screen watches as narrow application cases (a Room `Flow` behind
`GetPriceAlerts`, `GetRecentAssets`, `SelectSearch`) and `GetSession`. A case that only forwards a
Core call (`SetPriceAlertsEnabled` over `set_enabled`, `SearchCustomToken` over
`ensure_token_asset`) is migration debt: delete it and call the service. A non-private service on
iOS usually means the view is reaching through the model for a dependency.

This limit does not count explicit platform ports such as a signer, keystore, observation source
or navigation builder. Those remain narrow injected dependencies; they do not decide shared
product behavior.

When a real screen-level service is needed, name it for the screen it backs, not for the layer:
`GemManageContactService` backs the add-and-edit screen. No `Scene` or `Facade` in the name. A
`GemContactsService` that only forwarded calls to `GemContactService` was wrapper debt, not the
pattern, and is deleted: the list screen holds the owning `GemContactService`. When a screen needs
a cohesive answer from several Core owners, Core composes them:

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

The pure address-list transformation stays on its request value:
`input.add_address(addresses)`. It does not belong on this service because it uses none of the
service's dependencies.

A sheet belongs to the screen that presents it and shares that screen's service. A screen you navigate *to* is a different screen with its own.

### Composition services are reached through the screen service

`GemExplorerService`, `GemDeeplinkService`, `GemSwapService`, `GemAssetConfigService`,
`GemPriceService` are *composition* services: screen services hold them, and a screen reads their
answers through its own service — the chart's token link is `GemChartService::token_url`, the
confirm screen's sender link is `GemConfirmTransferService::address_url`, the asset scene's share
link is `GemAssetDetailsService::deeplink_url`, its swap pair `swap_pair`, the confirm sheet's
acquire flow `acquire_asset_flow`. A composition service's method is exported only while an app
still calls it; once every screen reads it through its screen service, move the method to a plain
`impl` block (the constructor stays exported because the composition root builds the object).
The September 2026 sweep found the same answer reached three ways — iOS through the screen
service, Android through the composition service from a Hilt-injected coordinator, and Android's
Compose layer through a `CompositionLocal` — and each pair disagreed somewhere (the slippage
default, the acquire-flow title). One route per answer, and it is the screen service's.

On Android the Hilt module binds both the concrete class and the generated interface
(`fun provideGemFooServiceInterface(service: GemFooService): GemFooServiceInterface = service`):
Core constructors need the concrete type to compose, view models and coordinators take the
interface.

### A service never hands out another service

`service.manageContact()` is the same reach-through as `model.nameService`, one level down: the
caller now depends on something it was not given. Every service is constructed in the composition
root and injected. Returning `Arc<GemFooService>` from an exported service is migration debt, not
an exception to this rule.

A **shared component** — `AddressInputViewModel`, `NetworkSelectorViewModel` — takes the Core
service it needs by its own protocol: `AddressInputViewModel` and `NameRecordViewModel` take
`any GemNameServiceProtocol` (`GemNameServiceInterface` on Android), and the parent view model
receives that `nameService` as a plain constructor dependency beside its `service` and passes it
down. The screen service does not forward name methods and the client does not declare a
protocol intersection (`any GemFooServiceProtocol & AddressInputResolving`) or a builder closure
to reach the component's dependency — both hide a second dependency inside the first.
`NetworkSelectorViewModel` needs only the dependency-free `GemChainService` and builds it itself.

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

The same applies to state: a view switching on the model's `mode` forces `mode` to be non-private. Name the decision on the model instead — `var rowAction: RowAction` — and the view switches on the answer, not the input.

### Depend on the generated abstraction, not the concrete object

On iOS, UniFFI generates a protocol for every exported object. `GemAddressServiceProtocol` exists;
importing `class Gemstone.GemAddressService` at a consumer means that consumer cannot be
substituted without relying on UniFFI's fragile no-handle test path. On Android the same holds
for the generated `GemFooServiceInterface`: bind it in the Hilt module
(`): GemReceiveServiceInterface = GemReceiveService(...)`) and inject the interface.

- **iOS consumers** (view models, components, validators) take `any GemFooServiceProtocol`.
- **Android consumers** take the generated interface, or the observed-read case, used by their layer.
- **The composition root** (`ServicesFactory`, `ViewModelFactory`) holds the concrete type — a UniFFI constructor needs it, and the root is the one place allowed to construct. Its fields are grouped by what they are: Core services, platform services, stores.

## 8. Services are injected, never constructed at a call site

A `GemFooService()` in a field initialiser or at file scope is a second instance the graph does not know about, and it is where an app-side variant creeps back in.

- **iOS** — an owner (a service with a store, a client, a stream, or anything the app needs from
  launch) is registered in `ServicesFactory`, exposed through an `@Entry` in
  `ios/Gem/Types/Environment.swift`, and passed into the view model. A screen service — one that
  only composes owners for a single screen (`GemAssetDetailsService`, `GemChartService`,
  `GemTransactionDetailsService`, `GemWalletHomeService`) — is built in the `ViewModelFactory.xxxScene(...)` that builds
  its view model, from the owners the factory already holds. It is never a field of
  `AppResolver.Services` and never an `@Entry`: that constructs it on every launch of an app that
  may never open the screen, and hands views a composition detail.
- **Android** — provided in a Hilt module, injected. A Compose scene reads one instance from a `CompositionLocal` provided at `MainActivity` (`LocalChainService`, `LocalAssetConfigService`) only for the dependency-free config services; a screen's Core answers come from its view model's service, never from a `CompositionLocal` inside a feature composable (`LocalDeeplinkService` building the share link and `LocalAssetConfigService.acquireFlow` deciding the confirm button were reach-throughs and are gone). A non-`@Composable` helper takes an explicit parameter — a `CompositionLocal` cannot be read outside a composable. A screen service is a `@Provides` like any other — Hilt builds it when its view model first asks, so nothing is built at launch — and is never read from a `CompositionLocal`.
- **A value type or a namespace of statics** takes the service as a method parameter only when the
  answer genuinely requires that service's dependencies. A pure receiver-owned answer stays on
  the receiver according to § 6.

Dependency-free FFI transport adapters are the exception: `GemSimulationFormatter` and
`PriceAlertFormatter` may be constructed locally because they have no state to substitute. Do not
extend that exception to a service, store, client or a type whose behavior can cross on its honest
receiver.

Prefer the platform abstraction (`GemConfirmServiceProtocol` on iOS,
`GemConfirmServiceInterface` or a case on Android) wherever a test needs substitution. Mocking the
concrete UniFFI object is fragile: any unstubbed generated method can reach a native handle the
mock does not have.

## 9. Errors

Use the shared `GemServiceError` for ordinary API, store and service failures. Add a feature error
enum in `error.rs` only when the app needs structured domain data to render or branch without
parsing a message:

```rust
pub enum GemConfirmError {
    ScanMemoRequired { symbol: String },
    BalanceMissing { asset_id: AssetId },
    Sign { error: GemSignerError, msg: String },
}
```

When a lower-level error has a canonical feature-level mapping, implement `From` once in
`error.rs` and use `?`:

```rust
impl From<GemServiceError> for GemConfirmError {
    fn from(error: GemServiceError) -> Self {
        Self::Load { msg: error.to_string() }
    }
}
```

Use `map_err` only when the call site adds context or deliberately selects a non-default category,
such as `Record`, or when a named mapper preserves structured `Offline`/`Network` gateway cases.

The app **localizes Core's error directly** — it does not translate it into a parallel app-side enum first:

```swift
extension GemConfirmError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .ScanMemoRequired(symbol): Localized.Errors.ScanTransaction.memoRequired(symbol.boldMarkdown())
        ...
        }
    }
}
```

A duplicate taxonomy costs a mapping function, re-derives data Core already carries, and drifts. Classify Core's error where a screen needs to branch; do not re-wrap it.

## 10. Tests

| Layer | What it tests | Where |
|---|---|---|
| Core | the pure rule or intrinsic receiver behavior | owning module, usually `rules.rs`; beside the type for intrinsic behavior |
| iOS | the mapping Core → app types | feature tests, substituting an I/O screen service when needed |
| Android | the wiring — that the case passes Core's answer through | module unit tests, substituting the case or service interface |

Neither app tests a rule that lives in Core. If an app test would fail when a Core rule flips, the rule is in the wrong place or the test is asserting the mock.

A Core test double for a store or a port lives in the owning folder's `testkit.rs`
(`#[cfg(test)] pub(crate) mod testkit;`), named after the trait it implements —
`MemoryPreferencesStore`, `MemoryWalletStore`, `MemoryConnectionStore`, `TestWalletConnectSigner`
— so a service test composes the doubles of every folder it depends on instead of writing one
struct that implements six traits. Cross-cutting doubles (`TestAlienProvider`) live in
`gemstone/src/testkit.rs`. A double that exists to probe one behavior of one test (a store that
counts writes or delays a read) stays inline with that test.

**A test never hand-rolls a double a testkit already ships.** A `struct` in a test module that
implements `Client`, `Target` or a store trait is a stand-in nobody else uses: it drifts the moment
the real trait grows a method, and it asserts the stand-in rather than the path the app takes. Take
the double from the owning crate's `testkit` — enabled through that crate's `testkit` feature under
`[dev-dependencies]`, never copied — and drive the real request through it:

```rust
let client = AlgorandClient::new(MockClient::new().with_post_with_headers(|path, body, headers| {
    assert_eq!(path, "/v2/transactions");
    assert_eq!(body, [0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(headers.get(CONTENT_TYPE).map(String::as_str), Some(ContentType::ApplicationXBinary.as_str()));
    Ok(br#"{"txId":"TXID"}"#.to_vec())
}));
```

`MockClient` encodes the body exactly as `ReqwestClient` and `RpcClient` do, so a handler that
asserts bytes and headers is asserting the wire. References:
[`gem_client::testkit`](../core/crates/gem_client/src/testkit.rs) and `mock_jsonrpc_client` for HTTP,
[`gem_hypercore/src/testkit.rs`](../core/crates/gem_hypercore/src/testkit.rs) for a chain client,
[`primitives/src/testkit/asset_mock.rs`](../core/crates/primitives/src/testkit/asset_mock.rs) and
[`storage/src/testkit/scan_address_mock.rs`](../core/crates/storage/src/testkit/scan_address_mock.rs)
for fixtures; call sites in
[`gem_algorand/src/rpc/client.rs`](../core/crates/gem_algorand/src/rpc/client.rs) and
[`gem_stellar/src/rpc/client.rs`](../core/crates/gem_stellar/src/rpc/client.rs).

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

- **Never mock a dependency-free constructible service** (`GemChainService`,
  `GemAssetConfigService`, …). Construct the real one. An app test may substitute an I/O screen
  service to test mapping or state; the returned Core answer is then a stated premise, not a rule
  assertion.
- **Never fabricate I/O to reach a rule.** An offline provider, in-memory stores and empty rows stood up so a test can touch rules that use none of them is always the wrong answer. Pass the answer in from the caller, or mock the service and state the premise.

A mock's defaults should be the *usual* case. A mock that fails by default becomes a trap the moment another method starts depending on it.

## 11. Landing a change

1. Implement in Core with the rule test.
2. If a UniFFI signature, TypeShare model, `remote_types.yml` entry or mobile integration boundary
   changed, run `just generate` from the repo root. Internal Core changes that preserve those
   contracts do not require regeneration. Never generate against half-edited Core.
3. Wire both platforms.
4. **Delete the old path it replaces:** free function or wrapper, app call sites, obsolete mocks,
   duplicate tests and unused imports. A migration that leaves both paths has not migrated anything.
5. Build both apps when their generated interface or integration changed, then verify the affected
   suites (see `SERVICES.md` § Verification).
6. Search for the old symbol and review the diff for unused public API, redundant conversions and
   stale generated files.
7. If the change completes an item tracked in `SERVICES.md`, remove that item with the change.
   Commit and publish only when authorized under [Task Workflow](../skills/task-workflow.md).

### When the platforms disagree

Check the documented contract, callers, and tests: a difference may be intentional, or a test may pin a bug. Explain the evidence for the shared behavior before changing it. Ask only when the product decision remains unresolved; a passing test alone does not choose the contract.

## 12. A client's requests are one enum; the client only sends

**REST clients own a request target; JSON-RPC clients own a request enum.** Keep paths and request-specific metadata in the target. Credentials, transport, envelope handling, and pagination belong in the client. The deliberate exceptions are listed below.

The enum is a § 1 rule for a request: inputs in, wire format out, no transport, no secret, no
clock. The two references are
[`TronGridTarget`](../core/crates/gem_tron/src/rpc/trongrid/target.rs) for REST (`FooTarget` in
`rpc/target.rs`) and [`SolanaRpc`](../core/crates/gem_solana/src/jsonrpc.rs) for JSON-RPC
(`FooRpc` in `jsonrpc.rs`, method constants in `method.rs`). For a direct GET/POST without shared client work, use [`AptosClient`](../core/crates/gem_aptos/src/rpc/client.rs); do not add a forwarding `send` helper.

Inspect the [target](../core/crates/gem_tron/src/rpc/trongrid/target.rs) and [client](../core/crates/gem_tron/src/rpc/trongrid/client.rs) together; keep examples linked to their implementation instead of maintaining a second code sample here.

Variant fields are named: `GetAccount(String)` does not say what the string is. No path or query string is a `const`; `path()` builds every one. The target implements
`gem_client::Target` (`path()`, `headers()` when a request carries one, and `content_type()` when a
body is not JSON); the client owns the
transport, the credentials, the clock, the signature, the envelope and the pagination loop. A
method is `self.client.get(target).await` or `self.client.post(target, &body).await`. For `gem_client::Target`, do not add a body enum or a second dispatch over GET/POST variants to unify these calls; the device client's `gem_jsonrpc::Target` has a separate contract below. A private helper exists only for shared work: credentials
(`TronGridClient::send`), a 404 that is a value (`StellarClient::get_or_not_found`), an envelope
(TON). A GET-only host needs only `path()`; `GemDeviceApiTarget` is the full shape with
`method()`, `body()` and a signed header.

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

**Tests.** A client test over `MockClient` or `mock_jsonrpc_client` asserts behaviour the wire
shape does not show: an envelope's failure branch, the paths a pagination loop produced, the
body and content type of a broadcast, a merged credential header. Do not test `path()` by
copying its implementation into the expectation. A wire-contract regression test uses an independently specified request and exercises the real client through its mock transport.

**Deliberate exceptions.** Three things stay on raw `reqwest` because they are not REST clients:
the off-chain NFT metadata fetch (an arbitrary HTTPS URL read under a byte cap with redirects
off), the image downloader (binary bodies), and the egress node health probe (only the status
matters). The OKX client sends the string it signed rather than a target, and the alien reqwest
provider is the transport itself.

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

- **Human input** (text a person typed into a field): parse locale-aware. iOS `ValueFormatter.inputNumber(from:decimals:)`; Android `String.parseInputNumber()`. These interpret the device's grouping/decimal separators.
- **Machine strings** (QR/payment-link amounts, API/exchange payloads, `Double.description`, anything the app did not get from a keyboard): parse locale-independently. iOS `BigNumberFormatter.standard.number(from:decimals:)` (fixed `en_US`). A machine string always uses `.` as the decimal point, so feeding it to the human-input parser makes a dot-grouping locale read it 1000x too high.
- **A value the app itself produced** (e.g. a `Decimal` from fiat/crypto conversion): use iOS `ValueFormatter.displayedNumber(from:decimals:)` so the app never re-parses its own formatted output.
- **Amounts Core formats for notifications** (`number_formatter::ValueFormatter`, `ValueStyle::Auto`, used by the daemon pusher, the staking rewards notifier and in-app notifications) keep two decimals above one and four significant digits below it, dust included: a push title has no room for `0.000040036032429186 ETH` (issue #1155), so it reads `0.00004003 ETH`. The app list formatters keep full precision for dust on purpose; that is a screen with room, not a title.

### iOS localization compiles to String Catalogs, accessors are generated in Core

The localization generator writes one `.xcstrings` String Catalog per table (app, InfoPlist, widget) instead of per-language `.lproj/*.strings` files, and generates the typed `Localized.swift`/`WidgetLocalized.swift` accessors itself. SwiftGen is not used. Two constraints drove this:

- Xcode 26 can generate catalog symbols natively, but they are internal to the owning target; `Localization` is a shared SPM package consumed across the app, so the public accessor surface must be generated by our tooling.
- The historical dotted iOS keys (`common.cancel`) cannot be derived from the underscore Fluent IDs (`common_maximum_value` vs `common.maximum_value`); the committed `Localizable.xcstrings` is the source of that mapping, which the generator reads back on every run. Do not delete the catalogs and regenerate from scratch — the dotted key mapping (and with it the `Localized.*` API) would be lost.
