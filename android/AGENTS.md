# AGENTS.md

Guidance for Coding Agents working in the Android app.

## Skills

Read this file first, then load the relevant skills before editing Android code. `development-commands.md`, `project-overview.md`, and `code-style.md` are the default set for most tasks. Load `setup.md` for environment/bootstrap work and `release-and-verification.md` for Docker, reproducible builds, or release packaging.

- [Project Overview](skills/project-overview.md) — Repo structure, technologies, and build flavor layout
- [Setup](skills/setup.md) — Prerequisites, bootstrap, and local credential requirements
- [Development Commands](skills/development-commands.md) — Gradle and `just` workflows for build, test, generate, and lint
- [Code Style](skills/code-style.md) — Kotlin, Compose, DI, and validation expectations
- [Testing](skills/testing.md) — Test organization, mocks, unit and instrumented test patterns
- [Release and Verification](skills/release-and-verification.md) — Docker, reproducible builds, artifact verification, and CI/release context
- [Troubleshooting](skills/troubleshooting.md) — Common pitfalls, recovery commands, and important file locations

## Related Guides

- [Monorepo](../AGENTS.md)
- [Core](../core/AGENTS.md)

Read `core/AGENTS.md` when the task touches `core/`, generated models, JNI bindings, or shared blockchain behavior.

## Task Completion

Before finishing an Android task:
1. Build the affected variant or module
2. Run the relevant Gradle tests
3. Run the relevant lint and formatting tasks when Kotlin or resources changed
4. Clean imports and avoid unnecessary comments
5. If `core/` changed, regenerate shared artifacts and verify Android still builds
6. In tests, prefer shared `:gemcore` fixtures with sensible defaults over inline full-field mock construction

Do not finish an Android task without running at least one real Gradle verification command for the touched codepath. `git diff --check`, code inspection, or reasoning are not enough. If Gradle is blocked by unrelated repo failures, report the exact command and the blocking error instead of claiming the change was verified.
