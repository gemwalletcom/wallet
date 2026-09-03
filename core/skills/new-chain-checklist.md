# New Chain Checklist

How to add a chain to Core without spreading it through shared code. A chain owns its behavior behind one boundary, even when it reuses a family's plumbing. This file covers adding a chain in Core; removing or disabling one follows [Release Process](../../skills/release-process.md) § Removing or Disabling Support.

## Boundary Rule

- Chain-specific behavior lives in `crates/gem_<chain>`: provider overrides, signing, fee logic, RPC mapping fixups, testkit mocks, testdata fixtures. `gem_tempo` is the reference for an EVM-family chain that diverges.
- Family crates (`gem_evm`, `gem_bitcoin`, `gem_cosmos`) stay chain-agnostic. When a new chain needs a family building block, export it as `pub` and compose it in the chain crate. Never add a `match chain` arm for the new chain inside the family crate.
- A family member keeps the family `ChainType`. Do not add a `ChainType` per chain; see the reverted shape in [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) § 13. Family-wide behavior (addressing, WalletConnect namespace, fee mapping) follows from `ChainType`, per-chain divergence rides in `ChainConfig` fields or the chain crate.
- Dispatch happens once, at the factories: `settings_chain::build_provider` (`crates/settings_chain/src/lib.rs`), `gemstone::gateway::chain_factory`, and `gemstone::signer::chain`. Inside the `ChainType::Ethereum` arm the provider diverges through `TempoProvider::new_or_else(client, fallback)` and per-`EVMChain` staking or gas clients; the signer diverges through `Chain::Tempo => EvmChainSigner::new(TempoSigner)`. Everything downstream is virtual through `ChainTraits` and `ChainSigner`.
- EVM seams to implement instead of forking `gem_evm`: `EvmSigner` (`gem_evm/src/signer/chain_signer.rs`, default `StandardEvmSigner`) for transaction formats, `EvmFeeCalculator` and `EvmStakingClient` (`gem_evm/src/rpc/chain_provider.rs`) injected via `EthereumProvider::new_rpc_only_with_provider`.
- A family member does not inherit every behavior. For a nonstandard EVM-like chain, inspect the transaction envelope and type, chain ID, fee asset and units, nonce, gas estimation, simulation, token semantics, status mapping, swap parsing, signing, and mobile parity explicitly before declaring support.
- Compare `Chain::<Chain>` directly only where behavior is truly per-chain (a swap provider's gas limit for one deployment), never as a substitute for a missing `ChainType` or `ChainConfig` field.

## Steps

1. **Primitives**: add the `Chain` variant, the `ChainConfig` entry in `crates/primitives/src/chain_config.rs` (`chain_type`, `evm`/`stake` blocks, capability flags), default nodes in `node_config.rs`, explorer module, and asset constants. `is_swap_supported`, `is_nft_supported`, and similar flags are the only source of chain capabilities for both apps.
2. **Crate**: create `crates/gem_<chain>` (picked up by the `crates/gem_*` workspace glob) with the family feature layout, mirroring `gem_tempo`: `rpc`, `reqwest`, `signer`, `chain_integration_tests`, `default = []`. Put mocks and `TEST_` constants in `testkit.rs`.
3. **Wiring**: extend the factory arms above and the `settings` chain entries (`settings.chains.<chain>`, `Settings.yaml`) plus `apps/dynode/chains.yml` when the node proxy must route it.
4. **Equality gates**: every `chain_type() == ChainType::Ethereum` (or family) gate now includes the chain. Audit each one it newly satisfies, especially swap provider capability checks and asset lists, and confirm they still exclude it where the provider has no deployment.
5. **Generated models**: run `just generate`. `Chain`, `ChainType`, and `EVMChain` are TypeShare enums; bindings and models ship in the same change or the apps crash on an unknown value.
6. **Apps**: iOS needs the chain image in `Packages/Style/Sources/Images.swift` and the exhaustive switch in `PrimitivesComponents/Sources/Types/ChainImage.swift`; Android resolves chain assets from the generated enum. Search both apps for exhaustive `switch`/`when` over `Chain` before declaring parity. Validation, derivation, and signing already flow through gemstone.
7. **Docs**: update the chain capability, WalletConnect, indexing, and swap tables in [docs/FEATURES.md](../../docs/FEATURES.md) in the same change.
8. **Outside this repository**: chain and token logos in the assets repository, backend redeploy for search, prices, and fiat lists, and the dynode allowlist. List these in the handoff; a chain that builds locally is not live until they land.

## Verification

- `just test gem_<chain>` and `cargo clippy -p gem_<chain> --all-features --all-targets -- -D warnings`; the crate's gated modules do not compile under default features.
- Live read paths (balances, fee rates, transaction load) as `chain_integration_tests` against mainnet with a funded `TEST_` address from the crate's testkit; signing as byte-exact vectors in unit tests.
- Both apps build after `just generate`, and the new chain appears with the capabilities `ChainConfig` declares.
