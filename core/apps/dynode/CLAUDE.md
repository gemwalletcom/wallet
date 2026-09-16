# Dynode

Read [README.md](README.md) for local usage and configuration.

## Architecture

- `src/main.rs` loads configured capabilities and preserves the optional positional chain argument.
- `src/server.rs` owns server startup, health, metrics, and configured routing.
- `src/node_service.rs` and `src/monitoring/` own chain health and node selection.
- `src/proxy/` owns blockchain request forwarding, JSON-RPC behavior, and shared HTTP transport.
- `src/gateway/` owns provider routes, source attribution, endpoint selection, quotas, and cooldowns.
- `src/config/` owns the shared configuration, node and provider settings, path rules, and cache rules.
- `src/cache/` owns the reusable cache implementation, independent family budgets, and node decoders; `src/webhook.rs` owns transaction webhooks.
- `src/metrics/` owns request metrics and chain monitoring metrics.

## Configuration

`DYNODE_CONFIG` selects the root configuration; `EGRESS_CONFIG` remains a compatibility override. Preserve environment expansion and ordered chain-file overlays when changing the loader.

Keep node/provider retry and health policies explicit. Reuse shared transport and cache mechanics without coupling their settings or state.

## Commands

Run from `core/`:

```sh
cargo build --locked -p dynode
cargo run --locked -p dynode
cargo test --locked -p dynode --all-features
cargo clippy --locked -p dynode --all-features --all-targets -- -D warnings
```
