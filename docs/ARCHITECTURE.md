# Feature architecture

The reference for how a feature is built across Core, iOS and Android. Every new feature follows this; every existing one converges on it. When code and this file disagree, the file is the target.

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

**Verify the test actually flips.** Invert the rule, run the test, confirm it fails, restore. A test that passes both ways is worse than no test — it certifies nothing while looking like coverage.

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

In this target shape, `GemConfirmError` implements `From<GemServiceError>` once in `error.rs`, so
the three reads use `?` without repeating the same `Load` conversion. The pending migration is
tracked in [`SERVICES.md`](SERVICES.md#4-core-surface).

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
    Error { error: GemTransferAmountError, asset: Asset },
}
```

**State that travels together is one type.** An approval is either an exact amount or unlimited — never a string plus a boolean the caller has to reassemble:

```rust
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemApprovalValue {
    Exact { value: GemBigUint },
    Unlimited,
}
```

### Field types

- Big-integer atomic quantities are `GemBigInt` / `GemBigUint`, never `String`. `String` moves the parse to every call site, and each one invents its own failure behaviour.
- `amount` is for `f64`. `value` is for big integers. Do not mix them.
- Full domain words: `transaction`, not `tx` — except where an external protocol field, database column or URL uses the short form verbatim.

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
override fun getTransactionDetails(id: TransactionId): Flow<TransactionDetailsAggregate?> =
    combine(getSession().filterNotNull(), getTransaction(id)) { session, data -> Pair(session, data) }
        .flatMapLatest { (session, data) ->
            val transaction = data?.transaction ?: return@flatMapLatest emptyFlow()
            val explorer = transactionDetailsService.transactionLink(...)
            getWalletAssets(transaction.getAssociatedAssetIds()).mapLatest { assets ->
                TransactionDetailsAggregateImpl(
                    data = data,
                    associatedAssets = assets,
                    explorer = explorer,
                    participant = transactionDetailsService.participant(transaction.toJson()),
                    ...
                )
            }
        }
        .flowOn(Dispatchers.IO)
```

The store is the change trigger. Core is the decider. Core has no observation primitive, and that is the only reason the app watches its own tables.

### Never call Core from the main thread

The `flowOn` above is not decoration. A synchronous Core call such as
`transactionDetailsService.participant` can read store callbacks that block on Room, and UniFFI
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
    .mapLatest { data -> transactionDetailsService.participant(data.transaction.toJson()) ... }
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
and `GemTransactionSummary::new` do the same for fees and rows. The apps and their tests construct
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
hand-written parsers the record mappers call by convention (`X(core:)` / `.identifier`,
`toX()` / `toIdentifier()`). Before adding a `remote` type, verify that the generator can represent
its full shape and inspect both generated mappers. It handles fieldless enums; `StakeType` has
data-carrying variants, so adding it mechanically would produce an incomplete mapping. Keep a single
JSON bridge at the boundary until the generator supports the type; never add a second app-side
model or copy policy to avoid that bridge.

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
    chains: Arc<GemChainService>,
}

#[uniffi::export]
impl GemManageContactService {
    #[uniffi::constructor]
    pub fn new(contacts: Arc<GemContactService>, addresses: Arc<GemAddressService>, names: Arc<GemNameService>, chains: Arc<GemChainService>) -> Self { ... }

    pub fn default_chain(&self) -> Chain { self.contacts.default_chain() }
    pub async fn save_contact(&self, input: GemContactInput) -> Result<Contact, GemServiceError> { ... }
    pub fn format_address(&self, address: String, chain: Chain, style: GemAddressFormatStyle) -> String { ... }
}
```

The pure address-list transformation stays on its request value:
`input.add_address(addresses)`. It does not belong on this service because it uses none of the
service's dependencies.

A sheet belongs to the screen that presents it and shares that screen's service. A screen you navigate *to* is a different screen with its own.

### A service never hands out another service

`service.manageContact()` is the same reach-through as `model.nameService`, one level down: the
caller now depends on something it was not given. Every service is constructed in the composition
root and injected. Returning `Arc<GemFooService>` from an exported service is migration debt, not
an exception to this rule.

The reusable exception is a **shared component** — `AddressInputViewModel`,
`NetworkSelectorViewModel` — which takes a narrow protocol. On iOS that protocol is
`AddressInputResolving`: four methods every screen service that owns a recipient input already
exports (`GemNameService`, `GemRecipientService`, `GemManageContactService` conform through an
empty extension), so the parent passes its own service and no Core service returns another
service. Android's `AddressInputModel` takes the same four-method `AddressInputResolving`; Kotlin
has no retroactive conformance, so `GemNameServiceInterface.addressInput()` and
`GemRecipientServiceInterface.addressInput()` wrap the service's own methods, and the parent passes
`service.addressInput()`. `NetworkSelectorViewModel` needs only the dependency-free
`GemChainService` and builds it itself.

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
- **Android** — provided in a Hilt module, injected. A Compose scene reads one instance from a `CompositionLocal` provided at `MainActivity` (`LocalChainService`, `LocalAssetConfigService`). A non-`@Composable` helper takes an explicit parameter — a `CompositionLocal` cannot be read outside a composable. A screen service is a `@Provides` like any other — Hilt builds it when its view model first asks, so nothing is built at launch — and is never read from a `CompositionLocal`.
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
7. Commit and push. If the change completes an item tracked in `SERVICES.md`, remove that item in
   the same commit.

### When the platforms disagree

Check for a test pinning the difference before picking a side — a divergence is sometimes a deliberate decision no one wrote down. If a test pins it, adopt that reading into Core; if nothing does, take the better one and say which in the commit message.

## 12. Shapes that were tried and reverted

Each of these looked right once and cost a revert. Read them before reaching for the same shape.

**A shared child renders a value; it does not hold a service.** `NetworkFeeSceneViewModel`,
`SwapDetailsViewModel` and `PriceImpactViewModel` are built by more than one screen, each with a
different single service. Two shapes were tried to let them share a dependency and both were
rejected: closures (`CustomFeeEstimating`, `mapQuote`) hide the dependency behind a type alias,
and a narrow Swift protocol cannot be satisfied by `any GemFooServiceProtocol`, because UniFFI
generates that protocol and Swift cannot make it inherit another. The shape that holds: the
parent computes the answer through its one service and passes the value — `swapPriceImpact`,
`minReceiveValue`, `etaMinutes`, `contractExplorerLink`. A child with no service is the goal,
not a compromise.

**A stateless object with a no-argument constructor is not a service.** `GemFeeService` was an
empty struct wrapping one pure function. The answer belongs on the value type as a constructor —
`GemCustomFee::estimate(...)` — and the service is deleted along with the field, the environment
entry and the `fee()` accessor that handed it out. `GemSwapQuoteService` is the remaining case;
its four methods are the same shape.

**Split by responsibility to break a composition cycle; never reach through it.** `GemNodeService`
supplies node URLs to the provider the gateway is built on, so it can never hold the gateway.
Putting `check_node` on it was tried and does not compose. The networks screen's
`GemChainSettingsService` sits above the gateway and composes it with `GemNodeService` and
`GemExplorerService`, so status, validation, the node list and the explorer choice come from one
per-screen service; `GemNodeService` keeps the list. Both apps' hand-written node wrappers went with
the change.

**Core returns the decision, not the ingredients for it.** When both apps would derive the same
thing from a returned list, return the derived thing: `GemChart { values, current }` instead of
each app comparing the stored price's timestamp against the last point (iOS used `>`, Android
`>=`); `GemConfirmSimulation` carries `primary_fields`, `secondary_fields` and
`has_critical_warning`; `GemAssetDetailsService::refresh` returns a failure per step instead of
swallowing them. If the app still filters, compares or checks after the call, the boundary is one
step too early.

**A value that crosses as its code needs no decode and no fallback.** `Chain`, `AssetId` and
`Currency` lower as their string code. `(try? Currency(json)) ?? .usd` invents a currency on a
path Core makes impossible; it was written three times before being removed. The only way that
parse fails is a generation mismatch between two artifacts of one Rust enum, so it is handled
once, by a named conversion with the same contract as UniFFI's own `try_lift` — `Currency(core:)`
on iOS, `toCurrency()` on Android, both generated from `remote_types.yml` — and never per call site.

**Construct a screen service where the screen is constructed.** `GemChartService` and
`GemTransactionDetailsService` first landed as `ServicesFactory` fields with an `@Entry` each,
and were moved into `ViewModelFactory.chartScene(...)` / `.transactionScene(...)` (§ 8): an
`@Entry` builds a screen service on every launch of an app that may never show that screen, and
puts a composition detail where views can see it.

**An extension that repeats conversions for a composed service is duplication, not convenience.**
`setAssetPinned(wallet:)` on `GemAssetDetailsServiceProtocol` copied the same `wallet.id.id` /
`assetId.identifier` mapping already written for `GemBalanceServiceProtocol`, and every composed
service would need another copy. Convert at the call site.

**A test that pins a rule moves with the rule.** When `minimumReceiveAtomic` moved to Core, the
Android factory test that asserted it was repointed through the real `GemSwapQuoteService`, so it
still guards parity end to end; it was not deleted and not left asserting a copy that no longer
exists. Mutation-check the moved rule with inputs that can tell the mutants apart — a
single-element list makes `any` and `all` indistinguishable.

**Stage explicit paths in a shared checkout.** A `git add -A` sweep committed another session's
half-applied view-model edit with no Core counterpart and broke `main` for everyone until the
Core half was written. Nothing in this repo is safe to stage blind.

**Never hand-edit a generated file.** `toPrimitivesOrNull()` was added by hand to the generated
`RemoteTypeMappers.kt`; the next `just generate-models` erased it and broke the confirm screen.
It was also a fallback Core makes impossible — an `Asset` Core returns always carries a valid
`AssetId` — so the honest fix was `toPrimitives()` at the call site, not a generator change.
When a generated shape is genuinely wrong, change `remote_types.yml` or the generator.

**A rule with no dependencies belongs to the owner that has them, not to an empty object.**
`GemRecipientService` was a `new()` with no fields whose two methods validated and built a
recipient from a chain, an input and a name record; `GemNameService` handed it out through
`recipients()`. Both rules are now methods on `GemNameService`, the owner of name records, and
the hand-out is gone.

**A screen service built per screen may keep that screen's state.** The confirm screen kept a
mutable `ConfirmTransferState` on iOS and three stitched flows on Android so a fee change would
re-run only the preload and keep the fee assets and simulation. `GemConfirmTransferService` is
constructed when the screen opens, so it holds the loaded scene itself: every call is `load`, and
the second one reuses what did not change. The apps replace their state wholesale.

**A "which address, which role" decision is a rule.** The transaction participant — sender or
recipient by direction, contract for approvals, validator for delegations, provider for earn,
recipient-or-contract from WalletConnect metadata — lived in a 35-line Swift `switch` and a
40-line Kotlin `when`, each also choosing the explorer link. `transaction_participant` in Core
returns the role and address, `GemTransactionDetailsService::participant` attaches the link, and
each app maps a role to its localized title.

**When both apps validate the same input differently, pick the exact one.** The fiat sell
check compared a fiat→crypto conversion through the session price on Android and the selected
quote's `crypto_amount` on iOS. `GemFiatQuoteService::amount_check` compares the quote's atomic
`value` (a field the apps never saw) to the available balance after the range check, so both
apps map one `GemFiatAmountCheck` to their error and neither converts anything. The random
amount, the defaults and the suggestions come from the same service, and `quote_url` enables
the asset it just sold a quote for.

**A timing constant the two apps disagree on is a Core method.** The swap quote debounce was
250 ms on iOS and 500 ms on Android; `GemSwapQuoteService::quote_debounce_milliseconds` (250,
matching the fiat quote debounce) sits beside `refresh_interval_milliseconds`, iOS passes it to
`debouncedTask(interval:)` and Android's `RequestSwapQuotes` takes it as a parameter, so neither
app keeps a number of its own. Android's stop-on-failed-quote break stays. The name-record
debounce (250/500 ms) went the same way: `name_record_debounce_milliseconds` on `GemNameService`,
forwarded by `GemRecipientService` and `GemManageContactService` so the `AddressInputResolving`
port carries it to the shared input model on both apps.

**A launch-time entry is not the place for a screen's dependency.** `GemRewardsService`,
`GemReceiveService`, `GemNodeStatusService` and `GemPerpetualDetailsService`'s inputs were
`@Entry` values on iOS only so a navigation view could assemble a view model. Every one is now
built in `ViewModelFactory.xxxScene(...)`; the environment keeps only what the app needs at
launch.
