# Context Efficiency

Load when a task will take more than a handful of tool calls, spawns subagents, runs builds or test suites, or when the session's context has passed a third of the window.

## Cap the Context

- Cap the auto-compact window at 400K tokens on a 1M-context model. Every tool call re-reads the whole prefix, so cost grows with context size, and a compaction that runs late is itself a large request. Claude Code: `/autocompact 400k` once (saved as `autoCompactWindow` in user settings), or `CLAUDE_CODE_AUTO_COMPACT_WINDOW=400000` for scripted and cloud runs. The setting is personal; it is not checked into this repository.
- Clear between unrelated tasks instead of continuing the same session. Claude Code: `/clear`, after `/rename` when the session is worth finding again.
- Compact deliberately before a long new phase, with instructions on what to keep (the diff in progress, the failing test, the verification list). Claude Code: `/compact <what to keep>`. After any compaction, re-read the files you are editing before the next edit; the summary is not the file.
- Keep tool output small. A loop that would take several tool calls is one script run once. Build and test output is read for diagnostics and the summary; the `running N tests` and `test result` lines must survive any filter, because a filtered run that selected zero tests is not a pass (`skills/quality-checks.md`). Never print a lockfile, a generated binding, or a large fixture whole.
- Check where the context goes when a session feels slow or expensive. Claude Code: `/usage` attributes recent usage to skills, subagents, and MCP servers and flags long context and cache misses; `/insights` reports patterns across sessions. The repository-level view is step 5 of the [`guidance-refresh` skill](../.agents/skills/guidance-refresh/SKILL.md).

## Delegate to Subagents

A subagent runs in its own context window and returns only its final report, so its file reads and build logs never enter yours.

- Delegate work whose value is the conclusion, not the trail: a search across many files or crates, a verbose verification run, an audit of a finished diff, or independent tasks that can run in parallel. Keep in the main context anything that needs the conversation so far, edits to the files you are working on, and judgment calls on wallet-critical code (signing, keystore, transaction construction, auth).
- Brief a subagent as a stranger to the session. It loads the guides but not the conversation. Give it the checkout path and branch, the exact files, crates, or commands, the question, and the shape of the answer you want. Ask for facts with `file:line` references, not opinions.
- Match the agent to the job: a read-only search agent for exploration, a plain agent given the exact closing commands from `skills/quality-checks.md` for verification, a security-review agent for adversarial review of a finished diff. Launch independent subagents in one message; do not launch more than the task needs, since each one loads the guides afresh.
- Relay what comes back as second-hand evidence. Name the agent and repeat the exact commands and results it reported; do not re-run a search a subagent already finished, and do not upgrade its summary into a claim you verified yourself. The handoff rules in `skills/task-workflow.md` § 5 apply to delegated work unchanged.
- Route subagents to a cheaper model by default and pin the exceptions. Claude Code: `CLAUDE_CODE_SUBAGENT_MODEL=sonnet` in user settings sets the default, and an agent definition in your own `~/.claude/agents/` pins its `model:` and `effort:`. A security-review agent stays on the session model; a cheaper model must never be the only reviewer of a wallet-critical change.
