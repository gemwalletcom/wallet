# Engineering Principles

Use for every code change. Guidance precedence is defined in [AGENTS.md](../AGENTS.md#using-the-guidance).

## Fix Causes, Not Symptoms

Trace the failure to the code that owns the broken invariant. Fix that owner within the task's scope; this does not authorize a wider architectural migration.

- Who produced this value or state? Fix it there (the parser that accepted the input, the mapper that built the value, the Core rule the apps consume, the config that declared support), not in the caller that noticed it, and delete the downstream guards the fix makes unnecessary (see § No Over-Defensive Code)
- Is this condition real? A null check, swallowed error, retry, wider timeout, or sleep is a fix only when you can name the state it handles
- Diagnose a failing test against the intended contract. Fix the implementation when it violates the contract; update the test when its expectation is stale or the task intentionally changes behavior. Never weaken an assertion just to pass
- Is this the only place? Two similar patches usually mean one shared cause; check whether the failure recurs through another entry point, chain, provider, or timing
- If the real fix is out of scope (another repository, a provider, shipped clients, a scope the user set), propose it in the handoff with the layer, the change, and what it would remove. Ship a symptom patch only if it is safe, minimal, and labeled temporary; never present it as the fix
- The regression test reproduces the cause at the producer, not the guard at the consumer

## Clean Code Principles

- Touch only what the task requires; adjacent improvements go in their own PR or stay out
- No code comments. Convey intent through names and structure; if code seems to need a comment, rename or restructure it. Only compiler- or tooling-required comments (attributes, lint directives, license headers) are exceptions
- Full domain terms in names (`transaction`, not `tx`) except when preserving external protocol fields, database columns, or URLs verbatim
- Intent-specific names that state the domain action and result (`parse_destination_tag`, `build_transfer_message`, `map_balance_assets`). Generic verbs such as `process`, `handle`, `manage`, `perform`, `execute`, and `resolve` hide the contract; keep them only when a framework or protocol owns the signature
- Extend the existing component, domain type, mapper, or fixture before adding another. Reuse the flow's loading, error, navigation, and cancellation behavior; do not introduce a parallel path for the new entry point
- Model variants with a type, not a boolean flag or a bare string. An enum or sealed hierarchy the compiler checks exhaustively replaces paired booleans, optional-plus-flag pairs, and default branches that hide a missing state
- Keep types and functions single-purpose; expose only what current callers require
- Immutable bindings (`let`, `val`, non-`mut`); mutation only where ownership requires it, in the narrowest scope
- Resolve conflicting examples against the documented contract and caller behavior. Recency or a passing test alone does not make a pattern correct; explain the choice and flag unrelated drift

## Abstractions Must Earn Their Place

- Use the existing family contract (`ChainConfig`, provider trait, mapper, Core service) for shared behavior. Extract a new abstraction only for current consumers with the same domain rule, or an explicit boundary required by this task. Hypothetical reuse does not justify a crate, trait, wrapper, service, module, or wider signature.
- Keep a single-case rule small and named at its owner. Do not create empty companion files, forwarding-only services, or helper objects to satisfy a layout convention. Put intrinsic behavior on its type and pure rules without a natural receiver in the existing mapper/rules module.
- Preserve independent concurrent requests. Fetch a shared value once in the orchestrator and pass it down instead of adding a singleton cache or lock to deduplicate calls. Add mutable coordination only for a demonstrated race, lifetime, or consistency requirement; preserve required security and transaction ordering.

## No Over-Defensive Code

- Validate once at the trust boundary (external payloads, RPC responses, user input, FFI edges), then trust the types inside. Redundant null checks, re-validation of typed values, catch-all handlers, and `unwrap_or`-style fallbacks that hide a failure are review findings, not safety.
- Do not duplicate an invariant across layers. When Core or a typed shared contract owns a rule, the apps consume the result.
- Fail fast and loud where the app cannot safely continue; fail closed on security state. Apply the platform safety rules and [Security](security.md).

## Tests

- Tests protect an independent contract: a business rule, failure boundary, or required wire/signing format. For a new or changed domain rule, invert it temporarily, run the targeted test, confirm failure, and restore it. A test that mocks the defect's owner proves nothing about that defect
- For a high-impact bug with a deterministic seam, add the smallest test that materially reduces regression risk and write it before the fix; skip trivial, framework, formatting-only, and purely visual coverage unless asked or already cheap
- "Tests pass" is not a green light if any were skipped, marked expected-failure, or gated behind features you did not run. Report what you executed
- Unit tests never spin up ad hoc HTTP/TCP servers. Use the platform testkit fixtures, pure mappers and parsers, or injected clients; when network behavior matters, use the gated integration tests

Review criteria live in `skills/code-review.md`; this file is for writing code.
