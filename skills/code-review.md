# Code Review

Apply these repository-specific checks within the agent's native review workflow and output format. Review only unless fixes are requested; topic and platform guides remain the source of truth.

## Setup and Scope

1. Follow [Task Workflow](task-workflow.md) and `AGENTS.md` precedence. Read each affected platform guide and [Security](security.md) for wallet-critical flows or external payload parsing.
2. Fix the review range: the PR's actual base and head commits, explicitly identifying the PR from a detached checkout. For large refactors, establish the architectural starting point and affected surfaces. Refresh affected evidence if the head moves, or mark it stale; do not claim coverage beyond completed review work.
3. Read the diff, changed files, callers, exports, configuration, and relevant documented examples.
4. Keep the original feature contract across rounds. Recheck the complete patch for missed defects and regressions from fixes. Restore invariants at all affected sites without expanding unrelated additions into new requirements. Stop when behavior is correct, focused checks pass, and no material defect or required check remains unresolved.

## Correctness and Consistency

- Derive expected behavior from the task, current contracts, and callers. For non-trivial changes, challenge assumptions with reachable counterexamples: empty or malformed inputs, unsupported assets, repeated operations, partial failure, a wallet switch during an async request, a superseded quote, or cancellation before persistence.
- Trace behavior through decoding, transaction construction, signing, app consumption, and runtime loading as applicable. Registrations, generated types, and successful builds alone do not establish runtime support.
- Apply [Fix Causes, Not Symptoms](engineering-principles.md#fix-causes-not-symptoms); verify producer fixes and any explicitly temporary symptom relief.
- Apply [Cross-Platform Awareness](cross-platform-awareness.md) for parity, localization, generation, and bindings. Generated outputs must trace to source inputs. Check stale callers, missing migrations or localization keys, and feature-flag behavior.
- Check numeric input and output under comma-decimal locales and non-Latin digits such as `٥٠`, following the [number-parsing contract](../docs/ARCHITECTURE.md#number-parsing-human-input-vs-machine-strings).
- Tests may share the implementation's assumptions. Reproduce useful checks and probe missed scenarios; confirm tests fail when the protected rule is inverted. Preserve error context and fail closed when safe continuation is impossible.
- Verify compatibility against repository exports, bindings, configuration, and shipped tags, separately from upstream compatibility. Support changing protocol/provider/fee/release claims with authoritative sources and a revision, block, tag, or observation time. Limited live samples are indicative; corroborate transport/TLS/rate-limit failures before attributing them to product or protocol behavior.

## Design and Style

Apply [Engineering Principles](engineering-principles.md) and platform conventions against the existing owner and documented examples. Each new abstraction or public symbol needs a current consumer or required contract. Flag unnecessary wrappers, duplicate paths, and hypothetical infrastructure; keep unrelated refactors out.

## Security

Apply [Security](security.md) to the changed paths:

- Challenge external values from RPCs, dapps, deep links, files, URLs, and clipboard content at trust boundaries.
- Check isolation between chains, wallets, accounts, dapps, sessions, and cached responses.
- Ensure fallback, retry, cache, and recovery paths reject stale, attacker-controlled, or unverifiable security state.
- Trace injection risks where strings become commands, SQL, URLs, HTML/Markdown, JavaScript bridge messages, or protocol payloads.
- For signing, secure storage, transaction construction, or other wallet-critical boundaries, include a compact matrix of normal, boundary, and hostile scenarios, expected behavior, and covering tests. Record uncovered cases as verification gaps.

## Reporting

Report findings by severity with file and line references. Each correctness/security finding needs a reachable failing scenario, expected versus actual behavior, impact, and the smallest reasonable fix. Trace callers and guards, compare with the base revision, and distinguish reproduction from code inspection.

For explicit repository-contract violations, cite the rule and concrete mismatch; rate by demonstrated impact. Style preferences, speculation, duplication, or missing tests alone are not correctness findings. Optional cleanup belongs in the author's cleanup rounds.

Separate verification gaps (uncovered scenarios, missing regression coverage, skipped checks) from pre-existing bugs verified against the base. Gaps are not proof of defects; required checks still gate completion. Report exact commands run, skipped, or blocked. If no findings remain, say so and disclose residual risks or gaps.

Use native severity labels when available; otherwise:

- Critical: security compromise, wallet-critical correctness, data loss, signing or transaction integrity failures
- High: likely user-facing regressions, parity breaks, broken builds, missing migrations, invalid bindings
- Medium: edge-case correctness bugs or incorrect error handling
- Low: minor correctness bugs with limited impact

For authorized fixes, make the smallest complete patch within scope, run affected [Quality Checks](quality-checks.md), and re-review the diff.
