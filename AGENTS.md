# AGENTS.md

Guidance for Coding Agents (Claude Code or Codex, etc.) collaborating in this monorepo.

## Skills

Read this file first, then load only the skills relevant to your current task — you do not need to read all files upfront. `cross-platform-awareness.md` and `engineering-principles.md` apply to every task; the rest are load-on-demand.

- [Project Overview](skills/project-overview.md) — Repo layout, ownership boundaries, and shared concepts
- [Cross-Platform Awareness](skills/cross-platform-awareness.md) — Rules for changes that can affect both apps
- [Development Commands](skills/development-commands.md) — Root build, generate, localization, and platform entrypoint commands
- [Architecture](skills/architecture.md) — High-level iOS, Android, and shared-layer architecture
- [Engineering Principles](skills/engineering-principles.md) — Clean-code rules and code review standards shared across the repo
- [Security](skills/security.md) — Wallet-critical security rules for key material, signing, auth, and transaction handling
- [Quality Checks](skills/quality-checks.md) — Lint, format, and static-analysis commands for each platform
- [Release Process](skills/release-process.md) — Branching, versioning, and commit expectations
- [Localization](skills/localization.md) — Shared localization flow and generated output locations
- [New Feature Workflow](skills/new-feature-workflow.md) — End-to-end sequence for cross-stack features (Core → bindings → iOS/Android)
- [Decision Records](skills/decisions.md) — Non-obvious architectural choices and their rationale

## Platform Guides

Read the relevant platform guide(s) before editing code in that area:

- [iOS](ios/AGENTS.md) — SwiftUI, MVVM, SPM modules, testing conventions
- [Android](android/AGENTS.md) — Kotlin, Compose, Hilt, Gradle workflow
- [Core](core/AGENTS.md) — Rust crates, UniFFI/TypeShare, clippy, defensive programming

If a task spans multiple platforms, read every affected guide. If you touch `core/`, treat it as a cross-platform change and verify both apps.

## Security

This is a crypto wallet. Treat security-sensitive changes as high risk by default.

- Read [skills/security.md](skills/security.md) before changing key management, wallet import/export, seed phrases, signing, transaction construction, auth, secure storage, or cryptographic flows
- Never log, print, persist, snapshot, or expose secret material unless the feature explicitly requires secure handling and existing patterns already support it
- Preserve transaction integrity: amounts, addresses, chain IDs, signatures, simulation data, and confirmation flows must stay explicit and verifiable
- Prefer existing secure-storage and auth layers over inventing new persistence or authentication paths
- If a change affects `core/` cryptography or signing behavior, verify both apps after regeneration

## Testing

- When fixing a high-impact bug, add or update the smallest meaningful test only if it materially reduces regression risk; keep it compact, avoid trivial/framework/formatting-only coverage, and skip purely visual UI polish unless coverage is explicitly requested or already cheap to extend

## Task Completion

Before finishing a task:
1. Build the affected platform(s)
2. Run the relevant test suites
3. Run the relevant linters and formatters when code changed
4. Review security impact for changes affecting secrets, signing, auth, transactions, or wallet recovery
5. If `core/` changed, regenerate bindings/models and verify both apps
6. Remove dead code, keep imports clean, and follow platform patterns

Do not close a task based only on reasoning, `git diff`, or file inspection. Run real verification commands for the changed area. If verification is blocked by unrelated repo state, report the exact command you ran and the blocking failure explicitly.
