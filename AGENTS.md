# AGENTS.md

Guidance for Coding Agents (Claude Code, Codex, etc.) collaborating in this monorepo. This file is the routing layer plus the non-negotiable rules. Everything else lives in a skill, a platform guide, or a design doc; load only what the task needs.

## Skills

`task-workflow.md`, `cross-platform-awareness.md`, and `engineering-principles.md` apply to every task. The rest are load-on-demand; the one-line description tells you when.

- [Task Workflow](skills/task-workflow.md) — Scope, investigation, cross-stack order (Core → bindings → iOS/Android), verification, cleanup rounds, handoff, and guide maintenance
- [Cross-Platform Awareness](skills/cross-platform-awareness.md) — Rules for changes that can affect both apps
- [Engineering Principles](skills/engineering-principles.md) — Clean-code, generic-solution, and test-intent rules shared across the repo
- [Project Overview](skills/project-overview.md) — Repo layout, layer architecture, and ownership boundaries
- [Development Commands](skills/development-commands.md) — Root build, generate, localization, and platform entrypoint commands
- [Quality Checks](skills/quality-checks.md) — Iteration and closing check matrices per change type
- [Code Review](skills/code-review.md) — Review checklist for correctness, conventions, parity, and adversarial security hardening
- [Security](skills/security.md) — Wallet-critical security rules for key material, signing, auth, and transaction handling
- [Localization](skills/localization.md) — Fluent sources, generation, and generated output locations for both apps
- [Maestro UI Testing](skills/testing-maestro.md) — When to use Maestro flows vs unit or native UI tests, and cross-platform authoring rules
- [Release Process](skills/release-process.md) — Branching, versioning, commits, publication boundaries, store builds, and removing or disabling support
- [Guidance Refresh](.agents/skills/guidance-refresh/SKILL.md) — How lessons become shared guidance, budgets, skill format, and the local sweep each teammate runs

## Platform Guides

Read the relevant platform guide(s) before editing code in that area:

- [iOS](ios/AGENTS.md) — SwiftUI, MVVM, SPM modules, testing conventions
- [Android](android/AGENTS.md) — Kotlin, Compose, Hilt, Gradle workflow
- [Core](core/AGENTS.md) — Rust crates, UniFFI/TypeShare, clippy, defensive programming

If a task spans multiple platforms, read every affected guide. Do not treat every `core/` edit as a cross-platform build change. Regenerate and verify the apps only when Core changes UniFFI/TypeShare interfaces, generated models, platform build inputs, or app-side integration. For internal Core implementation changes that preserve those contracts, run the relevant Core verification without building iOS or Android.

## Design Docs

Cross-platform subsystem references live in [docs/](docs). Read the relevant one before changing that area:

- [Architecture](docs/ARCHITECTURE.md) — the reference every new feature follows: Core-owned services, pure rules, stores, and app mapping
- [Decision Records](docs/DECISIONS.md) — repo-wide architectural choices and their rationale
- [Services](docs/SERVICES.md) — how a Gemstone service is built and the remaining migration work
- [Deep links](docs/DEEPLINKS.md) — deep link URL contract, support-chat links, and the web association requirements
- [Device and subscriptions](docs/DEVICE_SUBSCRIPTIONS.md) — device registration, subscription sync, and the iOS/Android contract
- [Payments](docs/PAYMENTS.md) — payment decoding flow, implementation map, and QR test cases
- [Swapper](docs/SWAPPER.md) — quote flow, route preloading, and the shared route cache

Core-owned subsystems (keystore, device and wallet authentication, WebSockets, provider coverage) are listed in [core/AGENTS.md](core/AGENTS.md).

## Security

This is a crypto wallet. Treat security-sensitive changes as high risk by default.

- Read [skills/security.md](skills/security.md) before changing key management, wallet import/export, seed phrases, signing, transaction construction, auth, secure storage, or cryptographic flows
- Never log, print, persist, snapshot, or expose secret material unless the feature explicitly requires secure handling and existing patterns already support it
- Preserve transaction integrity: amounts, addresses, chain IDs, signatures, simulation data, and confirmation flows must stay explicit and verifiable
- Prefer existing secure-storage and auth layers over inventing new persistence or authentication paths

## Task Completion

Fix causes, not symptoms. Ask why the failure was possible before changing anything, and fix the layer that made it possible; if that fix is out of scope, name the cause and flag it (see [Engineering Principles](skills/engineering-principles.md)).

Non-negotiable, whatever the task size:

1. Build the affected platform(s) and run the relevant tests. Documentation-only changes use the lightweight checks in [skills/quality-checks.md](skills/quality-checks.md) instead.
2. Run two cleanup rounds on your own diff before handoff. Round one, once the code works and before the verification batch: dedupe, simplify, remove dead code and stale fixtures, and match the surrounding codebase conventions. Round two, on the final diff after verification: read it as a reviewer would and remove anything round one left. Rerun the affected targeted checks if a round changed source. Details in [skills/task-workflow.md](skills/task-workflow.md).
3. Review security impact for changes touching secrets, signing, auth, transactions, or wallet recovery.
4. If `core/` changed mobile interfaces, generated models, platform build inputs, or app-side integration, regenerate and verify the affected app(s); otherwise keep verification scoped to Core.

Do not close a task on reasoning, `git diff`, or file inspection alone. Run real verification for the changed area and report the exact commands, their results, and anything skipped or blocked. For wallet-critical flows (signing, secure storage, migrations, key import/export, transaction construction), "completed" is wrong if anything was skipped silently: surface skipped records, swallowed errors, and untested branches explicitly. A silent success on these paths is the most expensive failure mode in this repo.
