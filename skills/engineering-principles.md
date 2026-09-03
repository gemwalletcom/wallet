# Engineering Principles

Applies to every task while writing code. These rules govern the monorepo unless a platform guide gives a stricter local rule.

## Fix Causes, Not Symptoms

The most important rule in this file. Before changing anything, ask why the failure was possible at all, and keep asking until the answer names a layer you can fix.

- Who produced this value or state? Fix it there (the parser that accepted the input, the mapper that built the value, the Core rule the apps consume, the config that declared support), not in the caller that noticed it, and delete the downstream guards the fix makes unnecessary (see § No Over-Defensive Code)
- Is this condition real? A null check, swallowed error, retry, wider timeout, or sleep is a fix only when you can name the state it handles
- Why is this test failing? Change the code, not the assertion
- Is this the only place? Two similar patches usually mean one shared cause; check whether the failure recurs through another entry point, chain, provider, or timing
- If the real fix is out of scope (another repository, a provider, shipped clients, a scope the user set), propose it in the handoff with the layer, the change, and what it would remove. Ship a symptom patch only if it is safe, minimal, and labeled temporary; never present it as the fix
- The regression test reproduces the cause at the producer, not the guard at the consumer

## Clean Code Principles

- Touch only what the task requires; adjacent improvements go in their own PR or stay out
- YAGNI: no behavior until the task needs it; keep types and functions single-purpose
- No code comments. Convey intent through names and structure; if code seems to need a comment, rename or restructure it. Only compiler- or tooling-required comments (attributes, lint directives, license headers) are exceptions
- Full domain terms in names (`transaction`, not `tx`) except when preserving external protocol fields, database columns, or URLs verbatim
- Intent-specific names that state the domain action and result (`parse_destination_tag`, `build_transfer_message`, `map_balance_assets`). Generic verbs such as `process`, `handle`, `manage`, `perform`, `execute`, and `resolve` hide the contract; keep them only when a framework or protocol owns the signature
- Understand a nearby pattern before copying it; copying patterns you cannot explain is how dead conventions spread
- Small API surface: public only when it must be
- Immutable bindings (`let`, `val`, non-`mut`); mutation only where ownership requires it, in the narrowest scope
- When two patterns contradict (iOS vs. Android on a shared flow, two error-mapping styles in `core/`, parallel providers), do not blend them. Pick the more recent or better tested one, say why, and flag the other for follow-up

## Generic Over Special-Cased

- Solve the class, not the instance. When a fix special-cases one chain, provider, asset, or screen, name the class it belongs to and put the rule at that level (a `ChainConfig` field, a provider seam, a shared mapper, a Core service rule) when two members need it or the class is known to grow. Otherwise keep the special case small, named, and next to the class it refines.
- Generic means the right level of abstraction, not speculation. Do not pre-build for members that do not exist or widen a signature "for later".

## No Over-Defensive Code

- Validate once at the trust boundary (external payloads, RPC responses, user input, FFI edges), then trust the types inside. Redundant null checks, re-validation of typed values, catch-all handlers, and `unwrap_or`-style fallbacks that hide a failure are review findings, not safety.
- Do not duplicate an invariant across layers. When Core or a typed shared contract owns a rule, the apps consume the result.
- Fail fast and loud where the app cannot safely continue; fail closed on security state. The non-negotiable safety rules (exhaustive matching, no panics in production, typed errors) live in the platform guides; anything beyond them needs a reason.

## Tests

- Tests verify intent, not just behavior. If the test still passes after the business rule flips, it is a tautology: fix the assertion or the function under test
- For a high-impact bug, add the smallest test that materially reduces regression risk; skip trivial, framework, formatting-only, and purely visual coverage unless asked or already cheap
- "Tests pass" is not a green light if any were skipped, marked expected-failure, or gated behind features you did not run. Report what you executed
- Unit tests never spin up ad hoc HTTP/TCP servers. Use the platform testkit fixtures, pure mappers and parsers, or injected clients; when network behavior matters, use the gated integration tests

Review criteria live in `skills/code-review.md`; this file is for writing code.
