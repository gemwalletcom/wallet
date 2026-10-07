# Engineering Principles

Use for every code change. Guidance precedence is defined in [AGENTS.md](../AGENTS.md#using-the-guidance).

## Fix Causes, Not Symptoms

Fix the broken invariant at its owner, even when that requires changing a type or Core contract. Do not defer an in-scope fix to [Open work](../docs/TODO.md).

- Fix the producer and delete downstream guards it makes unnecessary; see § No Over-Defensive Code
- A null check, swallowed error, retry, wider timeout, or sleep requires a named state it handles
- Diagnose failing tests against intent. Fix a violating implementation; update stale expectations or intentionally changed behavior. Never weaken assertions just to pass
- Check other entry points, chains, providers, and timing for the same cause
- When scope, another repository, a provider, or shipped clients prevent the owner fix, describe it and the downstream work it would remove in the handoff. Symptom relief must be safe, minimal, and labeled temporary
- Verify producer and consumer behavior before asserting a defect; test its cause at the producer

## Clean Code Principles

- Touch only what the task requires; adjacent improvements go in their own PR or stay out
- No code comments; express intent through names and structure. Only compiler- or tooling-required comments and license headers are exceptions
- Use full domain terms except for external protocol fields, database columns, or URLs. Core error fields keep `msg`: UniFFI's generated Kotlin exceptions cannot declare `message` without shadowing `Throwable.message`
- Name the domain action and result (`parse_destination_tag`, `build_transfer_message`, `map_balance_assets`). Keep generic verbs (`apply`, `process`, `handle`, `manage`, `perform`, `execute`, `resolve`) only when a framework or protocol owns the signature
- Extend existing components, types, mappers, and fixtures. Reuse loading, error, navigation, and cancellation behavior for new entry points
- Model variants with exhaustive enums or sealed types, replacing paired flags, optional-plus-flag pairs, and strings that hide missing states
- Keep types and functions single-purpose; expose only what current callers require
- Immutable bindings (`let`, `val`, non-`mut`); mutation only where ownership requires it, in the narrowest scope
- Resolve conflicting examples from current contracts and callers; recency or passing tests alone is insufficient. Explain the choice and flag unrelated drift
- A press highlight matches the control's visible shape; see [Android modifier ordering](../android/skills/code-style.md#core-rules)

## Abstractions Must Earn Their Place

- Use existing family contracts (`ChainConfig`, provider traits, mappers, Core services). Extract abstractions only for current consumers sharing a domain rule or a task-required boundary; hypothetical reuse does not justify infrastructure or wider signatures.
- Keep single-case rules at their owner. No empty companion files, forwarding-only services, or helper objects for layout. Intrinsic behavior belongs on its type; pure rules without a receiver belong in existing mapper/rules modules.
- Preserve independent concurrent requests. Fetch shared values once in the orchestrator and pass them down. Mutable coordination needs a demonstrated race, lifetime, or consistency requirement; preserve security and transaction ordering.

## No Over-Defensive Code

- Validate external payloads, RPC responses, user input, and FFI values once at the trust boundary. Trust established invariants internally; redundant checks, catch-all handlers, and defaults that hide failures are review findings.
- Do not duplicate an invariant across layers. When Core or a typed shared contract owns a rule, the apps consume the result.
- Fail fast and loud where the app cannot safely continue; fail closed on security state. Apply the platform safety rules and [Security](security.md).

## Tests

- Protect independent business rules, failure boundaries, and wire/signing contracts. Temporarily invert a changed domain rule, confirm the targeted test fails, then restore it. Do not mock the defect's owner
- For high-impact bugs with deterministic seams, write the smallest useful regression test before fixing. Skip trivial, framework, formatting, and purely visual coverage unless requested or already cheap
- Use focused database checks for straightforward schema, operator flags, and unchanged persistence mechanics; tests must protect behavior beyond repeating SQL or mappings
- Unit tests use testkit fixtures, pure rules, or injected clients; never ad hoc HTTP/TCP servers. Network compatibility belongs in gated integration tests

Review criteria live in `skills/code-review.md`; this file is for writing code.
