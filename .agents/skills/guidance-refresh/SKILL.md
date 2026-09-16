---
name: guidance-refresh
description: Audit shared agent guidance against local Claude and Codex history, resolve contradictions, and promote reusable lessons while trimming stale rules. Use for guidance audits, cleanup, or the monthly review. Documentation-only; leaves edits uncommitted.
---

# Guidance Refresh

Use when auditing or updating this repository's agent guidance. Investigation is read-only unless edits are requested. Do not change product code, generated files, configuration, dependencies, or external systems.

## Canonical Guidance

Follow [Task Workflow](../../../skills/task-workflow.md#6-maintain-agent-guidance) for capturing lessons during ordinary tasks. A sweep promotes confirmed lessons into shared guidance so both agents benefit. Agent memory supplies evidence, not policy; compare it with current source, security requirements, and user instructions before promoting it. Do not copy a stale workaround merely because it was previously accepted.

One canonical statement per rule. Root `AGENTS.md` routes tasks and defines non-negotiables; procedures live in `skills/`, platform rules beside the platform, current contracts in `docs/`, and historical rationale in decision records. Other files link to the rule instead of paraphrasing it.

## Budgets and Format

- Keep root `AGENTS.md` under 800 words and the always-loaded set (root plus task workflow, cross-platform awareness, engineering principles) under 3,000.
- Split or trim skills over about 900 words. Platform `AGENTS.md` files are routing plus non-negotiables.
- Open each skill with its load trigger. State action, verification boundary, and important exception directly.
- Point at verified source files for examples. Prefer links over copied code that will drift; keep snippets only where the shape is clearer than prose.
- Keep commands in development-command guides or verification matrices. Remove transient inventories, counts, obsolete symbols, and session stories from implementation instructions.

## Procedure

1. **Preflight.** Read root `AGENTS.md` and the task workflow. Check checkout, branch, and status; preserve existing changes. Reuse an investigation already completed in this task instead of repeating it.
2. **Inventory.** Inspect root and platform guides, their skills, and linked contracts relevant to the requested audit. Record word counts. Identify duplicate ownership of rules, contradictions, stale references, and instructions that expand task scope.
3. **Read local memory.** Claude uses `~/.claude/projects/<slug>/memory/`, with the checkout path's `/` replaced by `-`; each worktree may have its own slug. Codex's standard directory is `~/.codex/memories/`. An absent directory is not a blocker. Ignore unrelated repositories, credentials, transient status, and material marked private.
4. **Inspect session evidence.** Claude transcripts are `~/.claude/projects/<slug>/*.jsonl`; Codex rollouts are under `~/.codex/sessions/` and `~/.codex/archived_sessions/`. Filter by the repository and its worktrees (`git worktree list`). Use the previous sweep cutoff, otherwise the last 30 days; widen only for an explicitly requested full sweep. Extract user corrections and read surrounding turns to distinguish a rejected design, a changed requirement, and an agent's unsupported assumption. Keep raw transcripts local.
5. **Measure where useful.** Extract compactions, reported input/context sizes, subagent launches by model, and inspection/edit calls. Account for transcript format differences, inherited/forked turns, and task scope. Do not present raw session totals as a controlled comparison of agents. Keep model settings and tool failures out of repository policy.
6. **Choose lessons.** Promote repeated, costly, security-sensitive, or non-obvious lessons supported by source. Group repeats and reject advice already covered. Record rejections for unverified workarounds, unsafe defaults, transient failures, or preferences not established as team policy.
7. **Edit when authorized.** Fix the canonical rule and replace contradictory duplicates with links. Give a new abstraction a current consumer requirement; avoid blanket bans that conflict with valid security, concurrency, or wire-contract testing needs. Preserve user-defined scope and publication boundaries.
8. **Trim and review.** Remove obsolete instructions and historical inventories; keep necessary rationale discoverable separately. Check examples against their named responsibility, not just path existence. Apply both cleanup rounds from the task workflow.
9. **Verify and report.** Run the documentation checks in [Quality Checks](../../../skills/quality-checks.md): whitespace, local links and anchors, cited paths/commands, and contradiction review. Report changes, rejected lessons, word counts, checks, and overlap with pre-existing edits. No app/Core builds for documentation-only work; leave changes uncommitted unless authorized otherwise.

## Shared Skill Layout

`.agents/skills/<name>/SKILL.md` is the canonical source, with YAML `name` and `description`. `.claude/skills` is a symlink to `../.agents/skills`; root and platform `CLAUDE.md` files link to `AGENTS.md`. Preserve those links and edit the shared sources. Personal ignore rules may hide these paths, so check tracked status when adding a skill. Keep agent-specific hooks and local settings out of the repository.
