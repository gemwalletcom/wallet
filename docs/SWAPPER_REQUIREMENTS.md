# New Swapper Requirements

Contact the Gem Wallet team through a [GitHub issue](https://github.com/gemwalletcom/wallet/issues/new/choose) with supporting evidence before starting integration work. Every addition requires team review and approval, even when all requirements are met.

## Hard requirements

Requests that fail any requirement are declined or deferred.

- **Referral or revenue share:** Supported for Gem Wallet's integration, with agreed fee attribution, reporting, and payout terms. All user-facing fees must be explicit in quotes.
- **Clear added value:** Demonstrably better execution after all fees, or meaningful coverage of missing assets, chains, or routes. Provide representative comparisons against current providers; duplicating existing liquidity alone is insufficient.
- **Quote performance and API capacity:** Quotes return in under one second under normal conditions. APIs must permit server-side proxying and provide agreed sustained, burst, and concurrency limits sufficient for Gem's shared traffic, including status polling, with documented throttling and a capacity upgrade path.
- **Security:** Document deployed contracts, custody, and upgrade authority, with public independent audits for contracts handling funds and remediation records. Spenders, routers, and transaction targets must be independently verifiable; API responses alone cannot authorize approvals.
- **Safe execution:** Provide clear quotes, fees, and execution conditions, with enough information for Gem Wallet to validate what the user signs.
- **Cross-chain tracking:** A status API or independently queryable on-chain mechanism is mandatory to track pending, completed, failed, and refunded swaps. Provide source, destination, and refund transaction references where applicable, plus documented refund conditions and recovery procedures.
- **Support and integration:** A responsive engineering contact, agreed incident escalation, API change notices, and test access. Integration must satisfy the [Swapper Provider Integration Checklist](../core/skills/swapper-checklist.md) through shared Core on iOS and Android.

## Review factors

- **Direct integration:** Prefer on-chain protocols and direct liquidity access over API-only services or additional aggregator layers. Provider APIs remain acceptable where necessary, particularly for cross-chain protocols.
- **Execution quality:** Consistent net output, liquidity depth, success rate, and settlement speed across representative routes and trade sizes.
- **Operational quality:** Reliable service, prompt support, transparent incidents, and manageable dependencies and maintenance costs.

Include evidence, commercial terms, API quotas, and known gaps. See [Swap providers](FEATURES.md#swap-providers) for current coverage.
