# Swapper

`GemSwapper` selects compatible providers, preloads reusable route data, and requests live quotes.

## Quote flow

`GemSwapService` calls `preload_routes` immediately before `get_quote`; iOS and Android request quotes through that Core service. Asset selection does not start preload work. Providers without reusable route discovery use the default no-op implementation.

```mermaid
flowchart LR
    Quote["iOS / Android quote request"] --> Preload["Preload route discovery"]
    Preload --> Cache["In-memory route cache"]
    Cache --> Live["Live amount quote"]
```

The current providers preload:

- Uniswap V3 and V4 pool existence.
- Cetus pool IDs and shared-object versions.
- STON.fi routers, pools, and jetton wallets.
- Chainflip broker asset IDs and minimums; live quotes use the Broker as a Service native quote endpoint.
- THORChain and MayaChain inbound addresses.

## Shared cache

EVM, Sui, and TON use the generic `RouteCache` with a one-day process-local TTL. It stores direction-independent pool discovery and direction-specific winning-route hints. Providers supply only their protocol-specific keys, probes, and candidates.

A route is only a hint. Every quote still uses the current amount and live chain state. If the cached route fails, the provider tries other discovered routes.

Exact concurrent RPC requests are joined by the same Gemstone coalescer used by `GemGateway`. Quote results, prices, balances, approvals, and transaction data are never cached.

The Gem API adds every Swaps.xyz quoted deposit address to the seven-day vault watchlist while proxying the action request. Source-chain memo and destination-tag values returned as `tx.toExtra` are preserved in `SwapQuoteData.memo`; Sui deposits also receive a prebuilt transfer payload. Status polling uses the confirmed source-chain transaction hash directly; clients do not register broadcasts or persist a provider transaction-ID mapping.

## Max amount

A max swap of a native coin cannot spend the whole balance: the transaction still pays the network fee, and some providers attach native value on top of the swapped amount. Core owns that arithmetic; the apps only flag `use_max_amount` when the entered value equals the available balance.

- `Swapper::get_quotes` reduces the value quoted to every `SwapAmountMode::Fixed` provider by the chain's `RESERVED_NATIVE_FEES` entry (`fees/reserve.rs`), which covers the wallet's own fee.
- A provider whose transaction attaches native value on top of the swapped amount subtracts that attachment itself before quoting, because only it knows the number. STON.fi keeps the forward gas its TON-to-jetton message carries (0.31 TON, the v2 amount, which also covers v1 routes) out of a max native quote, and reports the chain reserve plus the attachment as the minimum when the balance cannot cover them; a builder test keeps that constant above what either router version attaches.
- The chain preload turns the real attachment into the confirm fee: TON's `calculate_transaction_fee` reads it from the quote data, the message value minus the quoted amount for a native swap and the whole message value for a jetton swap, instead of a constant, so the balance check at confirm is exact for the route that was quoted.
- `TransferAmountInput::calculate` never trims a max amount the transaction cannot carry. A `Contract` swap sends the quoted value, so the confirm shows that value or an insufficient-balance error, never a smaller number than the one signed. Transfer-type swaps still trim by the fee at confirm.

Issue #1154 was the missing attachment: a max TON swap was quoted at the balance minus 0.02 TON, STON.fi attached 0.31 TON, the confirm displayed a trimmed amount, and the wallet skipped the message on chain.

- A flexible-input provider can report a minimum input above what a max swap can send once the network fee is paid: NEAR Intents' minimum is the quoted amount minus slippage, so a 50,200-sat Bitcoin max with a 537-sat fee lands 286 sats under it. The confirm keeps the sendable amount and reports `BelowMinimumValue` with the provider minimum, not an insufficient balance, so the user can lower the amount (issue #1147). The UTXO fee reserves that once buffered this at quote time were removed on 2026-06-11 (`54f237b00e`) together with mode-aware quoting, which skips reserves for flexible providers.

Accepted: for a transfer-type max swap the confirm trims by the fee only when the fee exceeds the chain reserve, while the EVM and Tron signers always subtract the fee (`SignerInput::swap_value`) and the other signers never do, so the confirmed and signed amounts can differ by up to the fee on those providers. Reviewed on 2026-09-14 and left as is: the difference is bounded by one network fee and only on a max swap, and moving the trim to either side would change a signing path for every provider to close it. Revisit only if a provider reports a rejection that this difference causes.

## Timing

`get_quotes` awaits every eligible provider, so one provider decides how long the screen waits. Measured on 2026-09-15 from a developer machine with `cargo test -p swapper --features swap_integration_tests --lib timing_tests -- --nocapture`, which prints per-provider preload and quote durations for an on-chain and a cross-chain pair, cold and warm.

| Pair | Phase | Cold | Warm | Slowest provider |
| --- | --- | --- | --- | --- |
| ETH → USDC (Ethereum) | preload | 250ms | 0ms | — |
| ETH → USDC (Ethereum) | quotes | 340–510ms | 185ms | OKX (340–510ms cold, 180ms warm) against ~100ms for Uniswap V3/V4 and PancakeSwap |
| ETH → SOL | preload | 360–390ms | 320–330ms | — |
| ETH → SOL | quotes | 1230–1320ms | 1225–1230ms | NEAR Intents (1.22–1.32s), roughly twice Mayan at 530–640ms |

What the numbers say:

- **The quote phase decides, not discovery.** Preload is a fifth of a cross-chain round and none of a warm on-chain one.
- **One provider owns the cross-chain tail.** NEAR Intents costs the same 1.23s cold or warm, so no cache warms it; every other eligible provider has answered by then.
- **Providers that cannot serve the pair still cost.** THORChain and Chainflip spend 320–630ms each on ETH → SOL before reporting no quote. They are not the tail today, only because NEAR Intents is slower.
- **The cross-chain preload never warms.** On-chain discovery drops to 0ms on the second round through `RouteCache`; THORChain and MayaChain inbound addresses are outside that cache and are re-requested every round.

**Transport deadlines are the apps', and they disagree.** Core sets none: `join_all` has no per-provider deadline, and the only way a quote round ends early is the whole future being dropped. iOS uses `URLSession.shared`, whose request timeout is 60 seconds; Android uses a default `OkHttpClient`, which gives up after 10 seconds of connect, read or write and sets no call timeout. The same stalled provider therefore stalls an iOS quote round six times longer than an Android one.

**Cancellation is also asymmetric.** Dropping the `get_quotes` future cancels the pending provider futures in Core. On iOS that reaches the socket, because `URLSession.data(for:)` honours task cancellation. On Android the call is `execute()` inside `withContext(Dispatchers.IO)`: cancelling the coroutine abandons the result but does not interrupt the blocking call, so a superseded quote round keeps its connections busy until each provider answers or its 10-second read timeout fires.

Any change here starts from these numbers. A deadline decides which providers get to compete and is a product policy, not a tuning constant; first-result selection would end ranking; and quote results are never cached by design. None of those belong in a timing fix without their own decision.

## Failure and lifetime

Preloading is best-effort. Completed probes are cached; transport failures and incomplete responses remain missing, so `get_quote` retries them through the provider's normal discovery path. Uniswap falls back to its full quote request set when discovery is unavailable.

The apps keep one `GemSwapper` per process. The cache survives swap screen recreation and resets on process restart.

## Code map

- [Core swapper](../core/crates/swapper/src/swapper.rs), with the timing harness in its `timing_tests` module
- [Route cache](../core/crates/swapper/src/route_cache.rs)
- [Max amount reserve](../core/crates/swapper/src/fees/reserve.rs)
- [Uniswap discovery](../core/crates/swapper/src/uniswap/discovery.rs)
- [Cetus discovery](../core/crates/swapper/src/cetus_clmm/client.rs)
- [STON.fi discovery](../core/crates/swapper/src/stonfi/provider.rs)
- [Gemstone bridge](../core/gemstone/src/gem_swapper/mod.rs)
- [Core quote orchestration](../core/gemstone/src/services/swap/mod.rs)
- [Core screen-facing quote service](../core/gemstone/src/services/swap/quote.rs)
- [RPC coalescing](../core/gemstone/src/alien/coalescing_provider.rs)
- [iOS quote path](../ios/Features/Swap/Sources/ViewModels/SwapSceneViewModel.swift)
- [Android quote path](../android/data/coordinators/src/main/kotlin/com/gemwallet/android/data/coordinators/swap/RequestSwapQuotesImpl.kt)
