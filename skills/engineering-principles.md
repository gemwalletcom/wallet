# Engineering Principles

These rules apply across the monorepo unless a platform guide gives a stricter local rule.

## Clean Code Principles

- Keep changes minimal and focused on the task
- Review for simplification before finishing: reduce duplication, extract helpers only when they earn their keep, and remove dead code
- Follow YAGNI: do not add behavior until the task needs it
- Keep types and functions single-purpose
- Prefer clear names over explanatory comments
- Avoid cargo-culting nearby patterns without understanding why they exist
- Keep API surface small: only make things public when they need to be public

## Code Review Standards

- Verify new code is actually used
- Check that copied patterns still fit the local context
- Look for regressions in behavior, not just compile errors
- Prefer tests that verify real usage paths over coverage-only tests
- Call out feature-parity gaps when a shared product flow changes on only one platform
