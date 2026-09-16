# Code Review

Use this guide as repository-specific criteria for coding agents' built-in review workflows. Keep the agent's native review mechanics and output format, but apply these checks when reviewing local changes, pull requests, or a proposed patch before implementation.

This guide is not a standalone command and does not replace platform guides, security rules, or the agent's built-in review behavior. By default, review only and report findings. Fix issues only when the user explicitly asks for fixes.

The source-of-truth details stay in the topic-specific skills. This file should point review attention to those rules instead of copying their full checklists.

## Review Setup

1. Follow the task setup and guidance precedence in `AGENTS.md` and `skills/task-workflow.md`.
2. Read the platform guide for every changed area: `ios/AGENTS.md`, `android/AGENTS.md`, or `core/AGENTS.md`.
3. Read `skills/security.md` before reviewing key management, wallet import/export, seed phrases, signing, transaction construction, auth, secure storage, external payload parsing, or cryptographic flows.
4. Fix the review range with `skills/task-workflow.md` § 1 and § 2: a PR's actual base rather than `main`, the PR addressed explicitly from a detached checkout, and for a large refactor the architectural start and every affected surface before the range is frozen. Do not claim full coverage while inventory, discovery, validation, or attack-path work is incomplete.
5. Inspect the diff, then read the changed files, callers, exports, configuration, and relevant examples before judging the change.
6. Check whether generated files, localization outputs, or mobile bindings were edited directly. Generated outputs must come from the source inputs.

## 1. Correct Implementation and Cross-Platform Consistency

- Verify the change implements the requested behavior, not just code that compiles.
- Verify the change removes the cause, not the symptom. A retry, fallback, wider timeout, or special case at the point where a bad value is observed is a finding unless the producer is fixed too, or the author has named it as temporary symptom relief with the root cause as follow-up (`skills/engineering-principles.md` § Fix Causes, Not Symptoms).
- Trace the runtime path that establishes the behavior. A registered chain/asset/provider, generated type, or successful build is insufficient when response decoding, transaction construction, signer support, app consumption, or runtime loading is unverified.
- Check edge cases, failure paths, empty states, invalid inputs, retries, cancellation, and unsupported chains or assets.
- Apply `skills/cross-platform-awareness.md` for shared app behavior, generated files, localization, and `core/` regeneration requirements.
- Confirm tests assert the business rule. A test that would still pass after flipping the rule is not meaningful coverage.
- Look for stale call sites, unused additions, unreachable branches, missing migrations, missing localization keys, and behavior hidden behind feature flags.
- Verify error handling preserves useful context while still failing closed where the app cannot safely continue.
- For compatibility claims, inspect the repository's exports, bindings, configuration, and shipped release tags as applicable. State repository-specific compatibility separately from general upstream compatibility.
- Tie drift-prone protocol, provider, contract, fee, minimum, or release claims to a current authoritative source and a named revision, block, tag, or observation time. Label limited live samples as indicative rather than universal.
- Separate network-level behavior from provider-specific behavior, and corroborate transport/TLS/rate-limit failures before treating them as product or protocol defects.

## 2. Coding Style, Codebase Convention, and Reviewability

- Apply [Engineering Principles](engineering-principles.md#clean-code-principles) and the affected platform style. Compare the implementation with the existing owner and the relevant documented example.
- For each new abstraction or public symbol, identify its current consumer or required contract. Flag forwarding-only wrappers, duplicate loading paths, ad hoc parsers, and new infrastructure whose need is hypothetical.
- Keep unrelated formatting and refactors out. Shared syntax alone does not justify merging different domain rules.

## 3. Adversary Review and Security Hardening

Review the change as if a hostile user, compromised website, malicious deep link, broken RPC, or tampered backend response is trying to exploit it.

- Apply `skills/security.md` as the source of truth for wallet-critical rules and optional external security skills.
- Identify trust boundaries and challenge every value crossing them, especially external payloads, RPC responses, browser or dapp handoff, files, URLs, and clipboard content.
- Check whether one chain, wallet, account, dapp, session, or cached response can influence another path it should not control.
- Confirm fallback, retry, cache, and recovery paths cannot silently accept stale, attacker-controlled, or unverifiable data.
- Look for injection risks where strings become commands, SQL, URLs, rendered HTML or Markdown, JavaScript bridge messages, or protocol payloads.

## Reporting

Report findings first, ordered by severity, with file and line references. Keep each finding concrete: describe the bug or risk, the user or security impact, and the smallest reasonable fix.

If no issues are found, say that clearly and list any residual risk or checks not run. Include the exact verification commands that were run, skipped, or blocked.

When the agent's built-in review workflow has its own severity labels, use those labels. Otherwise use:

- Critical: security issues, wallet-critical correctness bugs, data loss, signing or transaction integrity failures
- High: likely user-facing regressions, cross-platform parity breaks, broken builds, missing migrations, or invalid generated bindings
- Medium: edge-case correctness bugs, brittle error handling, test gaps for changed behavior, or maintainability issues that slow review
- Low: small style or convention issues that are safe to batch with other edits

When asked to fix issues, apply the authorized scope (selected findings or all findings), make the smallest complete patch, rerun affected [Quality Checks](quality-checks.md), and re-review the diff.
