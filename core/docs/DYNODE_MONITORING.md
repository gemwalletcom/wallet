# Dynode Node Monitoring

Dynode monitoring keeps one active RPC URL per chain and moves traffic between configured URLs when health changes. The mechanism is chain-agnostic: each chain provider defines how node status and profile checks are performed.

## Core rules

- URL order is priority order. The first configured URL is preferred.
- A node is usable only when its status and configured profile checks succeed and it is in sync.
- A healthy fallback remains active until a higher-priority URL becomes healthy again.
- Selection uses configuration order, not block height or latency.
- Switching is atomic: the selected URL is installed only if the active URL has not changed since the monitoring cycle started.
- Request retries and monitoring are separate. Retries select an upstream for one request; monitoring changes the active URL shared by later requests.

## Monitoring cycle

Monitoring runs only for chains with more than one configured URL.

```mermaid
flowchart TD
    Poll["Poll chain at configured interval"] --> Current["Read active URL"]
    Current --> Preferred{"Active URL is first?"}

    Preferred -- "Yes" --> CheckActive["Observe active URL"]
    CheckActive --> ActiveHealthy{"Healthy and in sync?"}
    ActiveHealthy -- "Yes" --> Keep["Keep active URL"]
    ActiveHealthy -- "No" --> CheckRemaining["Observe remaining URLs concurrently"]

    Preferred -- "No" --> CheckAll["Observe all URLs concurrently"]
    CheckRemaining --> Select["Apply ordered selection policy"]
    CheckAll --> Select

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

The first-URL fast path avoids unnecessary checks while the preferred node is healthy. When a fallback is active, all URLs are checked so an earlier URL can reclaim priority as soon as it recovers.

## Observing one URL

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
- `wallet` adds methods needed by wallet clients.
- `parser` verifies the latest block number and that the node can return transactions for a recent block.

Unsupported or failed required checks make the observation unhealthy. Optional checks may be recorded as warnings without rejecting the node.

## Ordered selection

Assume URLs are configured as `A, B, C`, where `A` has the highest priority.

| Active URL | State | Eligible search range | Result |
| --- | --- | --- | --- |
| `A` | Healthy | None | Keep `A`; lower-priority URLs are not checked |
| `A` | Unhealthy | `A, B, C` | Select the first healthy URL |
| `B` | Healthy | `A` | Return to `A` when it is healthy; otherwise keep `B` |
| `B` | Unhealthy | `A, B, C` | Select the first healthy URL |
| `C` | Healthy | `A, B` | Prefer `A`, then `B`; otherwise keep `C` |

An observation must be both healthy and in sync to be selected. A faster or higher-block lower-priority node does not displace a healthy higher-priority node.

## Endpoint construction

An empty RPC path uses the configured base endpoint after removing trailing separators. Dynode does not append `/` when no path is requested. For a non-empty path, Dynode preserves an existing leading separator or inserts one when needed.

This matters for authenticated endpoints where `/v1/key` and `/v1/key/` can have different authorization behavior.

## Code map

- [Monitoring worker](../apps/dynode/src/monitoring/worker.rs): schedules checks, controls the fast path, and applies switches.
- [Node observer](../apps/dynode/src/monitoring/node_observer.rs): creates one chain-provider observation.
- [Selection policy](../apps/dynode/src/monitoring/selection.rs): applies configured priority and recovery behavior.
- [Telemetry](../apps/dynode/src/monitoring/telemetry.rs): records observations and switch outcomes.
- [Node service](../apps/dynode/src/node_service.rs): routes requests through the active URL and handles per-request retries.
- [URL construction](../crates/gem_client/src/query.rs): joins base endpoints and request paths.
