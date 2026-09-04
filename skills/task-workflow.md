# Task Workflow

Use this workflow for every task. It keeps the agentic loop scoped, evidence-driven, and easy for another teammate or agent to continue.

## 1. Establish the Task Contract

- Classify the request before acting: investigate, review, implement, or publish. Investigation and review are read-only unless the user asks for fixes. Committing, rebasing, pushing, opening a PR or issue, posting or resolving review comments, and submitting external forms are separate publication actions.
- Confirm the repository root, active checkout or worktree, branch or detached state, and `git status --short` before relying on files or running verification. Preserve unrelated user changes; avoid `git stash` on the user's tree, and if unavoidable, label it (`git stash push -m <label>`) and confirm it is on top before popping.
- Fetch remote state only when the task depends on it, and record the exact base, head, tag, or commit behind any claim that may drift.
- Load the platform guide and topic skill for every area that may change. State assumptions that affect scope, security, compatibility, or verification.

## 2. Inventory Before Designing

- Trace the real path before adding a parameter, asset, chain, provider, generated model, or workaround: source or configuration → request/response or storage → mapper or domain logic → signer or runtime → exported interface → app consumer. A declaration, dependency, generated type, or `supported_assets` entry alone does not prove working support.
- For a large refactor or audit, find the architectural starting point from history and dependency changes, inventory every affected user and security surface, then freeze an explicit immutable review range instead of an arbitrary time window.
- When comparing or porting PRs, compare each change against its own base (`git range-diff` when bases differ) and inventory review threads separately from code differences.
- Distinguish current behavior from a proposal, this repository's source compatibility from general upstream compatibility, and a configured or generated artifact from a runtime-verified integration.

## 3. Run a Tight Implementation Loop

- Make the smallest complete change at the layer that owns the rule; never duplicate a Core or provider invariant in both apps when a typed shared contract can own it.
- Use targeted builds and tests while iterating, and checkpoint at each Core → bindings → app boundary with what changed, what passed, and what remains.
- Delegate read-heavy searches and verbose verification runs to subagents and keep the session under its compaction cap; the rules are in `skills/context-efficiency.md`.
- For a feature that spans Core and the apps, work outward: Core first (crate logic, then TypeShare or UniFFI exports in `gemstone/`; `just test <CRATE>`, clippy, `just format`; a Core-owned feature follows `docs/ARCHITECTURE.md` § 11; never stub app code before the Rust types exist), then `just generate` only when the change touches mobile interfaces, generated models, platform build inputs, or app-side integration, then iOS and Android with their platform guides loaded, extending generated types in separate files and iterating with the targeted commands in `skills/quality-checks.md`. A user-facing feature ships on both platforms or the handoff names the parity gap.
- **Cleanup round one**, once the code works and before the verification batch: read the whole diff; dedupe, simplify, remove dead code, unused imports, stale fixtures, and unnecessary abstractions; replace ad hoc helpers with the existing domain type, mapper, or testkit fixture; match the surrounding naming, module placement, and error style. Do not mix unrelated cleanup into the patch.

## 4. Verify the Actual Result

- Run every command from the checkout and directory that contain the change, with the feature flags that compile the changed path. A filtered command that selects zero tests is not a pass; if parallel invocations contend or return an unclear status, rerun the closing checks individually. Tool-specific caveats live in the platform development-commands skills.
- Verify deterministic behavior with ordinary tests and live network compatibility only with the gated integration features, and report the two separately.
- Correctness, performance, compatibility, and exact output format are separate claims; evidence for one does not establish another.
- After switching branches with different generated contracts, suspect stale ignored build artifacts before diagnosing app compilation; regenerate only the affected bindings or models and rerun the focused check.

## 5. Hand Off a Reproducible State

- **Cleanup round two**, on the final diff after verification: read it hunk by hunk as a reviewer. Every hunk is necessary for the task, matches the file it lands in, and adds no public API, comments, or debugging leftovers. If this round changes source, rerun the affected targeted checks.
- Inspect the final diff and `git status`. Report the exact commands run, their results, anything skipped or blocked, and remaining risk, naming the completion level honestly: inventory, static inspection, generation, compilation, and a live end-to-end check are different things.
- Leave the checkout uncommitted, unpushed, and external systems unchanged unless asked. After a rebase or amend, push only with explicit authorization, with `--force-with-lease` against a verified remote target.

## 6. Maintain Agent Guidance

- At handoff, ask whether the task produced a lesson: a user correction, a dead end that cost more than a few tool calls, a guide instruction that proved wrong or stale, or a workflow that took trial and error. Name it in the handoff with the file it belongs in, save it to your agent's memory so your next `guidance-refresh` sweep finds it, and apply it now when the task already touches guidance or the user asks. A guide instruction found wrong is fixed in the same change, never worked around.
- Promote a lesson when it is repeated, costly to rediscover, security-sensitive, or a non-obvious architectural decision. Never record temporary paths, transient status, version snapshots, credentials, or session-specific tool failures.
- One canonical statement per rule, stated as trigger, action, verification boundary, and exception. `AGENTS.md` stays routing plus non-negotiables; procedures go in `skills/`, platform rules beside the platform, rationale in `docs/DECISIONS.md`, subsystem contracts in `docs/`.
- Guide maintenance is a documentation-only change: check contradictions, stale commands, links, and paths with the checks in `skills/quality-checks.md`. For the periodic sweep and the policy behind it, invoke the [`guidance-refresh` skill](../.agents/skills/guidance-refresh/SKILL.md).
