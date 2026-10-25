# Dynode

Dynode proxies blockchain RPC requests and provider services. It can serve either or both from one process.

Run from `core/`:

```sh
cargo run -p dynode
```

Configuration lives in [core/apps/dynode](../core/apps/dynode):

- `config.yml`: listener address, port, and metrics prefix/source.
- `chains.yml`: node inventory, monitoring, request settings, allowlists, and cache rules. Additional `chains*.yml` files load in filename order and override matching chains.
- `routes.yml`: provider services, endpoints, request settings, allowlists, and cache rules.

Supply the family files you need. Chains and provider routes share the cache implementation but have separate `cache.memory.max` budgets. Keep `allowlist` and `cache` rules separate.

## Routes

Chain requests use `/<chain>`. Provider requests use `/<source>/<service>/<path>`; `source` identifies the caller, and the route's configured `group` is used for metrics.

```mermaid
flowchart TD
    Request["Provider request"] --> Route{"Route exists and request allowed?"}
    Route -->|No| Error["JSON error"]
    Route -->|Yes| Cache{"Cached response?"}
    Cache -->|Hit| Response["Return response"]
    Cache -->|Miss| Endpoint["Select endpoint"]
    Endpoint --> Headers["Forward allowed headers and apply endpoint headers"]
    Headers --> Upstream["Forward request upstream"]
    Upstream --> Filter["Filter response headers and cache when configured"]
    Filter --> Response
```

The local FastNear example allows:

```sh
curl http://localhost:3000/api/fastnear_transfers/
```

`/health` reports process health; `/metrics` exposes Prometheus metrics.

## Caching

Dynode keeps responses in memory, one namespace per chain and one per provider route, with the `cache.memory.max` budget split evenly between them. An entry lives until its rule's TTL ends; when a namespace is full, the oldest entries go first. Each Dynode host keeps its own cache, and only successful responses are kept.

A request is cached only when a rule in `chains.yml` matches it. Three kinds of rule exist:

| Rule | Matches | Example |
|---|---|---|
| `rpc_method` | a JSON-RPC method | Solana `getEpochInfo` |
| `path` with `params` | an HTTP path and method, and top-level JSON body fields equal to `params`; a path ending in `/**` matches everything under it | TON `/api/v3/runGetMethod` whose `method` is `get_pool_address`, Algorand `/v2/assets/**` |
| `contracts.methods` | contract calls decoded from the request: an `eth_call` to one of `addresses` with one of the 4-byte `identifiers`, a Sui transaction whose Move calls are all in `identifiers` on one of `addresses`, or a Solana `getProgramAccounts` of one of `addresses`; the address `*` matches any contract | Uniswap v3 factory `getPool`, Cetus `factory::pool_simple_info`, Meteora DAMM v2 pools, ERC-20 `decimals()` of any token |

The cache key is the request's own content, never its caller, so every wallet asking the same thing shares one entry:

| Request | Key |
|---|---|
| JSON-RPC call | host, path with its query, method and params; the call `id` is left out |
| Any other request | host, HTTP method, path with its query and the full body |

A JSON-RPC answer of `null`, an empty or all-zero hex string such as `0x` or a zero word, or an empty list `[]` is never kept: it says nothing exists yet, and a pending transaction, a contract not deployed yet, a pool not created yet or a token at a precomputed address would otherwise stay missing for the whole TTL (`test_empty_answers_are_not_cached`).

A JSON-RPC batch is checked call by call: cached calls are answered from memory, only the missing calls go upstream as one batch, and each answer is kept under its own key.

A batch follows a chain's overrides call by call. When every call goes to the same override, or to none, the batch goes upstream whole. Otherwise it is split into one batch per override, sent at the same time, and the answers are put back in call order; a part that fails answers each of its calls with a JSON-RPC error, and the other parts still answer (`test_batch_calls_follow_their_overrides`).

Swaps rely on Dynode only for lookups that do not price a quote, what a pool is and which pools a pair has, so a quote is still priced on live state ([Swap](product/swap.md#on-chain-quotes)). Which pools lost to a better one stays on the device: it depends on the amount, and Dynode keeps responses, not comparisons between them.

| Chain | Request | TTL |
|---|---|---|
| Ethereum and EVM networks | `eth_call` `getPool` to the Uniswap v3, PancakeSwap, Aerodrome, Oku and Wagmi factories | `1d` |
| Ethereum and EVM networks | `eth_call` `getSlot0` to the Uniswap v4 StateView; the wallet reads it only to learn whether a pool exists, never for its price | `1d` |
| Sui | `SimulateTransaction` calling Cetus `factory::new_pool_key`, `factory::pool_simple_info` or `factory::pool_id` | `1d` |
| Solana | `getProgramAccounts` of the Meteora DAMM v2 and DLMM, Raydium CPMM and Raydium AMM v4 programs, the pools that hold a pair | `1d` |
| Aptos | `/v1/view` of Hyperion `pool_v3::liquidity_pool_address_safe` or `pool_v3::liquidity_pool_info` | `1d` |
| TON | `runGetMethod` `get_wallet_address`, `get_pool_address` or `get_vault_address` | `30d` |

The production rules live in the infra repository's Dynode template.

## Transaction broadcast metrics

`dynode_transaction_broadcasts_total` counts completed recognized broadcast requests once, after Dynode retries. Labels are `chain`, `source`, `group`, `service`, and `outcome` (`success` or `failure`). Success requires a 2xx response with a nonempty transaction identifier decoded by the chain's existing broadcast provider; it measures RPC acceptance, not on-chain confirmation. Failure includes HTTP or proxy errors and rejected, malformed, or unsupported responses without a usable identifier. A failure does not prove the transaction was never submitted.

`dynode_transaction_broadcast_latency_milliseconds` is a histogram with the same labels, measuring elapsed request time through completion, including retries. It records successes and failures. Metrics run independently of broadcast webhook configuration and delivery. Transaction identifiers, addresses, bodies, and free-form error messages are never metric labels.

Coverage follows the existing webhook request classifiers: single JSON-RPC calls and recognized HTTP/gRPC broadcast paths for configured chains. JSON-RPC batches, unrecognized requests, requests rejected before reaching the node service, and cancelled requests are excluded. Repeated client submissions count separately; these are request counts, not unique transactions.

The Dynode Grafana dashboard in `../infra` has a Transaction Broadcast section above Cache Performance with two panels: a time-series chart of per-chain total, success, and failure counts plus success/failure percentages, and a table of final broadcast errors grouped by chain, remote host, and message. Chart counts and percentages use the selected interval, with percentages on the right axis. Latency remains available as a metric. Source, region, group, and chain filters apply to both panels. Host and provider filters apply to the error table using the final attempted upstream; they do not apply to the per-chain metrics chart. The platform filter does not apply. Chart counts use Prometheus `increase`, so they are estimates from scrapes and cannot recover historical broadcasts before instrumentation was deployed.

Each accepted broadcast emits a `Broadcast accepted` log with its chain, request ID, and returned transaction ID, plus the remote hostname. Each final failure emits a `Broadcast failed` log with chain, chain group, remote hostname, and error message. The hostname comes from the final selected URL after method/path overrides and retries; `unknown` means no upstream was selected. URL credentials, paths, and query strings are omitted. All configured chain families reuse the existing user-facing broadcast decoders, preserving their errors instead of discarding them. When the chain response cannot be deserialized, plain `error`, `error.message`, or top-level `message` responses retain their message; other responses use a generic failure summary and transport errors use a bounded reason. Messages are limited to 1024 characters and one line; response bodies, error data, transaction payloads, and upstream URLs are excluded. The existing server-side Alloy pipeline indexes broadcast logs with `log_type="broadcast"`, allowing the error table to scan only broadcast logs. Error-table counts begin with that indexing rollout and cover retained logs; older entries remain queryable in Loki without the label filter. They may differ from Prometheus estimates.

## Node monitoring

Dynode monitoring keeps one active RPC URL per chain and moves traffic between configured URLs when health changes. The mechanism is chain-agnostic: each chain provider defines how node status and profile checks are performed.

### Core rules

- URL order is priority order. The first configured URL is preferred.
- A node is usable only when its status and configured profile checks succeed and it is in sync.
- If `monitoring.trigger.latency` is set, a node observation must complete within that latency.
- A healthy fallback remains active until a higher-priority URL becomes healthy again.
- Selection uses configuration order among usable nodes.
- Switching is atomic: the selected URL is installed only if the active URL has not changed since the monitoring cycle started.
- Request retries and monitoring are separate. Retries select an upstream for one request; monitoring changes the active URL shared by later requests.

### Failure-triggered checks

Dynode also starts a monitoring cycle when the active URL reaches either configured threshold:

- `trigger.failures`: consecutive retryable failures.
- `trigger.rate`: retryable failure percentage within `trigger.window`, after at least `trigger.failures` failed requests.
- `trigger.latency`: maximum acceptable latency for one node monitoring observation before alternatives are checked.

Transport errors always count. Responses count when any `retry.errors` rule matches. Each rule has an optional HTTP `status` and an optional `message` substring matched case-insensitively against JSON `error` or `error.message`. When both are configured, both must match; a message-only rule applies at any HTTP status. Empty rules and blank message patterns never match. Cache hits and fallback attempts do not count. The window also acts as the cooldown between triggered checks for the active URL.

```yaml
retry:
  errors:
    - status: 503
    - message: "rate limit"
    - status: 400
      message: "502 Bad Gateway"
```

The rolling rate uses bounded time buckets, so memory use does not grow with request volume.

### Monitoring cycle

Monitoring runs only for chains with more than one configured URL.

```mermaid
flowchart TD
    Poll["Poll chain at configured interval"] --> Current["Read active URL"]
    Current --> Preferred{"Active URL is first?"}

    Preferred -- "Yes" --> CheckActive["Observe active URL"]
    CheckActive --> ActiveHealthy{"Healthy and in sync?"}
    ActiveHealthy -- "Yes" --> Keep["Keep active URL"]
    ActiveHealthy -- "No" --> CheckRemaining["Check URLs in priority order until one is usable"]

    Preferred -- "No" --> CheckEarlier["Observe active URL, then check eligible URLs in priority order"]
    CheckRemaining --> Select["Apply ordered selection policy"]
    CheckEarlier --> Select

    Select --> CurrentHealthy{"Active URL healthy and in sync?"}
    CurrentHealthy -- "Yes" --> Earlier["Search only earlier URLs"]
    CurrentHealthy -- "No" --> FullList["Search full configured list"]
    Earlier --> Candidate["Choose first healthy, in-sync URL"]
    FullList --> Candidate

    Candidate --> Found{"Candidate found?"}
    Found -- "No" --> Keep
    Found -- "Yes" --> Switch["Switch only if active URL is unchanged"]
    Switch --> Record["Update metrics and log reason"]
```

The first-URL fast path avoids unnecessary checks while the preferred node is healthy. Other checks stop at the first usable URL in configuration order. A healthy fallback checks only higher-priority URLs; an unhealthy active URL searches the full list.

### Observing one URL

The monitoring layer does not name or require a protocol-specific RPC method.

```mermaid
flowchart TD
    Start["Build client with configured headers"] --> Status["Provider: get_node_status"]
    Status --> StatusResult{"Status succeeded?"}
    StatusResult -- "No" --> Error["Record unhealthy observation"]
    StatusResult -- "Yes" --> Profile["Run configured node-check profile"]
    Profile --> ProfileResult{"Checks succeeded?"}
    ProfileResult -- "No" --> Error
    ProfileResult -- "Yes" --> Healthy["Record status observation"]
    Healthy --> Sync{"In sync?"}
    Sync -- "Yes" --> Usable["Eligible for selection"]
    Sync -- "No" --> Unusable["Not eligible for selection"]
```

`get_node_status` is implemented by each chain provider. For example, an EVM provider may use `eth_blockNumber`, while another chain uses its native status method. If the initial status call fails, profile checks are skipped because the node is already known to be unusable.

Every profile verifies the chain identity and latest block number. Additional checks depend on the configured profile:

- `basic` performs no additional checks.
- `wallet` checks the configured address balance on every chain and adds richer chain-specific checks where available.
- `parser` verifies the latest block number and that the node can return transactions for a recent block.

Failed required checks make the observation unhealthy. Optional checks may be recorded as warnings without rejecting the node.

### Ordered selection

Assume URLs are configured as `A, B, C`, where `A` has the highest priority.

| Active URL | State | Eligible search range | Result |
| --- | --- | --- | --- |
| `A` | Healthy | None | Keep `A`; lower-priority URLs are not checked |
| `A` | Unhealthy | `A, B, C` | Select the first healthy URL |
| `B` | Healthy | `A` | Return to `A` when it is healthy; otherwise keep `B` |
| `B` | Unhealthy | `A, B, C` | Select the first healthy URL |
| `C` | Healthy | `A, B` | Prefer `A`, then `B`; otherwise keep `C` |

An observation must be healthy, in sync, and within the latency threshold when configured. A faster or higher-block lower-priority node does not displace a higher-priority node that is still within threshold.

### Endpoint construction

An empty RPC path uses the configured base endpoint after removing trailing separators. Dynode does not append `/` when no path is requested. For a non-empty path, Dynode preserves an existing leading separator or inserts one when needed.

This matters for authenticated endpoints where `/v1/key` and `/v1/key/` can have different authorization behavior.

### Metrics

Each observed URL publishes its last check success, sync state, latest and current block, check latency, and check timestamp. A cycle counter distinguishes `scheduled` checks from `failure_trigger` checks.

Node-switch metrics use the same stable reasons as request retries: `status=<code>`, `timeout`, `connect_error`, `transport`, `response_body`, `response_decode`, `invalid_rpc_batch`, and the uncategorized fallback `request_error`. Profile failures use `request_error`. Malformed non-2xx JSON-RPC responses retain their HTTP status in retry metrics; malformed 2xx responses use `response_decode` or `invalid_rpc_batch`. Parsing failures remain retryable, and retry limits and routing are unchanged.

`Upstream error` logs include the request ID, chain, remote hostname, HTTP method, RPC methods or HTTP path, stable reason, latency, and a safe error detail. Decode details contain only HTTP status, parser category, line, and column; batch details contain only HTTP status and a bounded validation reason. Transport details identify the transport or response-body stage. Raw exceptions, keyed upstream URLs, and response bodies are excluded. Monitoring error logs retain their bounded type (`upstream`, `request`, or `node_check`) and provider message; raw provider errors are never Prometheus labels.

Each `Retry attempt` log records `original_host` (the first selected upstream), `failed_host` (the preceding failed upstream), `remote_host` (the selected fallback), the attempt number, and the preceding failure reason. Hosts come from the resolved endpoint after method/path overrides and exclude URL credentials, paths, and query strings; `unknown` means URL resolution did not yield a hostname. The fallback-host chart displays the original host and each failed-host → fallback-host transition, grouped by attempt and reason. It measures retry attempts per minute, not unique requests or successful recoveries. Older logs remain visible with missing host context marked `unknown`.

### Code map

- [Monitoring worker](../core/apps/dynode/src/monitoring/worker.rs): creates one monitor per eligible chain.
- [Chain monitor](../core/apps/dynode/src/monitoring/chain_monitor.rs): schedules periodic and failure-triggered checks.
- [Node health evaluator](../core/apps/dynode/src/monitoring/evaluator.rs): observes eligible URLs and applies switches.
- [Node observer](../core/apps/dynode/src/monitoring/node_observer.rs): creates one chain-provider observation.
- [Selection policy](../core/apps/dynode/src/monitoring/selection.rs): applies configured priority and recovery behavior.
- [Telemetry](../core/apps/dynode/src/monitoring/telemetry.rs): records observations and switch outcomes.
- [Node service](../core/apps/dynode/src/node_service.rs): routes requests through the active URL and handles per-request retries.
- [URL construction](../core/crates/gem_client/src/query.rs): joins base endpoints and request paths.
