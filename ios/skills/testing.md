# Testing

Use when adding, changing, or running iOS tests, or registering a new test target.
## Execution Rules

- Always run tests through the iOS `justfile`
- Default commands:
  - `just test`
  - `just test <TARGET>`
  - `just build-for-testing` followed by `just test-without-building` for repeated test-debug loops
  - `just test-integration` or `just test-ui` for the iOS integration suite
- Run the narrowest relevant target while iterating, then finish with the appropriate broader validation

## New Test Targets

A test target only runs if it is registered in all three places:

1. `.testTarget` in the package's `Package.swift`
2. The package is referenced in `Gem.xcodeproj` (Packages group)
3. An entry in `GemTests/unit_frameworks.xctestplan`

`swift test` inside the package and Xcode's package scheme bypass the test plan, so green there proves nothing about CI. xcodebuild silently ignores targets missing from the plan and plan entries pointing at deleted targets. After adding a test target, verify with `just test <TARGET>` from `ios/` and confirm the target's tests appear in the output.

## Test Structure

- Keep test names short and descriptive, for example `showManageToken`
- Keep tests concise, usually one behavior with a small number of assertions
- Skip trivial tests that only restate obvious behavior

## Mocks

A mock exists once, beside its type, so every test reuses it instead of rebuilding the value.

- Models, view models, services, and doubles used by tests come from `static func mock(...)` in the `TestKit` of the package or feature that owns the type (`Type+PrimitivesTestKit.swift`, `Features/<Feature>/TestKit/<ViewModel>+TestKit.swift`). Defaults are the usual case; expose only the parameters current tests vary
- A test file declares no fixture of its own: no `makeX()`, `private func wallet()`, local `extension Type { static func mock }`, or helper that assembles a value, even for one test. Search the owning `TestKit` first and extend an existing mock rather than adding a near-duplicate; merge near-duplicate mocks into one with sensible defaults and delete mocks nothing calls
- An internal view model keeps an internal mock in its feature `TestKit`, imported with `@testable import <Feature>TestKit`; never copy the construction into the test
- The test keeps literal inputs and one-off overrides passed at the call site

## Formatting

- Use direct assertions for short cases
- Break long mock setup into multiline formatting when it improves readability
- Avoid explanatory comments in tests
