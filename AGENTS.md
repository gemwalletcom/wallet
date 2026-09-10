# AGENTS.md

Guidance for Coding Agents (Claude Code, Codex, etc.) collaborating in this monorepo. This file is the routing layer plus the non-negotiable rules. Everything else lives in a skill, a platform guide, or a design doc; load only what the task needs.

## Using the Guidance

The user's task defines scope and authorization. Shared rules apply to both agents; platform guides specialize them, not silently contradict them. Current subsystem contracts govern design; nearby code provides examples, not permission to copy a known violation. Agent memory and external skills are advisory and must be checked against these sources. When guidance disagrees, resolve it from the current contract and source, fix the stale rule, and ask only if a product or security decision remains unresolved.

## Skills

`task-workflow.md`, `cross-platform-awareness.md`, and `engineering-principles.md` apply to every task. The rest are load-on-demand; the one-line description tells you when.

- [Task Workflow](skills/task-workflow.md) — Scope, investigation, cross-stack order (Core → bindings → iOS/Android), verification, cleanup rounds, handoff, and guide maintenance
- [Cross-Platform Awareness](skills/cross-platform-awareness.md) — Rules for changes that can affect both apps
- [Engineering Principles](skills/engineering-principles.md) — Ownership, reuse, minimal abstractions, and test intent
- [Project Overview](skills/project-overview.md) — Repo layout, layer architecture, and ownership boundaries
- [Development Commands](skills/development-commands.md) — Root build, generate, localization, and platform entrypoint commands
- [Quality Checks](skills/quality-checks.md) — Iteration and closing check matrices per change type
- [Code Review](skills/code-review.md) — Review checklist for correctness, conventions, parity, and adversarial security hardening
- [Security](skills/security.md) — Wallet-critical security rules for key material, signing, auth, and transaction handling
- [Localization](skills/localization.md) — Fluent sources, generation, and generated output locations for both apps
- [Translation Review](.agents/skills/translation-review/SKILL.md) — Context, approved terminology, compact labels, and scheduled localization audits
- [Maestro UI Testing](skills/testing-maestro.md) — When to use Maestro flows vs unit or native UI tests, and cross-platform authoring rules
- [Release Process](skills/release-process.md) — Branching, versioning, commits, publication boundaries, store builds, and removing or disabling support
- [Guidance Refresh](.agents/skills/guidance-refresh/SKILL.md) — How lessons become shared guidance, budgets, skill format, and the local sweep each teammate runs

## Platform Guides

Read the relevant platform guide(s) before editing code in that area:

- [iOS](ios/AGENTS.md) — SwiftUI, MVVM, SPM modules, testing conventions
- [Android](android/AGENTS.md) — Kotlin, Compose, Hilt, Gradle workflow
- [Core](core/AGENTS.md) — Rust crates, UniFFI/TypeShare, clippy, defensive programming

If a task spans platforms, read every affected guide. Generation and parity requirements live in [Cross-Platform Awareness](skills/cross-platform-awareness.md); verification commands live in [Quality Checks](skills/quality-checks.md).

## Design Docs

Cross-platform subsystem references live in [docs/](docs). Read the relevant one before changing that area:

- [Architecture](docs/ARCHITECTURE.md) — Current ownership contracts and a task-based index of implementation examples
- [Services](docs/SERVICES.md) — how a Gemstone service is built and the remaining migration work
- [Deep links](docs/DEEPLINKS.md) — deep link URL contract, support-chat links, and the web association requirements
- [Device and subscriptions](docs/DEVICE_SUBSCRIPTIONS.md) — device registration, subscription sync, and the iOS/Android contract
- [Payments](docs/PAYMENTS.md) — payment decoding flow, implementation map, and QR test cases
- [Swapper](docs/SWAPPER.md) — quote flow, route preloading, and the shared route cache
- [Egress](docs/EGRESS.md) — provider routing, scoped headers, credentials, and configuration rollout

Core-owned subsystems (keystore, device and wallet authentication, WebSockets, provider coverage) are listed in [core/AGENTS.md](core/AGENTS.md).

## Security

This is a crypto wallet. Treat security-sensitive changes as high risk by default.

- Read [skills/security.md](skills/security.md) before changing key management, wallet import/export, seed phrases, signing, transaction construction, auth, secure storage, or cryptographic flows
- Never log, print, persist, snapshot, or expose secret material unless the feature explicitly requires secure handling and existing patterns already support it
- Preserve transaction integrity: amounts, addresses, chain IDs, signatures, simulation data, and confirmation flows must stay explicit and verifiable
- Prefer existing secure-storage and auth layers over inventing new persistence or authentication paths

## Task Completion

1. Fix the cause at its owner, within the task's scope; see [Engineering Principles](skills/engineering-principles.md).
2. Run both cleanup rounds in [Task Workflow](skills/task-workflow.md), before and after verification. Rerun affected checks if cleanup changes source.
3. Review security impact for changes touching secrets, signing, auth, transactions, or wallet recovery.
4. Run the applicable [Quality Checks](skills/quality-checks.md), including generation and app verification when shared contracts change. Documentation-only work uses its lightweight checks.

For code changes, reasoning and file inspection do not replace execution. Report exact verification commands, results, and anything skipped or blocked. For wallet-critical flows, explicitly surface skipped records, swallowed errors, and untested branches.
