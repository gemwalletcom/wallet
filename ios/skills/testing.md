# Testing

Use when adding, changing, or running iOS tests, or registering a new test target.
## Execution Rules

- Always run tests through the iOS `justfile`
- Default commands:
  - `just test-package <Package>` as shorthand for `just test <Package>Tests`
  - `just check-test <Package>` for host-compatible packages without Gemstone linkage
  - `just test`
  - `just test <TARGET>` for app-hosted tests and app-plan registration checks
  - `just build-for-testing` followed by `just test-without-building` for repeated test-debug loops
  - `just test-integration` or `just test-ui` for the iOS integration suite
- Run the narrowest relevant target while iterating, then finish with the appropriate broader validation
- Commands above run from `ios/`; from the repo root use `just ios <recipe>`. Read the owning `Package.swift` for package and test-target names. Confirm a nonzero test count and the expected suite in the output

Performance benchmarks are opt-in: `TEST_RUNNER_BENCHMARKS=1 just test-package GemstoneServices` forwards `BENCHMARKS=1` to the simulator test process. Normal tests retain keystore correctness coverage and skip the repeated KDF timing benchmark; do not reduce cryptographic parameters to speed tests up.

## New Test Targets

A package test target participates in the app/CI test plan only if it is registered in all three places:

1. `.testTarget` in the package's `Package.swift`
2. The package is referenced in `Gem.xcodeproj` (Packages group)
3. An entry in `GemTests/unit_frameworks.xctestplan`

`just check-test` bypasses the app test plan and does not verify CI registration. Both `just test-package` and `just test` use the app test plan. xcodebuild silently ignores targets missing from the plan and plan entries pointing at deleted targets. After adding a test target, verify with `just test <TARGET>` from `ios/` and confirm the target's tests appear in the output.

## Test Structure

- Keep test names short and descriptive, for example `showManageToken`
- Keep tests concise, usually one behavior with a small number of assertions
- Skip trivial tests that only restate obvious behavior

## Async Work

A test awaits work, never time. `Task.sleep`, a polling loop, or a settle helper turns runner load into a failure and proves nothing when it passes.

- A view model method that does async work is `async`; the view owns the `Task` (`Button { Task { await model.onDeleteNode() } }`). The test calls `await model.onDeleteNode()` and asserts, including the negative case where nothing should have happened
- Work the model must keep running on its own (a debounced lookup) keeps its `Task` in the model and exposes the handle; the test awaits `model.nameRecordTask?.value` or asserts the synchronous state the request leaves behind
- A test never needs to be in the middle of a request. A rule about a late or stale result belongs to Core (`GemSwapSession.on_quote_results` drops a result for a request that is no longer current) and is tested there with data; a lookup that a newer one supersedes is cancelled by the model (`ReceiveViewModel.selectNetworkTask`), so the test makes both calls back to back and awaits the handle
- An external stream (a database observation) is awaited through `withObservationTracking` and a continuation, one change at a time until the state is there; the only deadline is the `.timeLimit(.minutes(1))` trait on the test

## Mocks

A data mock exists once per type and is the same on both apps, so every test reuses it instead of rebuilding the value.

- A generated model gets its `static func mock(...)` from `just generate-models` once it is listed under `mocks:` in `core/bin/generate/remote_types.yml`, written to `Primitives/TestKit/GeneratedMocks.swift` for a TypeShare model or `GemstonePrimitives/TestKit/GeneratedMocks.swift` for a Gemstone record or enum (Android gets the same mock). A listed enum's mock is its first case with default values. To mock one, add it to that list and regenerate; never hand-write it. Every field is a parameter with a default, in declaration order: `""`, zero, `false`, `nil`, `[]`, the first enum case (`.bitcoin` for `Chain`), the epoch for a date, and the nested type's or identifier's own mock. A field the rules cannot fill fails generation and names it; list its type, or override that field in `mocks:` only when the rule would build an invalid value
- A type the generator cannot reach (identifiers, hand-written app types) gets its hand-written mock in the package's one `TestKit/Mocks.swift`, under the same rules. View models, services and other doubles keep `static func mock(...)` or a class in the `TestKit` of the package or feature that owns them (`Features/<Feature>/TestKit/<ViewModel>+TestKit.swift`)
- No named presets (`mockEthereum()`, `mockWithChains()`): a test passes the values it asserts on, so every input it depends on is visible at the call site
- A test file declares no fixture of its own: no `makeX()`, `private func wallet()`, local `extension Type { static func mock }`, or helper that assembles a value, even for one test. Search the owning `TestKit` first and extend an existing mock rather than adding a near-duplicate; merge near-duplicate mocks into one with sensible defaults and delete mocks nothing calls
- An internal view model keeps an internal mock in its feature `TestKit`, imported with `@testable import <Feature>TestKit`; never copy the construction into the test
- The test keeps literal inputs and one-off overrides passed at the call site

## Formatting

- Use direct assertions for short cases
- Break long mock setup into multiline formatting when it improves readability
- Avoid explanatory comments in tests
