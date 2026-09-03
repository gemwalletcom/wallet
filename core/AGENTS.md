# AGENTS.md

Guidance for AI assistants (Claude Code, Gemini, Codex, etc.) collaborating on this repository.

## Skills

Read this file first, then load the relevant skills for your current task. `project-structure.md`, `development-commands.md`, `code-style.md`, `tests.md`, and `defensive-programming.md` are the default set for most Core work. Load `setup.md` for environment or bootstrap work, `error-handling.md` when touching error surfaces or JSON access, `architecture.md` when changing provider/repository/UniFFI patterns, `common-issues.md` when debugging tricky failures, `swapper-checklist.md` only for swapper integrations, `new-chain-checklist.md` when adding or restructuring a chain, and `localization.md` when adding Core user-facing strings.

- [Project Structure](skills/project-structure.md) — Repo layout, crates, and tech stack
- [Setup](skills/setup.md) — Toolchain, backend prerequisites, and the kache build cache
- [Development Commands](skills/development-commands.md) — Build, test, lint, format, mobile
- [Code Style](skills/code-style.md) — Formatting, naming, imports, code organization
- [Error Handling](skills/error-handling.md) — Error types, propagation, JSON access
- [Architecture](skills/architecture.md) — Provider/mapper, repository, RPC, UniFFI patterns
- [Tests](skills/tests.md) — Test conventions, mocks, integration tests
- [Defensive Programming](skills/defensive-programming.md) — Safety rules and exhaustive patterns
- [Common Issues](skills/common-issues.md) — Known anti-patterns and their fixes
- [Swapper Checklist](skills/swapper-checklist.md) — Integration checklist for swapper providers
- [New Chain Checklist](skills/new-chain-checklist.md) — Boundary rule, factory dispatch, and steps for adding a chain
- [Localization](skills/localization.md) — Fluent strings in the `localizer` crate and typed accessors

## Design Docs

Subsystem references live in [docs/](../docs). Read the relevant one before changing that area:

- [Gem Keystore v4](../docs/KEYSTORE_V4.md) — keystore file format, v3 migration, account public keys, and the keystore-internal signing / device-auth contract (key never crosses the FFI boundary)
- [Device Authentication](../docs/DEVICE_AUTHENTICATION.md) — Ed25519 request signing and the `Gem` Authorization header
- [Wallet Authentication](../docs/WALLET_AUTHENTICATION.md)
- [Device WebSockets](../docs/DEVICE_WEBSOCKETS.md)
- [Rewards and Referrals](../docs/REWARDS_AND_REFERRALS.md)
- [Core Features and Providers](../docs/FEATURES.md) — chain capabilities and indexing, WalletConnect, swap, fiat, and NFT provider coverage

## Before Coding

- Every Rust `use` declaration at the top of the file; never import inside a function or write a fully qualified path inline (see `skills/code-style.md` § Imports)
- State assumptions explicitly. UniFFI bounds, lifetimes, provider trait contracts, and JSON shape assumptions are invisible — call them out so a reviewer can spot the wrong one
- Read before you write. Open the file's existing exports, the immediate caller, the related provider/mapper/repository, and any obvious testkit fixture before adding code. "Looks orthogonal to me" is the most expensive sentence in this crate
- If two patterns in the codebase contradict (e.g., two providers handling decimals or error mapping differently), do not average them. Pick one — typically the more recent or better tested — explain why, and flag the other for cleanup
- Use single-word names for Core settings keys; `_` is reserved for separating the settings hierarchy in environment variables
- Keep `docs/FEATURES.md` (repo root) current in the same change when chain capabilities, simulations, WalletConnect coverage, transaction-indexing routes, active swap, fiat, or NFT providers, provider modes, amount/slippage behavior, deployments, or supported assets change. Recheck dynamic provider coverage weekly; update the reviewed date only after rechecking the linked provider sources

## Task Completion

During active implementation, rebase conflict resolution, or compile-fix loops, prefer targeted build/test commands and defer broad clippy/format runs until the change is ready to commit. Do not skip the required clippy/format checks silently before final handoff; run them then, or report the exact reason they are still pending.

Before finishing a task:
1. **Run the two cleanup rounds** from `skills/task-workflow.md` — reduce duplication, extract helpers only when they earn their keep, consolidate modules, remove dead code, and match the crate's conventions; then re-read the final diff as a reviewer
2. **Keep changes minimal** — code must be concise and focused; reviewers cannot realistically review thousands of lines per PR, so only include what is necessary for the task
3. **Run tests**: `just test` or `just test <CRATE>`
4. **Run clippy**: `cargo clippy -p <crate> --all-features -- -D warnings` (most crates gate modules behind features; without them clippy compiles nothing from those modules, see `skills/development-commands.md`)
5. **Format**: `just format`

Regenerate bindings and build iOS or Android only when the change affects UniFFI/TypeShare interfaces, generated models, platform build inputs, or app-side integration. Do not run mobile generation or builds for internal Core implementation changes that preserve those contracts.

Test rules and testkit conventions live in [Tests](skills/tests.md); read it before writing or changing any test.
