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
    rules.rs    pure functions and receiver methods + their unit tests
    model.rs    feature records and enums; only FFI types derive UniFFI
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
pub(super) fn selectable_fee_assets(assets: Vec<Asset>, balances: Vec<GemAssetBalance>, prices: Vec<GemAssetPrice>) -> Vec<GemFeeAsset> {
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

Its test states the rule in one sentence and fails if the rule flips:

```rust
#[test]
fn test_a_fee_asset_with_no_available_balance_is_not_selectable() {
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
        let assets = self.assets.assets(fee_asset_ids.clone()).map_err(load_error)?;
        let balances = self.balance.balances(wallet_id, fee_asset_ids.clone()).map_err(load_error)?;
        let prices = self.price.prices(fee_asset_ids).map_err(load_error)?;
        Ok(rules::selectable_fee_assets(assets, balances, prices))
    }
}
```

The method is thin: gather inputs, call the rule, return. Product or domain-decision branching
belongs in `rules.rs`; I/O sequencing, error propagation and empty-work short circuits may remain
in the service.

**Point reads should be synchronous.** `GemWalletStore.get_wallet` is a sync trait method, so `GemWalletService` answers a session lookup without `await`. Do the same for any single-row read — an `async` point read pushes the caller back to the store, which is how the confirm screen ended up reading `AssetStore` directly for two years.

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

An adapter maps reads and writes and nothing more — **no rules, no entity mapping inline**.
Mapping goes in a mapper file beside the adapter (`StoreModels.kt`, `nft/NftModels.kt`).

**Stores only write rows whose values differ.** A blanket write churns observers and hides real changes.

## 5. The app maps; it does not decide

### iOS

There is no app-side service wrapping a Core service. The view model holds its screen's Core
service and calls it; each call is one line in, one mapping out.

```swift
func preload(request: ConfirmTransferRequest, selection: FeeSelection, feeAssetSelection: FeeAssetSelection) async throws -> ConfirmTransferPreload {
    let account = try request.wallet.account(for: request.data.chain)
    let preload = try await gemConfirmService.preload(
        walletId: request.wallet.id.id,
        input: request.data.confirmInput(from: account),
        options: GemConfirmLoadOptions(feeSelection: selection.map(), feeAssetId: feeAssetSelection.selectedAssetId?.identifier),
    )
    let feeAsset = try Asset(preload.feeAsset)
    return try ConfirmTransferPreload(
        metadata: preload.metadata,
        input: ConfirmTransferInput(
            confirmData: preload.confirmData,
            fee: preload.confirmData.fee.map(),
            transferAmount: preload.amount.map(asset: request.data.type.asset, feeAsset: feeAsset),
            feeAsset: feeAsset,
        ),
        feeRates: preload.confirmData.feeRates.map { try $0.map() },
        simulation: preload.confirmData.simulation.map { try Primitives.SimulationResult($0) },
    )
}
```

Core → app mappings live in `GemstonePrimitives` as extensions. A mapping onto a *feature-internal* type stays in the feature — `GemstonePrimitives` cannot import a feature module, and reaching for one is the signal that the mapping belongs in the feature.

### Android

A case in `gemcore` `application/<area>/cases/`, implemented in `data/coordinators/<area>/`, injected by Hilt. An observed read returns a `Flow`; the case still asks Core for the decision on each emission:

```kotlin
override fun getFeeAssets(): Flow<List<AssetInfo>> = getCurrentWalletId().flatMapLatest { walletId ->
    combine(
        assetStore.observeAssetsInfoByChain(walletId.id, chain),
        assetStore.observeHiddenAssetsInfoByChain(walletId.id, chain),
    ) { visible, hidden -> visible + hidden }
        .map { assets ->
            val selected = confirmService.feeAssets(walletId.id, chain.string).map { it.asset.decodeJson<Asset>().id.toIdentifier() }.toSet()
            assets.filter { it.asset.id.toIdentifier() in selected }
        }
}
```

The store is the change trigger. Core is the decider. Core has no observation primitive, and that is the only reason the app watches its own tables.

## 6. Where a transaction input's answers come from

The app uses Core's types. It does not declare a parallel enum of the same shape — that is
two definitions of one thing, and every crossing pays for a two-way mapper. `TransferDataType`
was that copy: eleven restated cases and 138 lines of mapping. It is deleted.

Four kinds of member, four homes:

| | Home | Example |
|---|---|---|
| A field the case already carries | extension on the Core type | `inputType.asset` |
| An answer derived only from one local Core record or enum | exported method on that type | `inputType.transactionAsset()` |
| A rule requiring I/O, stored dependencies, or independently sourced inputs | method on the service that owns those dependencies | `confirmService.metadata(...)` |
| Encoding an app value into a case | extension on the Core type | `.stake(asset, stakeType)` |

**Never add a free exported function or a service wrapper — stateless or not — for an answer
already owned by one local Core type.** `transaction_input_asset(input_type)` and
`transferService.asset(inputType:)` both hide the natural receiver; use
`inputType.transactionAsset()`. A method that ignores `self` is the same mistake even on a
service with real dependencies: `confirmTransferService.simulationAssetIds(simulation:)` is a
property of `SimulationResult`, not of an eight-dependency orchestrator. Keep the owning service
when the rule performs I/O, holds real dependencies, or combines inputs without a single honest
receiver. Do not invent an artificial receiver for external or remote types Core cannot extend.

`#[uniffi::export]` on an `impl` exports its `pub` members and nothing else, and both directions
fail silently. Narrowing one member to `pub(crate)` drops it from both apps' bindings with no
Rust error — the break surfaces as `cannot find 'output' in scope` in whichever app still used
it, so a platform that never called it stays green. A private helper written inside an exported
`impl` becomes public API instead, and shows up as a protocol method every mock must implement.
Keep helpers in a separate un-exported `impl` block, and after changing a member's visibility
build both apps, not the one you were working in.

The encoding members are scaffolding, not a pattern to copy. `.stake(asset.map(), stakeType.json())`
carries a `.json()` only because `StakeType` is not in `EXPOSED_TYPES`
(`core/bin/generate/src/remote_mappers.rs`). Add a type there and its declarations and both
platforms' mappers are generated from the Rust definition; the `.json()` becomes a generated
`.map()`, and where nothing persists the app's twin, the member disappears. Do not add a new
encoding member without adding its type to `EXPOSED_TYPES`.

## 7. One service per screen

A view model holds **exactly one** service, named `service`, and it is **`private`**. A non-private service means something outside the model — usually the view — is reaching through it for a dependency.

The service is named for the screen it backs, not for the feature and not for the layer: `GemContactsService` backs the contacts list, `GemManageContactService` backs the add-and-edit screen. No `Scene` or `Facade` in the name. When a screen needs more than one thing from Core, Core composes them:

```rust
/// Backs the add and edit contact screen.
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
    pub fn add_address(&self, addresses: Vec<ContactAddress>, input: GemContactAddressInput) -> Vec<ContactAddress> { ... }
    pub fn format_address(&self, address: String, chain: Chain, style: GemAddressFormatStyle) -> String { ... }
}
```

A sheet belongs to the screen that presents it and shares that screen's service. A screen you navigate *to* is a different screen with its own.

### A service never hands out another service

`service.manageContact()` is the same reach-through as `model.nameService`, one level down: the caller now depends on something it was not given. Every service is constructed in the composition root and injected.

The exception is a **shared component** — `AddressInputViewModel`, `NetworkSelectorViewModel` — which is reusable across screens and takes a narrow Core protocol. A screen's service may vend one of those, because the component's dependency is genuinely narrower than the screen's.

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

### Use the generated protocol, not the concrete object

UniFFI generates a protocol for every exported object. `GemAddressServiceProtocol` exists; importing `class Gemstone.GemAddressService` at a consumer means that consumer cannot be substituted in a test.

- **Consumers** (view models, components, validators) take `any GemFooServiceProtocol`.
- **The composition root** (`ServicesFactory`, `ViewModelFactory`) holds the concrete type — a UniFFI constructor needs it, and the root is the one place allowed to construct. Its fields are grouped by what they are: Core services, platform services, stores.

## 8. Services are injected, never constructed at a call site

A `GemFooService()` in a field initialiser or at file scope is a second instance the graph does not know about, and it is where an app-side variant creeps back in.

- **iOS** — registered in `ServicesFactory`, exposed through an `@Entry` in `ios/Gem/Types/Environment.swift`, passed into the view model.
- **Android** — provided in a Hilt module, injected. A Compose scene reads one instance from a `CompositionLocal` provided at `MainActivity` (`LocalChainService`, `LocalAssetConfigService`). A non-`@Composable` helper takes an explicit parameter — a `CompositionLocal` cannot be read outside a composable.
- **A value type or a namespace of statics** takes the service as a method parameter; the caller that can hold one passes it down.

Prefer an interface (`GemConfirmServiceInterface`) over the concrete UniFFI object wherever a test needs to substitute it — mocking the concrete object dereferences native handles a mock does not have and takes the JVM down.

## 9. Errors

One error enum per feature, in `error.rs`, carrying what the app needs to render:

```rust
pub enum GemConfirmError {
    ScanMemoRequired { symbol: String },
    BalanceMissing { asset_id: AssetId },
    Sign { error: GemSignerError, msg: String },
}
```

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
| Core | the rule | `rules.rs`, with a mutation check |
| iOS | the mapping Core → app types | feature tests, driving a `Gem*ServiceMock` |
| Android | the wiring — that the case passes Core's answer through | module unit tests, mocking `Gem*ServiceInterface` |

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

- **Never mock a constructible service** (`GemChainService`, `GemAssetConfigService`, …). Construct the real one, or the test asserts the mock.
- **Never fabricate I/O to reach a rule.** An offline provider, in-memory stores and empty rows stood up so a test can touch rules that use none of them is always the wrong answer. Pass the answer in from the caller, or mock the service and state the premise.

A mock's defaults should be the *usual* case. A mock that fails by default becomes a trap the moment another method starts depending on it.

## 11. Landing a change

1. Implement in Core with the rule test.
2. Regenerate bindings **only if an exported signature changed** — `just generate-stone` and `just generate-android-stone` from the repo root. Never against a half-edited Core: a generate run mid-edit yields bindings that fail somewhere unrelated.
3. Wire both platforms.
4. **Delete the app code it replaces.** A migration that leaves the old path in place has not migrated anything.
5. Verify (see `SERVICES.md` § Verification).
6. Commit and push, removing the item's line from `SERVICES.md` in the same commit.

### When the platforms disagree

Check for a test pinning the difference before picking a side — a divergence is sometimes a deliberate decision no one wrote down. If a test pins it, adopt that reading into Core; if nothing does, take the better one and say which in the commit message.
