# Feature architecture

The reference for how a feature is built across Core, iOS and Android. Every new feature follows this; every existing one converges on it. When code and this file disagree, the file is the target.

The shape came out of the confirm migration — the largest thing moved into `gemstone` — so the examples below are real code from `services/confirm`, trimmed.

## The one rule

**Core decides. The apps ask, render, and store.**

A rule is anything that could produce a different answer on one platform than the other: a filter, a threshold, a gate, an ordering, a mapping from state to what the user sees. If iOS and Android could ever disagree about it, it belongs in Core.

Everything else is platform work: rendering, navigation, observation, secure storage, keychain and biometrics, and the SQL that stores rows.

## Layout

```
core/gemstone/src/services/<feature>/
    mod.rs      the service: holds dependencies, orchestrates, exported over UniFFI
    rules.rs    pure functions + their unit tests
    model.rs    the records and enums that cross the FFI boundary
    store.rs    the trait each app implements over its own database
    error.rs    the feature's error enum
```

Only the files the feature needs. A feature with no persistence has no `store.rs`.

## 1. Rules are pure functions with a test that flips

A rule takes data and returns an answer. No I/O, no services, no clock.

```rust
// services/confirm/rules.rs
pub fn selectable_fee_assets(assets: Vec<Asset>, balances: Vec<GemAssetBalance>, prices: Vec<GemAssetPrice>) -> Vec<GemFeeAsset> {
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

## 2. The service orchestrates; it holds services, never stores

A service composes rules with I/O. Its dependencies are **other services**, not their stores — a store belongs to exactly one service, and reaching around that service is a second read path its owner cannot see.

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

The method is thin: gather inputs, call the rule, return. If a service method has branching logic in it, that logic is a rule that has not moved to `rules.rs` yet.

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
    Error { error: GemTransferAmountError },
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

## 4. The store trait is the app's only obligation

Core declares what it needs; each app implements it over its own database. Nothing else about the app's storage crosses the boundary.

```rust
// services/<feature>/store.rs
#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemPerpetualStore: Send + Sync {
    async fn save_perpetuals(&self, data: Vec<PerpetualData>) -> Result<(), GemServiceError>;
    async fn get_positions(&self, wallet_id: WalletId, provider: PerpetualProvider) -> Result<Vec<PerpetualPosition>, GemServiceError>;
}
```

An adapter writes rows and nothing more — **no rules, no entity mapping inline**. Mapping goes in a mapper file beside the adapter (`StoreModels.kt`, `nft/NftModels.kt`).

**Stores only write rows whose values differ.** A blanket write churns observers and hides real changes.

## 5. The app maps; it does not decide

### iOS

The app service forwards to Core and maps Core records into app types. It holds Core services and the app's own presentation collaborators — never a store Core owns.

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
        metadata: preload.metadata.map(assetId: request.data.type.asset.id, feeAssetId: feeAsset.id),
        input: ConfirmTransferInput(
            confirmData: preload.confirmData,
            fee: preload.confirmData.fee.map(),
            transferAmount: transferAmount(preload.amount, asset: request.data.type.asset, feeAsset: feeAsset),
            feeAsset: feeAsset,
        ),
        feeRates: preload.confirmData.feeRates.map { try $0.map() },
        simulation: preload.confirmData.simulation.map { try Primitives.SimulationResult($0) },
    )
}
```

Core → app mappings live in `GemstonePrimitives` as extensions (`GemConfirmMetadata.map(assetId:feeAssetId:)`). A mapping onto a *feature-internal* type stays in the feature.

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

## 6. Services are injected, never constructed at a call site

A `GemFooService()` in a field initialiser or at file scope is a second instance the graph does not know about, and it is where an app-side variant creeps back in.

- **iOS** — registered in `ServicesFactory`, exposed through an `@Entry` in `ios/Gem/Types/Environment.swift`, passed into the view model.
- **Android** — provided in a Hilt module, injected. A Compose scene reads one instance from a `CompositionLocal` provided at `MainActivity` (`LocalChainService`, `LocalAssetConfigService`). A non-`@Composable` helper takes an explicit parameter — a `CompositionLocal` cannot be read outside a composable.
- **A value type or a namespace of statics** takes the service as a method parameter; the caller that can hold one passes it down.

Prefer an interface (`GemConfirmServiceInterface`) over the concrete UniFFI object wherever a test needs to substitute it — mocking the concrete object dereferences native handles a mock does not have and takes the JVM down.

## 7. Errors

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

## 8. Tests

| Layer | What it tests | Where |
|---|---|---|
| Core | the rule | `rules.rs`, with a mutation check |
| iOS | the mapping Core → app types | feature tests, driving a `Gem*ServiceMock` |
| Android | the wiring — that the case passes Core's answer through | module unit tests, mocking `Gem*ServiceInterface` |

Neither app tests a rule that lives in Core. If an app test would fail when a Core rule flips, the rule is in the wrong place or the test is asserting the mock.

Two more:

- **Never mock a constructible service** (`GemChainService`, `GemAssetConfigService`, …). Construct the real one, or the test asserts the mock.
- **Never fabricate I/O to reach a rule.** An offline provider, in-memory stores and empty rows stood up so a test can touch rules that use none of them is always the wrong answer. Pass the answer in from the caller, or mock the service and state the premise.

A mock's defaults should be the *usual* case. A mock that fails by default becomes a trap the moment another method starts depending on it.

## 9. Landing a change

1. Implement in Core with the rule test.
2. Regenerate bindings **only if an exported signature changed** — `just generate-stone` and `just generate-android-stone` from the repo root. Never against a half-edited Core: a generate run mid-edit yields bindings that fail somewhere unrelated.
3. Wire both platforms.
4. **Delete the app code it replaces.** A migration that leaves the old path in place has not migrated anything.
5. Verify (see `SERVICES.md` § Verification).
6. Commit and push, removing the item's line from `SERVICES.md` in the same commit.

### When the platforms disagree

Check for a test pinning the difference before picking a side. Twice during this migration the "wrong" platform was right and the divergence was a deliberate decision — the Android stream reconnect cap, and rewards staying visible before wallets load. If a test pins it, adopt it into Core; if nothing does, take the better reading and say which in the commit message.

### Sizing a change

Grep counts on property names are useless. `.feeAsset` matched 381 lines and was twenty sites. Make the change, let the compiler count, and revert if it lands somewhere a dependency cannot go.
