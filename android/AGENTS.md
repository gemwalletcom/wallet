# AGENTS.md

Guidance for Coding Agents working in the Android app.

## Skills

Read this file first, then load the relevant skills before editing Android code. `development-commands.md`, `project-overview.md`, and `code-style.md` are the default set for most tasks. Load `setup.md` for environment/bootstrap work and `release-and-verification.md` for release packaging.

- [Project Overview](skills/project-overview.md) — Repo structure, technologies, and build flavor layout
- [Setup](skills/setup.md) — Prerequisites, bootstrap, and local credential requirements
- [Development Commands](skills/development-commands.md) — Gradle and `just` workflows for build, test, generate, and lint
- [Code Style](skills/code-style.md) — Kotlin, Compose, DI, and validation expectations
- [Testing](skills/testing.md) — Test organization, mocks, unit and instrumented test patterns
- [Release](skills/release-and-verification.md) — Release builds and CI/release context
- [Troubleshooting](skills/troubleshooting.md) — Common pitfalls, recovery commands, and important file locations

## Related Guides

- [Monorepo](../AGENTS.md)
- [Core](../core/AGENTS.md)

Read `core/AGENTS.md` when the task touches `core/`, generated models, JNI bindings, or shared blockchain behavior.

## Task Completion

Follow [Task Workflow](../skills/task-workflow.md#4-verify-the-actual-result) and the Android rows in [Quality Checks](../skills/quality-checks.md). For UI changes, smoke the changed flow on an emulator or device when reachable. Test fixture conventions live in [Testing](skills/testing.md#shared-testkit).
