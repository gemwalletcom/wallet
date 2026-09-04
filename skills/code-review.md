# Code Review

Use this guide as repository-specific criteria for coding agents' built-in review workflows. Keep the agent's native review mechanics and output format, but apply these checks when reviewing local changes, pull requests, or a proposed patch before implementation.

This guide is not a standalone command and does not replace platform guides, security rules, or the agent's built-in review behavior. By default, review only and report findings. Fix issues only when the user explicitly asks for fixes.

The source-of-truth details stay in the topic-specific skills. This file should point review attention to those rules instead of copying their full checklists.

## Review Setup

1. Read `AGENTS.md`, `skills/cross-platform-awareness.md`, and `skills/engineering-principles.md`.
2. Read the platform guide for every changed area: `ios/AGENTS.md`, `android/AGENTS.md`, or `core/AGENTS.md`.
3. Read `skills/security.md` before reviewing key management, wallet import/export, seed phrases, signing, transaction construction, auth, secure storage, external payload parsing, or cryptographic flows.
4. Fix the review range with `skills/task-workflow.md` § 1 and § 2: a PR's actual base rather than `main`, the PR addressed explicitly from a detached checkout, and for a large refactor the architectural start and every affected surface before the range is frozen. Do not claim full coverage while inventory, discovery, validation, or attack-path work is incomplete.
6. Inspect the diff, then read the full changed files, callers, exports, configuration, and nearby patterns before judging the change.
7. Check whether generated files, localization outputs, or mobile bindings were edited directly. Generated outputs must come from the source inputs.

Review is findings-only by default. Do not edit code, commit, push, open or update PRs/issues, post review comments, or resolve threads unless the user separately requests that action.

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

- Match the local platform style before introducing a new pattern. Prefer existing domain types, mappers, repositories, view models, storage layers, and helper APIs.
- Keep the patch small and reviewable. Remove unrelated formatting, drive-by refactors, speculative abstractions, and unused public API.
- Reduce duplication when the repeated logic has the same domain meaning, but avoid abstractions that hide simple platform-specific behavior.
- Prefer clear domain names over generic names. Use full terms such as `transaction`, `address`, `wallet`, and `amount` unless preserving external protocol names.
- Keep functions and types single-purpose. Split large changes when separate concerns make review harder.
- Prefer explicit data flow over hidden globals, implicit defaults, or broad mutable state.
- Avoid ad hoc parsing when a structured parser, generated model, mapper, or existing primitive already exists.
- Keep imports, localization keys, generated outputs, and test fixtures consistent with nearby files.

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

When asked to fix issues, implement only the findings the user selected, make the smallest scoped patch, rerun the affected checks from `skills/quality-checks.md`, and re-review the resulting diff before handoff.
