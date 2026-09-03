# Engineering Principles

These rules govern the monorepo unless a platform guide gives a stricter local rule.

## Fix Causes, Not Symptoms

- Trace a failure to the layer that owns the invariant and fix it there, not in the caller that noticed it
- A null check, swallowed error, retry, or sleep is a fix only when the state it handles is real and you can name it
- When a test fails, change the code, not the assertion
- If the real fix is out of scope, say what the cause is and flag it. Never present a symptom patch as a fix

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

## Review Guidance

Use `skills/code-review.md` for repository-specific review criteria. Keep this file focused on implementation principles that apply while writing code.
