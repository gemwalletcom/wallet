# Testing

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
- Do not create unit tests that spin up ad hoc local HTTP/TCP test servers; use TestKit mocks, deterministic fixtures, or integration tests instead

## Mocks

- Prefer existing `TestKit` mocks over ad hoc mock services
- If a mock does not exist, add it in the appropriate `TestKit`, not inline in the test file
- Prefer `.mock()` style helpers and small deterministic fixtures

## Formatting

- Use direct assertions for short cases
- Break long mock setup into multiline formatting when it improves readability
- Avoid explanatory comments in tests
