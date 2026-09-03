# Engineering Principles

These rules govern the monorepo unless a platform guide gives a stricter local rule.

## Fix Causes, Not Symptoms

This is the first rule: when something is broken, find why it broke and fix that.

- Trace a failure to the layer that owns the invariant, and fix it there. A guard in the caller that hides a bad value produced upstream leaves the bug in place for the next caller
- Do not patch a crash with a `nil`/`null`/`Option` check, a `try?`, a `catch` that swallows, a retry, a `sleep`, or a defensive default unless the absence, failure, or delay is a real expected state you can name. If it is, say so in the code's structure; if it is not, fix the producer
- Do not reach for a wider fix than the cause justifies either — a real one-line cause gets a one-line fix
- When a test fails, change the code until the assertion holds. Loosening the assertion, deleting the case, or special-casing test input hides the defect instead of removing it
- If the true fix is out of scope, say so explicitly: apply the smallest containment, state what the actual cause is, and flag it for follow-up. Never present a symptom patch as a fix
- Duplicated bugs mean a duplicated cause. Two similar fixes in two places usually point at one shared root that should be fixed once

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
