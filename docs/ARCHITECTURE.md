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

A Core test double for a store or a port lives in the owning folder's `testkit.rs`
(`#[cfg(test)] pub(crate) mod testkit;`), named after the trait it implements —
`MemoryPreferencesStore`, `MemoryWalletStore`, `MemoryConnectionStore`, `TestWalletConnectSigner`
— so a service test composes the doubles of every folder it depends on instead of writing one
struct that implements six traits. Cross-cutting doubles (`TestAlienProvider`) live in
`gemstone/src/testkit.rs`. A double that exists to probe one behavior of one test (a store that
counts writes or delays a read) stays inline with that test.

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
entry and the `fee()` accessor that handed it out.

**Split by responsibility to break a composition cycle; never reach through it.** `GemNodeService`
supplies node URLs to the provider the gateway is built on, so it can never hold the gateway.
Putting `check_node` on it was tried and does not compose. The networks screen's
`GemChainSettingsService` sits above the gateway and composes it with `GemNodeService` and
`GemExplorerService`, so status, validation, the node list and the explorer choice come from one
per-screen service; `GemNodeService` keeps the list. Both apps' hand-written node wrappers went with
the change. The same service now finishes the two screens: `check_node` takes the raw input, applies
the one URL rule (`https`, dotted host, bare host gets the scheme) and returns `GemNodeCheck` with
the sync flag the provider reports and a classified `Latency`, failing with a typed
`GemAddNodeError`; `node_status` returns `GemNodeStatusState` — `Error` for a failed call *or* a
zero block, so neither app keeps a "block is zero" rule — and `Latency::from_milliseconds` owns the
fast/normal/slow thresholds both apps had copied. iOS's `URLDecoder`/`URLTextValidator`, Android's
`NodeUrlParser` and the hand-written `NodeStatus` models on both went with it.

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

**The confirm screen's destination row is the same rule.** iOS hid it for `.account`, `.swap`,
`.perpetual` and generic-sign and titled it by input type; Android hid it for approvals, earn,
freeze/unfreeze and multi-validator rewards, looked the validator back up from Room, and put
the dApp name in the row for every generic request. `GemTransferData::destination()` returns
`GemConfirmDestination { Recipient | Contract | Validator | Resource | Provider }` or nothing,
built from the stake type and earn type Core already carries (so Android's `GetStakeValidator`
lookup and its recipient-name gap are gone); both apps map a variant to a title and render.
Android keeps its dApp-name row beside it, as iOS keeps `.app`. The tests that pinned the old
tables on each app — `ConfirmRecipientViewModelTests` with its silently-passing `guard case`,
`ConfirmDestinationTableTest` — became one Core test plus a mapping test on iOS.

**Which header a transaction gets is a rule too.** iOS kept two tables in
`TransactionHeaderTypeBuilder` (by transaction type, by input type) and Android two more (the
details `amount` getter, the confirm screen's `when`), disagreeing on approvals (asset image vs
amount) and contract calls (amount vs symbol). `GemTransactionHeaderKind { Amount { shows_fiat }
| Swap | Nft | Symbol | AssetImage }` comes from `GemTransactionDetailsService::header_kind` for
a stored transaction (falling back to an amount when swap or NFT metadata is missing) and
`GemTransactionInputType::header_kind` for a confirm input; the apps only build the header the
kind names. The Android details amount also takes its sign from Core's row value now, as iOS
always did.

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
which the shared input model takes directly on both apps.

**Post-import setup belongs to the wallet-change reaction, not the import call.** Android's
`PhraseAddressImportWalletService` ran `setupWallet` and a device sync inline after every import
while `AppViewModel` already runs `setupWallet` whenever the session's wallet changes — every
import set the wallet up twice. Both apps now do what iOS did: `GemWalletService.importWallet`
then `setCurrentWalletId`, and the root's wallet-change handler does the rest. The Android
"importing" indicator (`SyncWalletImport`) stays; it is a platform progress port, not setup.

**One error type per flow, carrying what the screen renders.** The confirm screen failed with
three Core error types — `GemConfirmError`, the preload's `GemTransferAmountError` (asset ids
only) and `GemSignerError` — and each app kept a parallel enum to merge them (`ConfirmError` with
fourteen cases on Android, `ConfirmTransferError.amount/.scan` on iOS) plus a mapper and its
tests. `GemConfirmError` now carries the amount failures itself (`InsufficientBalance`,
`InsufficientNetworkFee`, `MinimumAccountBalanceTooLow`, each with the `Asset` the screen names
and the required/available values), so a preload amount failure is the same type as a scan or
broadcast failure and the apps only map variants to strings and info sheets. Android's
`ConfirmState` holds the `Throwable`; iOS's `ConfirmTransferError` keeps only the
platform-classified `chain` and `other` cases beside `confirm`.

**A screen's action list is a Core answer.** The stake screen's manage rows — stake (and whether
it is enabled or first needs a frozen balance), freeze, unfreeze, claim rewards — were derived on
both apps from five chain flags and two balance checks, with the view-only gate on one app only.
`GemStakeService::stake_actions(wallet_type, chain, has_validators, frozen, rewards)` returns
`[GemStakeActionItem { action, is_enabled, requires_frozen_balance }]` and
`can_claim_all_rewards(chain, delegations_with_rewards)` the claim-all decision, so both apps
render the list and neither reads a stake flag; the chain-flag extensions that existed only to
feed those rules went with it.

**The app forwards the SDK event; Core decides the reply.** The WalletConnect request path
was the same eight-step dance on both apps — dedupe, find the session, check the origin, handle,
respond, reject on error, notify the user, log — with different message-id formats and one app
notifying on a malicious origin and the other not. `GemWalletConnectService::process_request`
takes the raw event and returns the reply and what (if anything) to tell the user, so each app is
"build the request from the SDK event, send the response, show the failure". The pieces the apps
used to compose (`handle_request`, the connection lookup) stopped being exported.

**A launch-time entry is not the place for a screen's dependency.** `GemRewardsService`,
`GemReceiveService`, `GemNodeStatusService`, `GemPerpetualDetailsService`, `GemPriceAlertService`,
`GemServiceStatus`, `GemAppUpdateService`, `GemNotificationService`, `GemNftService`,
`GemStakeService` and `GemChainService` were `@Entry` values on iOS only so a navigation view
could assemble a view model. Every scene is now built in `ViewModelFactory.xxxScene(...)`; the
environment (and `AppResolver.Services`) keeps only what the app needs at launch, and a scene
that needs the current wallet gets it from the factory (`inAppNotificationsScene()`,
`selectAssetScene(selectType:)`), not from a session read in the view. The second pass moved
`assetsService`, `nftService` and `recentAssetsService` off the environment into
`NavigationPresenter`, which is the object that actually acts on them (transaction header
actions, swap completion, recording a recent when an asset action opens), and deleted a
`deviceService` entry that was threaded into a navigation view and never read. Eight entries
remain: navigation state, the presenter, the handler, the factory, preferences, the connector,
its presenter and connection status.

**Two service calls with a condition between them are one Core call.** The notifications screen
on both apps ran `sync`, then read the store for an unread row, then called `markRead` — the same
three steps with the same `if`. That `if` is a rule, so `GemNotificationService::open(wallet_id)`
owns it and the store port answers `has_unread_notifications`; `sync` and `mark_read` stopped
being exported. When a view model sequences service calls, the sequence is the thing to move.

**A secure port beside the service is the service missing a method.** `WalletDetailViewModel`
held the iOS `Keystore` next to `GemWalletService` only to export the secret phrase or private
key, while `GemWalletService` already holds the keystore and the password port. `export_secret`
returns `GemWalletSecret { Words | PrivateKey }` by wallet type, so the view model holds one
service and the type-to-secret rule lives in `rules::secret_export` with a flip test. Android
keeps its prompt-driven export until S3 decides where authentication is enforced. The same
shape held the WalletConnect message screens: iOS `SignMessageSceneViewModel` built a
`MessageSigner` and signed through the `Keystore`, Android through a `GemSignMessageOperator`
reading the same unauthenticated password store the Core port wraps. `GemSignMessageService`
now holds the keystore and the password port and `sign(wallet_id, message)` is the one path on
both apps; the operator and the view-model signer are gone.

**A nullable wallet at a call site is a session read that belongs in Core.** Android's
`updateRecent` began with `session.value?.wallet ?: return@launch` and iOS threaded
`wallet.id.id` into every recent-activity call, because the service took the wallet id as an
argument. Core already owns the current wallet in `GemWalletSessionService`, so
`GemRecentActivityService` reads it there and fails with `NotFound` when there is none — no
caller carries a wallet, and none returns silently. When an app reaches for the session to feed
a Core call, hand the session to the service instead. The same move removed the
`Option<Wallet>` on `GemPerpetualService::sync_enablement`, and with it the iOS mock's copy of
the connect rule: the lifecycle tests now set `connects` as a premise and check that the observer
reacts, which is the only thing the app decides.

**Two entry points that record the same thing are one method and a rule.** `add_recent_asset`
took a type the app had chosen and `add_recent_search` derived it from the asset; the perpetual
screen picked `Perpetual` by hand. `GemAssetAction::recent_activity_type(asset)` answers all of
them — `Open` is `Perpetual` for a perpetual asset and `Search` otherwise — behind one
`add_recent(action, asset)`. Android's select and search actions now carry the `Asset` the row
already had instead of an id the view model looked back up, and iOS's `SelectAssetFlow` lost the
`selectionEffect` enum that re-encoded which flows record.

**An export nobody on either app calls is a Core-internal method.** A sweep of every
`#[uniffi::export]` method against both apps' non-generated sources found forty with no caller:
raw API pass-throughs (`GemAssetsService::{get_asset, get_assets, search, search_assets}`),
whole services only ever passed as dependencies (`GemPriceService`), pieces of a flow Core
already composes (`GemConfirmService::{preload, simulation, fee_assets}`,
`GemTransactionInputType::{fee_asset, asset_ids, recent_activity}`) and preferences only Core
reads. Each moved to a plain `impl`. The iOS `GemConfirmTransferServiceMock` had been the only
thing keeping the confirm pieces exported — it recomposed Core's flow from them — so it now
answers from premises and takes the concrete confirm mock. Re-run the sweep after removing a
caller: `rg -l '#\[uniffi::export\]'`, list the `pub fn`s, camel-case each, search both apps.
The second pass found twenty-three more outside `services/` — the gateway, swapper, keystore,
simulation and signer methods Core's own services call — and four that nothing called at all
(`GemMnemonic::is_valid_word`, two `GemSwapper` wrappers, `GemAutocloseEstimator::price_change_percent`)
plus a `GemJobConfiguration` record the tracker had stopped using; the former moved to plain
`impl`s, the latter were deleted.

**A debug screen is a screen.** iOS's `DeveloperViewModel` held five Core services; Android's
`DevelopViewModel` three cases and a `PlatformStore`. `GemDeveloperService` composes the device
platform, the preferences, the transaction-state store and the perpetual service, so each app
holds it alone (iOS keeps the plain stores it wipes); `reset_transactions_timestamp` and
`delete_preferences` stopped being exported from `GemWalletPreferencesService`.

**A transfer the confirm screen receives is built by Core, not assembled at the call site.**
Both apps built the stake confirm transfer themselves — Android's `GemTransferData.stake` with
a `StakeType.validatorId` switch, iOS's inline `TransferData(...)` in three view models — and
each decided on its own which stake types keep the max flag and which name the validator.
`stake_transfer_data(asset, stake_type, value, use_max_amount)` on `GemAmountService` and
`GemStakeService` and `GemAmountService::earn_transfer_data` answer that once; the earn builder
also finds the account on the session wallet and asks the gateway itself, so the apps stopped
calling `get_earn_data`. When a screen packs a `GemTransferData` by hand, the recipient and the
flags it picks are a rule Core should own.

**A rule about a value takes the value, not the fields the app tore off it.** The delegation
screen asked Core four questions with the delegation dismantled at the call site —
`delegation_actions(wallet_type, chain, provider, state)`, `can_claim_delegation_rewards(…,
state, rewards)`, `shows_rewards(state, rewards)`, `shows_completion_date(state)` — and Android
filled `provider` with a constant `Stake` that would have been wrong for an earn position. Each
now takes the `Delegation` (or its `DelegationBase`) and reads the chain, provider, state and
rewards itself; the rewards compare as the `BigUint` they already are instead of re-parsing a
string. When a Core method's parameters are three fields of one typeshare value the app
already holds, pass the value.

**One balance record crosses the FFI.** Core carried three — `GemAssetBalance` for the balance
store, `GemTransferBalance` (available, frozen, locked, withdrawable, votes) for the amount rules
and `GemStakeBalance` (frozen, locked, staked, pending, rewards) for the staked-value rule — and
each app wrote a field-copy bridge per record from its own balance model, so a new Core rule that
needed the balance meant a fourth copy (`GemTransferBalance+GemstonePrimitives.swift` lasted one
commit). Core even converted `GemAssetBalance` into `GemTransferBalance` internally on the confirm
path. `GemAssetBalance` is the one balance input now: the amount rules, `available_value`,
`stake_actions` and `staked_value`/`shows_stake_balance` take it, and each app keeps a single
bridge — iOS `GemAssetBalance(balance, assetId:)`, Android `AssetBalance.toGem()`. A Core record
that is a subset of another Core record is a second bridge on both apps; take the whole value.

**A screen's decision that ends in a transfer is a Core answer with the transfer inside.** The
stake screen decided on both apps whether claiming rewards goes straight to confirm or through
the amount screen — filter the delegations with rewards, count them, ask Core `can_claim_all_rewards`,
sum the rewards, build the transfer — and computed the frozen balance and the reward total for
`stake_actions` itself. `GemStakeService::claim_rewards(chain, delegations)` returns the total and
`GemClaimRewardsDestination::{Transfer, Amount}`, and `stake_actions` takes the balance and the
delegations; iOS's `frozenResources`/`rewardsValue`/`delegationsWithRewards` and Android's
`getFrozenResourceAmount`/`sumRewardsBalance` are gone, and `can_claim_all_rewards` is no longer
exported.

**The transaction detail screen's rows are one Core answer.** Both apps decided on their own
whether a swap shows a two-step progress (cross-chain provider, not yet confirmed), what each
step's status is per transaction state, when "swap again" appears, when the confirmation ETA
row shows (pending, positive, and not already inside the progress), and whether a perpetual's
pnl and price rows show — and Android's Compose item mapped the state to step statuses a
third time. `GemTransactionDetailsService::details(TransactionExtended)` returns
`GemTransactionDetails { swap_progress, swap_again, provider_name, estimated_confirmation_seconds,
pnl, price }`; the view models only format what is there. `TransactionExtended` crosses as JSON
like `Transaction` does (the primitives struct existed for typeshare but was never declared as a
module, and Android carried a stale hand-copied version without `prices` — both fixed). When
several rows of one screen each re-derive a fact from the same record, ask Core for the screen's
facts in one call.

**Which balance rows an asset shows is a rule on the balance.** iOS's asset scene decided the
breakdown section from a `[BalanceType: BigInt]` map ("any bucket beyond available"), then
each row from its own predicate, and fell back to a lone APR row; Android's factory decided it
from `isStaked(chain) || reserved != 0`, hid the available row when it equalled the total, and
rendered three of the five rows. `GemAssetBalance::detail_rows(chain, is_stake_enabled)` returns
the ordered rows (`Available`, `Staked`, `Earn`, `PendingUnconfirmed`, `Reserved`) with their
values — the available row only when something is held beyond it, the staked row whenever the
chain stakes (zero means "offer staking", which both apps render as the APR) — and each app only
formats them. iOS's `AssetData.balances` map and the `has*Balance` predicates, and Android's
`hasBalanceDetails`/`formatAvailable`/`formatStake`/`formatReserved` and `isStaked`, are gone.

**A direction the signer interprets is a Core convention, not a screen's choice.** The HyperCore
signer treats `PerpetualConfirmData.direction` as the *position* direction for a close and a
reduce (`is_buy = direction == Short`, reduce-only), and the slippage price follows the same
reading. Android built a reduce order with the position direction; iOS flipped it, so a reduce
on a long became a reduce-only buy. The screens now both pass the position direction and the
signer test pins the convention. When two apps disagree on an input Core signs, the signer's
reading is the rule — write it down there, then fix the app.

**A position action is a Core value the apps carry, not a mirror they each build.** Both apps kept
their own `PerpetualTransferData` / `PerpetualPositionAction` (provider, direction, asset index,
price, leverage, margin type; open / increase / reduce with the reducible margin), built them
from the market and the position in the position screen, and turned them into an order and a
transfer in the amount screen through per-app factories — which is where the reduce-direction
bug lived. `GemPerpetualDetailsService::position_action(perpetual, asset, position, kind)` and
`close_transfer` build the action and the close on the position screen;
`GemAmountService::perpetual_transfer_data(action, value, use_max_amount, leverage, take_profit,
stop_loss)` turns it into the transfer, formatting the trigger prices itself; the autoclose sheet
asks `GemAutocloseModify::transfer(provider, asset)` for the modify. iOS carries the record through
navigation as-is; Android carries it through the route via `GemTransferService::{encode,
decode}_position_action`, the same way it already carried the confirm input.
`GemPerpetual::{order, close_order, transfer_data}` stopped being exported. When an app-side type
exists only to be handed to Core later, make it a Core record and let Core build it.

**The confirm screen reads the current wallet from Core.** Every Android screen that could end
in a confirm built `GemConfirmInput` itself — seventeen `session.wallet.getAccount(chain) ?:
return` / `assetInfo.owner ?: return` lookups — and the confirm view model read the session again
for `load`'s wallet id and `execute`'s wallet; iOS threaded `wallet` through
`ConfirmTransferRequest` and looked the account up in Swift. `GemConfirmTransferService` now
takes the session: `confirm_input(transfer)` picks the signing account for the transfer's chain
(`AccountMissing` if the wallet has none), and `initial_state`, `load` and
`execute(confirm, value, network_fee, simulation)` read the wallet themselves, so `GemSendInput`
stopped crossing the FFI. Both apps hand the confirm screen a `GemTransferData`: iOS's request lost
its wallet (the view model keeps one only to navigate to buy / receive from the fee sheet), and
Android's `ConfirmTransactionAction`, routes and `PaymentDestination` carry the transfer through
`GemTransferService::{encode, decode}_transfer_data`. A screen that always acts on the current
wallet must not be told which wallet that is.

