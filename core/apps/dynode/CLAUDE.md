# Dynode

Dynode serves blockchain nodes and provider routes from one binary and container image with separate family configuration files. Configured capabilities enable either or both route families. Read the [provider gateway contract](../../../docs/EGRESS.md) for configuration, policy ownership, and deployment compatibility.

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

`DYNODE_CONFIG` selects the root configuration; `EGRESS_CONFIG` remains a compatibility override before the default `config.yml`. Root settings are only address, port, and metrics. Sibling `chains.yml` owns node request limits/timeouts, headers, retry, monitoring, webhooks, cache budget, chain-type policy, and inventory. Sibling `routes.yml` owns provider request limits/timeouts, headers, retry, proxies, cache budget, and inventory. Sorted sibling `chains*.yml` files overlay node settings and replace matching chain configurations. Each family uses its own cache instance and `cache.memory.max` budget. Node caching rules live in `chain_types.<type>.cache`, independently of `allowlist`, with chain overrides under `chain_types.<type>.chains.<chain>.cache` and contract methods under `contracts.methods`. Provider rules stay on each route. The image includes only root settings; deployments mount the required family files. Environment placeholders retain their existing behavior. `metrics.source` identifies node traffic: API and parser configure their own source; consumer and daemon use `METRICS_SOURCE` overrides with the shared API configuration. Regional sources are configured as `public`.

Route IDs identify the provider and operation and are globally unique with `group` stored in each route configuration for metrics and cache organization. Provider requests use `/<source>/<service>/<path>` without internal caller credentials; source identifies traffic in metrics and cache partitions. Keep node/provider retry and health policies explicit. Consolidate shared mechanics without changing deployment isolation.

## Commands

Run from `core/`:

```sh
cargo build --locked -p dynode
cargo run --locked -p dynode
cargo test --locked -p dynode --all-features
cargo clippy --locked -p dynode --all-features --all-targets -- -D warnings
```
