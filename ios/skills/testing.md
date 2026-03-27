# Testing

## Execution Rules

- Always run tests through the iOS `justfile`
- Default commands:
  - `just test-all`
  - `just test <TARGET>`
  - `just test-ui`
- Run the narrowest relevant target while iterating, then finish with the appropriate broader validation

## Test Structure

- Keep test names short and descriptive, for example `showManageToken`
- Keep tests concise, usually one behavior with a small number of assertions
- Skip trivial tests that only restate obvious behavior

## Mocks

- Prefer existing `TestKit` mocks over ad hoc mock services
- If a mock does not exist, add it in the appropriate `TestKit`, not inline in the test file
- Prefer `.mock()` style helpers and small deterministic fixtures

## Formatting

- Use direct assertions for short cases
- Break long mock setup into multiline formatting when it improves readability
- Avoid explanatory comments in tests
