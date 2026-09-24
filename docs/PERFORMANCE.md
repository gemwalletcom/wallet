# Performance

## Goal

Wallet, asset, transaction, confirmation, and swap screens should feel immediate on iOS and Android: no freezes, smooth transitions and scrolling, responsive typing, and refreshes that keep content usable.

Core owns domain logic and orchestration; apps own rendering, observation, and navigation. Fix the layer causing the delay. Follow [Architecture](ARCHITECTURE.md), the [screen-service map](ARCHITECTURE.md#screen-services), and [Security](../skills/security.md).

## Principles

1. **Keep heavy work off the UI thread.** Network calls, storage, JSON and image decoding, large sorts, and transaction preparation must not block rendering. An `async` function does not guarantee background execution; check the actual thread, including FFI callbacks.
2. **Render prepared state.** Keep SwiftUI `body`, Compose composition, and row getters cheap. Prepare display values when their inputs change. Update affected rows instead of rebuilding the whole screen.
3. **Reduce FFI work.** FFI is the Swift/Kotlin ↔ Rust boundary. Return coherent screen or section records, batch related reads, decode once, and avoid a call per property per render. Measure conversion and copying as well as Rust execution; keep payloads bounded.
4. **Run independent work in parallel.** Start independent requests together in the existing owner, fetch shared inputs once, and bound concurrency. Preserve real dependencies and required transaction ordering. Verify overlap in a trace.
5. **Show useful content early.** Render available local data while refreshing. Optional charts, images, or metadata should not delay the main content. Show loading and errors honestly; cached data is not newly verified data.
6. **Keep scrolling stable.** Use lazy lists, bounded pages, stable row IDs, and fixed image placeholders. Preserve scroll position and focus during updates. Reuse formatters, downsample images, and avoid full-list sorting or animation on every price tick.
7. **Stop obsolete work.** Cancel old requests and release screen observers. Reject late results for a previous wallet, asset, amount, or quote even when cancellation is unavailable. Keep required transaction tracking alive through its service after the screen closes.
8. **Reuse existing refresh and cache policies.** Avoid duplicate requests, unnecessary writes, and new polling timers. Bound caches and invalidate derived values when their inputs change. For swap, cached routes are hints only; quotes, prices, balances, approvals and transaction data are never cached, and every quote uses the current amount and live chain state.
9. **Preserve correctness.** Performance changes must retain amount, recipient, chain, fee, simulation, approval, authentication, and signing checks. Prevent duplicate sends and stale selectable quotes. Return after broadcast and recording the pending transaction; track chain finality in the background.

## Primary screen checks

The owner column is where the work actually happens now: a journey that misses a target is fixed there, not in the view model that reads it, and a fix lands for both apps at once.

| Screen | Owner | What to verify |
|---|---|---|
| Wallet | [`GemWalletHomeService`](../core/gemstone/src/services/wallet_home/mod.rs) — `refresh` joins balances and discovery | Show cached assets promptly. Refresh balances and discovery concurrently. Scroll during price updates and switch wallets during refresh without showing stale data. |
| Asset | [`GemAssetDetailsService`](../core/gemstone/src/services/assets/details.rs) — `refresh` joins the per-step loads and records each failure | Load independent chart, history, and metadata work concurrently. Keep the header usable and scrolling stable while results arrive. Test rapid chart-range changes. |
| Transactions | [`GemTransactionsService`](../core/gemstone/src/services/transactions/mod.rs) with the app's observed store read | Load bounded pages, preserve row position during status updates, and avoid rebuilding the full history. Test pagination, filters, and repeated detail navigation. |
| Confirmation | [`GemConfirmTransferService`](../core/gemstone/src/services/confirm/mod.rs) and its `GemConfirmation` | Overlap independent state, fee, and simulation work while respecting their dependencies. Keep controls responsive. Test slow preload, fee changes, errors, authentication cancellation, and repeated send taps. |
| Swap | [`GemSwapSession`](../core/gemstone/src/services/swap/session.rs) — it owns the debounce, the refresh interval and which quote is selectable | Update typing immediately and debounce quote requests through the existing policy. Run eligible provider requests concurrently. Test amount edits, Max, pair reversal, slippage, failed providers, cold/warm routes, and swap-to-confirm navigation. Only current, valid quotes may be selected. |

Every number below is a target, not a measurement: nothing in this repository has been profiled against them on a physical device, and no automated gate enforces them. Treat a journey as unverified until someone records a baseline under **How to test**.

## Initial targets

These are starting targets, not measured results or existing automated gates. Establish a baseline per platform and device. p95 means 95% of samples finish within the target.

| Metric | Target |
|---|---|
| Tap, typing, or refresh feedback | p95 ≤ 100 ms to the first visible response |
| Warm screen with local data | p95 ≤ 200 ms to useful content |
| Applying a received update | p95 ≤ 100 ms from result arrival at the app boundary to the displayed frame |
| Scrolling and transitions | At least 99% of frames meet the active display deadline; investigate every reproducible visible hitch |
| Repeated navigation | No sustained growth in retained screens, tasks, observers, or requests after settling |

A frame interval is about 16.7 ms at 60 Hz or 8.3 ms at 120 Hz; the UI thread has only part of that time. Measure network-dependent readiness separately from immediate feedback. Keep cold and warm results separate, and compare each platform using its own frame metrics.

## How to test

1. **Choose a journey.** Use the table above and deterministic test wallets. Cover empty, typical, and large datasets; fast, slow, offline, failed, and out-of-order responses. Use no real funds or secrets.
2. **Measure on devices.** Use release-like builds on an older supported and a current physical device per platform. Keep device, OS, data, network, cache state, power mode, and temperature comparable between baseline and candidate.
3. **Capture a baseline.** On iOS, use Instruments Time Profiler, Hangs/Hitches, and SwiftUI analysis. On Android, use Macrobenchmark frame metrics and system traces. Trace UI work, FFI conversions and calls, storage, RPCs, and lock waits to find the delay.
4. **Repeat under load.** For warm timings, start with 5 warmups and 30 measured runs. Scroll for 30 seconds during data updates and repeat navigation 20 times. Test rapid input changes, cancellation, wallet switching, and background/foreground. Reset state deliberately for cold runs.
5. **Fix and compare.** Change the responsible owner, then repeat the same scenario. Compare median, p95, worst stalls, missed frames, FFI/request counts, and settled memory. Separate network wait from local work. Add deterministic tests for ordering and stale-result bugs.
6. **Check both apps.** Shared changes affecting these journeys need evidence from iOS and Android. Run the applicable [Quality Checks](../skills/quality-checks.md). Before release, exercise all five primary screens. A visible freeze, scroll jump, incorrect result, or security regression is a failure even if average timings improve.

Use [Maestro](../skills/testing-maestro.md) or native UI tests for journey correctness. Simulators and emulators help reproduce issues; physical devices provide performance evidence. Keep authentication enabled when measuring confirmation. Until a benchmark harness exists for a journey, record the manual profiler setup and steps.

Tool references: [Apple responsiveness](https://developer.apple.com/documentation/xcode/improving-app-responsiveness), [SwiftUI analysis](https://developer.apple.com/documentation/swiftui/performance-analysis), [Android Macrobenchmark](https://developer.android.com/topic/performance/benchmarking/macrobenchmark-overview), and [frame metrics](https://developer.android.com/topic/performance/benchmarking/macrobenchmark-metrics).

## Report with the change

- Journey, fixture, baseline and candidate commits, device, OS, build, network, and cache state.
- Exact commands or profiler setup, sample count, before/after measurements, and trace location.
- Cause, owning layer, fix, correctness checks, and any skipped scenarios or platform gaps.

Keep traces free of keys, seed phrases, authentication tokens, signing payloads, and identifying wallet data. Report unmeasured performance as unverified.
