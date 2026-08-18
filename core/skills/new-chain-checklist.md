# New Chain Checklist

How to add a chain to Core. The unit of integration is a dedicated `crates/gem_<chain>` crate — a chain owns its provider behavior and its signer behind one boundary, even when it reuses another chain family's plumbing. Family crates stay chain-agnostic; new chains compose their exported building blocks.

## The boundary rule

- All chain-specific behavior lives in `crates/gem_<chain>`: provider trait impls or extension seams, signing, fee logic, RPC mapping fixups, testkit mocks, testdata fixtures.
- Shared-family crates (`gem_evm`, `gem_bitcoin`, `gem_cosmos`) stay chain-agnostic. If a new chain needs a family crate's building blocks, export them as `pub` functions and compose them in the new crate — never add `match chain` branches for the new chain inside the family crate.
- Dispatch happens once at the factories: `settings_chain::build_provider`, `gemstone::gateway::chain_factory`, `gemstone::signer::GemChainSigner`, and `settings_chain::broadcast_providers` — by `ChainType` for families, diverging on `EVMChain::<Chain>` inside the Ethereum arms for chains with a dedicated provider or signer seam (Tempo). Everything downstream is virtual through `ChainTraits` / `ChainSigner`.

## Two integration shapes for EVM-family chains

`EthereumProvider` takes `EvmProviderExtensions` — injection seams for the behaviors chains most often customize. Pick the smallest shape that fits:

1. **Extension seams only** — the chain is a standard EVM chain plus extras. Implement the seam traits in the chain crate and wire them in the `evm_provider_extensions` helpers (`settings_chain/src/lib.rs` and `gemstone/src/gateway/chain_factory.rs`, kept in sync): `EvmFeeCalculator` (`gem_optimism` OP-stack L1-fee oracle), `EvmStakingClient` + `ProtocolParser` (`gem_monad`, `gem_everstake`, `gem_bsc`), `EvmSigner` for chain-specific transaction formats (Tempo, wired via `evm_signer` in `gemstone/src/signer/chain.rs`). No new `ChainType`, no provider wrapper.
2. **Provider wrapper** — the chain redefines flows that have no seam (balances, transaction params, status, per-transaction mapping). Wrap `EthereumProvider`, delegate what is family-generic, override the rest — and still inject the seams that do fit so the shared load flow stays in `gem_evm`. `gem_tempo` is the reference: it injects `TempoFeeCalculator` as the `EvmFeeCalculator` seam, rewrites native transfers into pathUSD token transfers and delegates the whole load flow, and overrides native balances (pathUSD `balanceOf`), transaction status (token-scaled fees), and transaction mapping (receipt `fee_token` + pathUSD→native collapse).

## Steps

1. **Primitives**: add the `Chain` variant, chain config entry (`chain_type`, `fee_unit_type`, `evm` block), asset constants, explorer module, and derivation expectations. The chain keeps its family's `ChainType` — family-wide behavior (addressing, WalletConnect namespace, fee mapping, timeouts) follows automatically, and per-chain divergence rides in config (`fee_unit_type`, `weth_contract`, `chain_stack`) or the chain crate. Do not add a new `ChainType` for a family member; it forces `Ethereum | <Chain>` dual arms through core and both apps.
2. **Crate**: create `crates/gem_<chain>` (auto-included by the `crates/gem_*` workspace glob) with feature flags mirroring the family pattern: `rpc`, `signer`, `testkit`, `chain_integration_tests`.
   - `provider.rs` (wrapper shape only): a `<Chain>Provider` implementing the `chain_traits` traits — delegate what is family-generic, override what the chain redefines.
   - Seam impls: `<Chain>FeeCalculator: EvmFeeCalculator`, `<Chain>StakingClient: EvmStakingClient`, `ProtocolParser` for indexing, as needed.
   - `signer/`: a `<Chain>Signer` implementing the `EvmSigner` seam when transaction formats differ — only the overridden operations, everything else stays in `EvmChainSigner` (for Tempo: native transfers as pathUSD ERC-20 calls, swaps as type `0x76` batched-call transactions with a `fee_token`).
   - `testkit.rs`: mocks and shared test constants, following the `mock_*` conventions in [tests.md](tests.md).
3. **Wiring**: seam-only chains extend the `evm_provider_extensions` helpers; wrapper chains diverge inside the factories' `ChainType::Ethereum` arms on `EVMChain::<Chain>` (provider) and on `Chain::<Chain>` in `GemChainSigner` (signer).
4. **Equality gates**: family-wide `chain_type() == ChainType::Ethereum` gates now cover the chain automatically — audit each one the new chain newly satisfies (swap provider capability checks especially) and confirm the per-chain asset lists or route mappings still exclude it where unsupported. Chain-specific behavior gates on `Chain::<Chain>` (e.g. uniswap v4 gas limit for Tempo).
5. **Generated models**: run `just generate` — typeshare (`ChainType`, `Chain`) and UniFFI bindings must ship in the same change or the apps crash on unknown enum values.
6. **Apps**: regenerate models; the family `ChainType` means existing fee-mapping, namespace, and address-comparison switches already cover the chain. Both apps perform validation, derivation, and signing through gemstone, so most chain behavior needs no app code.
7. **Docs**: update [FEATURES.md](../docs/FEATURES.md) tables (chain capabilities, WalletConnect, indexing, swap providers) in the same change.

## Fee assets

`TransactionFee.fee_asset` is a required `AssetId` end to end — every constructor takes it explicitly (native asset ID of the chain for standard chains), and a chain fee calculator sets it when fees are paid in another asset (Tempo). No optional field, ID reconstruction, or default fill-in at the FFI boundary: apps always receive the fee asset ID selected by the calculator.
