---
name: guidance-refresh
description: Mine this machine's own agent history and memory for this repository (Claude Code session transcripts and project memory, Codex session rollouts and memories), find corrections and lessons the shared guides are missing, measure session efficiency, and promote the lessons into the canonical AGENTS.md, skills, or docs file while deduping and trimming. Documentation-only; leaves edits uncommitted. Use when asked to refresh, audit, or trim agent guidance, after a run of sessions with corrections, or on the monthly cadence.
---

# Guidance Refresh

Each teammate runs this on their own machine. It reads the history and memory their agents accumulated for this repository, which never leave the machine, and turns the missing lessons into shared guidance. This file is both the policy (how guidance improves, budgets, skill format) and the procedure.

Documentation-only. Do not change code, generated files, configuration, dependencies, or external systems. Do not commit, rebase, push, open PRs or issues, post comments, or resolve threads unless separately requested.

## How Guidance Improves

1. **Capture during the task.** `skills/task-workflow.md` § 6 makes every handoff answer one question: did this task produce a lesson (a user correction, a dead end that cost more than a few tool calls, a guide instruction that proved wrong, a workflow that took trial and error)? A wrong instruction is fixed in the same change. Everything else is saved to the agent's own memory so the next sweep finds it, and applied right away when the task already touches guidance or the user asks.
2. **Sweep locally.** Each teammate runs this skill on their machine, monthly or after a run of sessions with corrections. Different people hit different problems, so the union of these sweeps is what keeps the guides complete. Only distilled rules leave the machine.
3. **Review as policy.** The edits go up as a pull request and are reviewed like any other change. Human review is the approval gate; nothing here self-applies to `main`.

Per-turn nudging is deliberately not used; the handoff question costs one sentence. Agent-specific hooks that call step 1 stay in local configuration, not in this repository.

## Budgets

- The always-loaded set (root `AGENTS.md`, `skills/task-workflow.md`, `skills/cross-platform-awareness.md`, `skills/engineering-principles.md`) stays under 3,000 words; root `AGENTS.md` under 800.
- A skill over about 900 words is split by trigger or trimmed. A platform `AGENTS.md` is routing plus non-negotiables only.
- One canonical statement per rule; every other mention is a link.

## Skill Format

- Title, then one opening sentence that says when to load the skill.
- Rules state trigger, required action, verification boundary, and important exception. Direct sentences, no session stories, no names, quotes, or local paths.
- Repo-specific idioms point at a real file with `Reference:`; verify the path exists. No good/bad sample code: a fragment appears only when it is shorter than the words needed (a one-line API reveal) or when a shape is hard to describe (a YAML flow, the service layout in `docs/ARCHITECTURE.md`). Command listings stay in the `development-commands.md` files.
- The index line in the owning `AGENTS.md` says what the skill covers in one clause.

## Layout of Runnable Skills

- `.agents/skills/<name>/SKILL.md` is the source, in the `agentskills.io` format Codex reads: YAML frontmatter with `name` and `description` (when to use it), then the body.
- `.claude/skills` is a committed symlink to `../.agents/skills`, so Claude Code sees every skill without a copy.
- Personal global gitignores commonly exclude `.agents/` and `.claude/`; the repository `.gitignore` re-includes exactly `.agents/skills/` and the `.claude/skills` symlink and keeps ignoring the rest (`settings.local.json`, worktrees).
- Add a skill the same way, and add an index line in the owning `AGENTS.md` when agents should discover it from the guides.

## Procedure

1. **Preflight.** From the repository root run `git status --short` and preserve every existing change. Read the root `AGENTS.md` and `skills/task-workflow.md`.
2. **Inventory the guides.** Root `AGENTS.md`, `skills/*.md`, `core/AGENTS.md` and `core/skills/*.md`, `ios/AGENTS.md` and `ios/skills/*.md`, `android/AGENTS.md` and `android/skills/*.md`, and the `docs/*.md` they link. Record word counts against § Budgets above.
3. **Read your agent memory first.** It is already distilled, so it is the richest source. Default locations; another agent knows its own.
   - Claude Code: `~/.claude/projects/<slug>/memory/`, where `<slug>` is the repository path with `/` replaced by `-`. Each worktree checkout has its own slug.
   - Codex: `~/.codex/memories/`.
   Skip entries about other repositories, transient status, credentials, and anything the operator marks private.
4. **Mine the session transcripts for corrections.** User turns that redirect the agent are the highest-signal lessons. Write your own extraction for the run; the sources are:
   - Claude Code: `~/.claude/projects/<slug>/*.jsonl`, one file per session, each line a JSON event with the working directory and timestamp.
   - Codex: `~/.codex/sessions/` (dated subfolders) and `~/.codex/archived_sessions/`, one rollout file per session; the working directory is in the session and turn metadata.
   Filter to sessions whose working directory is this repository or one of its worktrees (`git worktree list`), and to turns after the date cutoff: the date of your previous sweep if you know it, otherwise the last 30 days. Older history has either been promoted already or was not worth promoting; widen the window only when the operator asks for a full sweep. Look for correction language such as "don't", "instead", "no need", "only", "wrong", "from now on", then read the surrounding turns to understand what the agent did and what the user wanted instead.
5. **Measure session efficiency.** From the same sessions and date window as step 4, extract per session: the context size of each model call against the auto-compact window and how many calls exceed it, the number of compactions, subagent launches by model, and the ratio of inspection tool calls to edit calls. Write the extraction for your own agent's transcript format, as in step 4. A pattern that repeats across sessions is a lesson candidate when a guide could remove it (a missing pointer to where something lives, a verbose command that needs a quieter recipe); otherwise it is a local setting (the compaction window, the subagent model default) and stays out of the repository.
6. **Write the candidate list.** One sentence per lesson with its trigger, the action, and a proposed home. Group repeats; a lesson that appears in two or more sessions or in both tools is a strong candidate.
7. **Decide.** Promote a lesson that is repeated, costly to rediscover, security-sensitive, or a non-obvious architectural decision, and that the current guides do not already state. Reject transient paths, SHAs, versions, service status, tool or network or auth failures, credentials, unverified workarounds, and personal preferences that are not team policy. Record each rejection with its reason.
8. **Edit the canonical source only.** Root `AGENTS.md` stays routing plus non-negotiables. Prefer an existing skill; create a new one only for a coherent reusable workflow. Platform rules go beside the platform, architectural rationale in `docs/DECISIONS.md`, subsystem contracts in `docs/`. Follow § Skill Format above. Never copy transcript text, names, quotes, or local paths into the guides. Preserve the `CLAUDE.md` symlinks and edit the shared `AGENTS.md` sources only.
9. **Dedupe and trim.** For each rule touched, search the other guides for the same rule and replace copies with a link. Delete inventories that rot (lists of files with a smell, debt lists, counts) in favor of the rule alone. Check every skill still opens with a load trigger and every index line still matches its file.
10. **Verify.** Run `git diff --check`, resolve every relative markdown link and cited path in the changed guides, and re-read every changed file for contradictions. Do not run app or Core builds for a documentation-only change.
11. **Report and hand off.** List what was promoted with its target file, what was rejected and why, files changed, word counts before and after against the budgets, checks run, and overlap with uncommitted changes from other agents. Leave the edits uncommitted for the operator to review and open as a pull request.
