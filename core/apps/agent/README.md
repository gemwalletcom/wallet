# agent

Gem Wallet's internal agent engine — a Rust service that bridges Slack (Socket Mode) and Chatwoot
to an LLM, with tool use, a per-agent long-term memory (sqlite vector store), and a scheduler.

## Layout

- `src/` — the engine (slack/chatwoot dispatch, tools, scheduler, memory store).
- `Settings.yaml` — generic, non-secret defaults. Tokens come from the environment at runtime.
- `agents/<name>/` — one directory per agent: `agent.yaml` (model, tools, channels, allowed
  context), `role.md` (the system role), optional `schedules/` and `memory/`.
- `context/*.md` — shared knowledge included into an agent's preamble via `include_context`.

This repo ships a single generic demo agent, **`security`**, which reviews the public
`gemwalletcom/wallet` repo for security issues. It contains no secrets and no operational data.

Production agents and their operational context (roles, channels, team roster, etc.) are private
and live in the deployment repo (`playbooks`), bind-mounted over `Settings.yaml`, `agents/`, and
`context/` at deploy time. The runtime memory (vector index) is a persisted volume.

## Run locally

```sh
cp .env.example .env   # fill in tokens
AGENT_NAME=security cargo run -p agent

# one-shot REPL against an agent definition:
cargo run -p agent --bin repl -- security
```

## Configuration

Config is layered (later overrides earlier): `Settings.yaml` → `agents/<name>/agent.yaml` →
environment variables (`_`-separated, e.g. `SLACK_BOT_TOKEN` → `slack.bot.token`). The agent to
run is selected by `AGENT_NAME` (or `argv[1]`).
