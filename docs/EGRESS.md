# Provider Egress

[Dynode](../core/apps/dynode/src/main.rs) serves blockchain nodes and backend provider routes from one binary and container image with separate settings for each route family. Configured capabilities determine the routes available; deployments can enable either or both while retaining separate containers for workload isolation.

## Ownership

The [server](../core/apps/dynode/src/server.rs) owns startup, health, and metrics endpoints. [Shared HTTP transport](../core/apps/dynode/src/proxy/transport.rs) handles forwarding, and [path rules](../core/apps/dynode/src/config/path.rs) provide matching for node and provider routes. The [provider endpoint](../core/apps/dynode/src/gateway/endpoint.rs) owns request pacing for its configured rate limit.

Node routing retains chain monitoring, JSON-RPC error classification, and transaction webhooks. Regional Nginx applies the former authenticated limits to all requests: 1,000 per minute with burst 500 and 100 per second with burst 250, without JWT subrequests. Dynode no longer exposes `/auth`; metrics remain protected by Nginx basic authentication. The [provider gateway](../core/apps/dynode/src/gateway/mod.rs) retains provider key selection, endpoint quotas, outbound proxy health, and path-scoped cooldowns. These policies remain distinct because their inputs and failure handling differ. Each deployed process retains its own in-memory limits and health state.

Node and provider routes share one metrics registry and the `dynode` metric prefix. Common traffic dimensions are `source`, `group`, and `service`; `region` remains a deployment label. Provider groups retain their configured names. Ethereum-family chains use `group="evm"`, and node service names are chain names. API and parser configurations explicitly set `metrics.source` to `api` and `parser`; consumer and daemon containers override it through `METRICS_SOURCE` because they share the API configuration. Regional instances explicitly configure `public`. Provider source labels identify the source segment supplied by the internal caller. The shared server exposes `/metrics` and `/health`. Locally generated routing errors use the same JSON error envelope as the API (`{"error":{"message":"request not allowed"}}`) with the corresponding HTTP status; forwarded upstream responses and JSON-RPC error payloads retain their own formats.

The Dynode dashboard combines request outcomes, upstream attempts, latency, quotas, cooldowns, chain monitoring, caching, RPC methods, and logs. The old `egress_*` traffic families become `dynode_*`; detailed chain metrics retain their names and gain source/group/service labels. Node monitor cycles use `trigger` for periodic/failure checks, separate from traffic source.

## Routing

Internal requests use `/<source>/<service>/<upstream-path>`, for example `/api/fastnear_transfers/v0/transfers`. The source identifies the caller in metrics and cache partitions; it is not an authentication credential. Service names identify the provider and operation and are globally unique, and each route has a required `group` used for metrics and cache organization. There are no caller keys or caller authorization settings. When chains and providers run together, chain names are reserved as first path segments; use application names such as `api`, `parser`, or `consumer` for provider sources. Routes select upstream endpoints and configure rate limits, retries, and optional outbound proxies.

```mermaid
flowchart TD
    Request["Caller request"] --> Access{"Configured route and allowed path/method?"}
    Access -->|No| Reject["Reject request"]
    Access -->|Yes| Route["Select route and available endpoint"]
    Route --> Filter["Filter inbound headers"]
    Global["Global headers.forward"] --> Allowed["Merge allowed header names"]
    Service["Selected service headers.forward"] --> Allowed
    Allowed --> Filter
    Filter --> Inject["Apply endpoint headers; configured values win"]
    Credentials["Endpoint headers with environment variables expanded"] --> Inject
    Inject --> Upstream["Send to upstream provider"]
    Upstream --> Response["Filter response headers"]
    Allowed --> Response
    Response --> Caller["Return status and body to caller"]
```

## FastNear Demo

Start `cargo run --locked -p dynode` from `core/`, then call the sample route:

```sh
curl http://localhost:3000/api/fastnear_transfers/v0/transfers \
  -H 'Content-Type: application/json' \
  -d '{"account_id":"root.near","limit":10,"desc":true}'
```

`api` is the source and `fastnear_transfers` the service. Its configured `group: indexer` is used in metrics and does not appear in the URL. The [route inventory](../core/apps/dynode/routes.yml) allows `GET /` for provider documentation and `POST /v0/transfers` for transfers, matching the [FastNear Transfers API](https://github.com/fastnear/transfers-api). The sibling transaction demo allows only `POST /v0/transactions`. The source needs no configuration entry or key.

## Headers

- Global `headers.forward` contains shared HTTP headers.
- `routes.<service>.headers.forward` adds headers for that service only. Omitted lists inherit the global set.
- Header names are case-insensitive, deduplicated, and validated when routes load.
- Endpoint `headers` inject configured values after forwarding and override matching inbound values.
- The effective forwarding list also filters upstream response headers.

```yaml
headers:
  forward: [accept, content-type, content-encoding, content-length, te, user-agent]
routes:
  security_hashdit:
    group: security
    selection: ordered
    headers:
      forward: [x-api-key]
    endpoints:
      - name: direct
        url: https://service.hashdit.io
  security_tronscan:
    group: security
    selection: ordered
    endpoints:
      - name: key_1
        url: https://apilist.tronscanapi.com
        headers:
          TRON-PRO-API-KEY: "${TRONSCAN_API_KEY}"
```

Forward dynamic signatures and caller-provided credentials only on the routes that consume them. Supply static upstream keys through environment variables, with placeholders in endpoint configuration. A key injected by an endpoint does not need a forwarding entry. Never commit real credentials.

## Caching

Node and provider routes reuse the same cache implementation with separate instances and memory budgets. Each family file sets `cache.memory.max`, including cached bodies and stored headers. Node caching rules live in `chain_types.<type>.cache`, with chain-specific rules under `chain_types.<type>.chains.<chain>.cache`; contract methods live in the corresponding `contracts.methods` list. Access rules remain separate under `allowlist`. Provider caching is opt-in through `routes.<service>.cache`, using the same path, method, optional body parameters, and TTL rule fields. Omitting rules preserves uncached provider forwarding. Provider cache entries are partitioned by source, group, and service. Cache hit and miss metrics use the same source/group/service/path dimensions.

## Configuration and Rollout

The root [configuration](../core/apps/dynode/config.yml) explicitly defines `address`, `port`, `metrics.prefix`, and `metrics.source`. Metrics settings have no Rust defaults. `DYNODE_CONFIG` selects this file, followed by the compatibility `EGRESS_CONFIG` override and the default `config.yml`. Sibling `chains.yml` owns node request limits, timeouts, headers, retry policy, monitoring, webhooks, cache budget, chain-type policies, and inventory. Sibling `routes.yml` owns provider request limits, timeouts, headers, retry policy, proxies, cache budget, and inventory. Each family can be enabled independently by supplying its file. Optional `chains*.yml` files load in filename order; later files overlay node settings and replace matching chain inventory entries. Endpoint URLs, headers, and query values retain environment-placeholder expansion; existing environment overrides apply to each family.

The Dynode image includes only the root `config.yml`. Deployments mount the required family files alongside it: node containers supply `chains.yml` and any regional overrides, while provider containers supply `routes.yml`. A combined container supplies both. Provider deployments retain their environment files and run `/app/dynode`. Existing node and provider containers remain isolated even though each uses the same executable and configuration model.

Deploy the merged image and migrated family files together on restart. The previous root configuration layout cannot start the new binary. Pause scheduled image-only updates during the transition, stage the new image and configuration, then restart the affected containers with both in place before resuming the scheduler. Deploy the explicit source settings and consumer/daemon overrides as part of that migration.

The provider container is named `dynode_egress` and uses the Dynode image, retaining its metrics port, upstream provider secrets, and process isolation. Internal provider URLs remove the key and group segments, and their templates no longer require `EGRESS_*_KEY` credentials. Coordinate a full API deployment for the gateway, nginx, and backend containers consuming `.env.egress.*`: the previous keyed URL format is no longer supported, and an image-only gateway restart does not reload backend URL environments. During the first migration, preserve the old Egress service until every backend consumer has received the new URLs. The shared Compose deployment removes orphan services, so directly replacing the service definition would stop the old gateway before those consumers restart. A temporary Compose override must retain the old service with its original image, configuration, and environment while the new gateway uses a temporary metrics host port; both gateways otherwise compete for the same port. Remove that override and restore the configured metrics port after the consumer cutover. After migration, the `dynode-image` deployment scope updates all node and provider instances. There is no separate Egress image build or scheduled image deployment. Reload the updated regional Nginx configuration before restarting Dynode so no request still calls the removed `/auth` endpoint. Deploy the unified dashboard alongside the metrics migration; historical `egress_*` series retain their old names in storage.

API and parser retain separate `chains.api.yml.j2` and `chains.parser.yml.j2` templates, with independent node lists and credentials. Each renders its own chain inventory and is mounted into its corresponding container. Regional node cache and monitoring settings remain separate from backend profiles.

## Verification

Run from `core/`:

```sh
cargo test --locked -p dynode --all-features --lib --bins
cargo clippy --locked -p dynode --all-features --all-targets -- -D warnings
cargo build --locked -p dynode
cargo test --locked -p dynode --test proxy_integration --features proxy_integration_tests
docker build --target dynode --tag dynode-merged:test --progress plain .
DYNODE_TEST_IMAGE=dynode-merged:test cargo test --locked -p dynode --test proxy_integration --features proxy_integration_tests
```

Validate configuration through loader tests and local startup with synthetic credentials. The original optional chain argument remains available, for example `dynode solana`; ordinary `/app/dynode` startup enables every configured capability. Required provider environment variables must be available at startup. Startup logs sorted, comma-separated lists of chain names and provider service IDs through `gem_tracing` at info level. Provider requests still include an application source such as `api` or `consumer`.

The integration test starts isolated fake upstreams and checks combined routing, configuration overrides, independent family request limits, timeouts and cache budgets, credentials, wire data, cache isolation, retries, cooldowns, pacing, and metrics. It makes no production requests. Compare rendered infra configurations before deployment.
