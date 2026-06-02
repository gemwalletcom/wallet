# Engineering Principles

These rules govern the monorepo unless a platform guide gives a stricter local rule.

## Clean Code Principles

- Touch only what the task requires. Adjacent "improvements" — formatting, comment cleanup, drive-by refactors — go in their own PR or stay out
- Review for simplification before finishing: reduce duplication, extract helpers only when they earn their keep, and remove dead code
- Follow YAGNI: do not add behavior until the task needs it
- Keep types and functions single-purpose
- Do not write code comments. Convey intent through clear names and structure; if a piece of code seems to need a comment, rename or restructure it instead. The only exceptions are comments the compiler or tooling requires (e.g. attributes, lint directives, license headers).
- Avoid unclear abbreviations in code names. Write full domain terms such as `transaction` instead of `tx`, except when preserving external protocol field names, database columns, or URLs verbatim.
- Use intent-specific names for APIs and helpers. A function name should state the domain action and expected output in the language of the codebase, for example `parse_destination_tag`, `build_transfer_message`, `sign_trust_set`, or `map_balance_assets`. Generic verbs such as `process`, `handle`, `manage`, `perform`, `execute`, and `resolve` usually hide the contract; keep them only when a framework or protocol owns the signature.
- Before copying a nearby pattern, understand why it exists. If you cannot, ask before copying — copying patterns whose purpose you do not understand is how dead conventions spread
- Keep API surface small: only make things public when they need to be public

## Code Review Standards

- Verify new code is actually used. Unused additions are a signal that the change guessed wrong about what was needed
- Check that copied patterns still fit the local context
- Look for regressions in behavior, not just compile errors
- Tests must verify intent, not just behavior. A test that still passes when the function returns a hardcoded constant is a tautology — fix the assertion or the function under test
- For bug fixes, add or update the smallest meaningful unit test that proves the real behavior with minimal setup and assertions; avoid testing trivial primitives or implementation details
- If a shared product flow changes on only one platform, surface the parity gap explicitly — do not assume the other platform will be picked up later
