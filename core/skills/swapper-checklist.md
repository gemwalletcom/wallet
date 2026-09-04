# Swapper Provider Integration Checklist

Audit a swapper provider in `crates/swapper/src/<provider>/provider.rs` against this checklist by reading the provider code and related files. Approval spender and router authority is owned by [Security](../../skills/security.md).

### 1. get_quote Performance
- [ ] Quote returns in under 1 second in normal conditions
- [ ] Same-chain swaps prefer onchain math and node RPC over centralized quote APIs when practical
- [ ] Cross-chain swaps may use centralized/provider APIs when that is the protocol integration surface
- [ ] Avoid unnecessary chained API/RPC calls before quoting; batch or cache calls where practical

### 2. get_quote_data Correctness
- [ ] Input amount: `get_quote()` applies `quote_value_after_reserve_by_chain(request)?` from `crate::fees` and uses the adjusted value for both the provider request and `Quote.from_value`; `get_quote_data()` reads `quote.from_value`, never `quote.request.value`. The two differ when `use_max_amount` is true, and `RESERVED_NATIVE_FEES` (`fees/reserve.rs`) needs an entry for every supported chain
- [ ] New chain or asset support is traced end to end before it is declared supported (`skills/task-workflow.md` § 2), with a correct transaction built for each supported chain type; non-EVM transaction fields are never forced into an EVM request shape
- [ ] Approval is checked for the specific chain and token standard, including chains whose native asset is itself an ERC-20 (Celo, Tempo pathUSD): branch on `EVMChain::native_asset_contract()` / `native_asset::requires_native_wrapping`, not `is_native()` alone, and do not extend that mechanism to a chain whose native and token units differ in decimals without scaling amounts. Spender and router authority follows [Security](../../skills/security.md): an independent provider-and-chain policy, with exact amount never a substitute
- [ ] Gas limit with a pending approval: the swap transaction is never estimated before the approval lands, so the quote's `gas_limit` is the only source. EVM: `approval::get_swap_gas_limit_with_approval` with the provider's per-step estimate plus a buffer, falling back to `DEFAULT_EVM_SWAP_GAS_LIMIT`. Tron: a static energy estimate must be present (`DEFAULT_TRON_SWAP_ENERGY_LIMIT` or the provider's value) because the shared `gem_tron` preload cannot simulate against zero allowance; `approval: Some` with `gas_limit: None` fails every first swap

### 3. Auto Slippage
- [ ] Slippage comes from the provider API or a sensible default through `apply_slippage_in_bp`, never a hardcoded value
- [ ] Requested slippage BPS is forwarded exactly when the provider accepts it; price impact fields are not reused as requested slippage

### 4. Referral Fee
- [ ] Fee BPS constant defined in `crates/swapper/src/fees/mod.rs` and passed to the provider API in quote requests
- [ ] Referral fee token side selection is explicit and tested (prefer native/wrapped native, then stablecoins, over arbitrary route tokens)

### 5. Vault Addresses & Transaction Indexing
- [ ] `get_vault_addresses()` returns every deposit address (user sends to vault) and send address (vault sends to user) the provider uses
- [ ] Each address is confirmed against live on-chain activity (explorer or node API) and the provider's authoritative source (backend repository, chain spec, or live chains endpoint), not the docs portal alone. A documented "vault" can be a deployed-but-unused contract, and PR test vectors may be testnet addresses
- [ ] Deposit addresses enable `is_cross_chain_swap()` detection in `cross_chain.rs`; send addresses enable `is_from_vault_address()` detection for incoming swap completions
- [ ] If the provider requires memo/payload validation (like Thorchain), `is_valid_swap_transaction()` handles it

### 6. Swap Result Tracking
- [ ] `get_swap_result()` maps provider status to `SwapResult` / `SwapStatus` and handles completed, pending, and failed/refunded states

### 7. Error Mapping
- [ ] Provider-specific errors map to the existing typed `SwapperError` variants: a minimum-amount rejection becomes `SwapperError::InputAmountError { min_amount }` with the amount converted to base units; unsupported assets and routes use their own variants instead of a generic failure

### 8. Supported Assets
- [ ] `supported_assets()` returns the correct `SwapperChainAsset` list using constants from `primitives::asset_constants`, not inline `AssetId::from_token`
- [ ] Provider chain identifiers that cannot be derived from `Chain` config (Relay's Tron `728126428` and Solana `792703809`) live in the provider's asset mapping with a unit test
- [ ] Providers that share one engine across networks (`THORChainNetwork::{Thorchain, Mayachain}`) keep supported chains, router allowlists, and API prefixes explicit per network: `ChainName::supported(network)` selects `THORCHAIN_NAMES` or `MAYACHAIN_NAMES`. Add a chain to one table only; never derive a network's chain list from the union of all known chain names

### 9. Tests
- [ ] Unit tests cover quote parsing, error mapping, and asset mapping, with fixtures in `<provider>/test/`
- [ ] Live integration tests cover provider quotes and results, gated behind `#[cfg(feature = "swap_integration_tests")]`, and use funded wallets that hold the balance or token account the route needs; placeholder keys make providers answer "no route" and mask real failures
- [ ] Avoid mock clients/tests that only assert mocked behavior and do not protect provider logic
