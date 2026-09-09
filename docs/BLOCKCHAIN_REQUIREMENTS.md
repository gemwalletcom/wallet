# New Blockchain Requirements

Contact the Gem Wallet team through a [GitHub issue](https://github.com/gemwalletcom/wallet/issues/new/choose) with supporting evidence before starting integration work. Every addition requires team review and approval, even when all requirements are met.

## Hard requirements

Requests that fail any of these requirements are declined or deferred.

- **Live mainnet:** A working production network with documented finality and no unresolved critical security issues.
- **Network security:** A documented consensus and trust model, with public evidence of how control is distributed and who can halt, censor, or reorganize the chain. Disclose dependencies on other networks.
- **Audited security:** Public independent audits covering deployed consensus and wallet-critical cryptography, with remediation records. Key generation, derivation, and signing must have documented specifications and test vectors; algorithm standardization alone is insufficient.
- **Self-custody:** Local signing and reproducible backup recovery without exposing secrets or depending on a proprietary service.
- **Open infrastructure:** Public node and SDK source under compatible licenses, documented RPCs, an explorer, and reliable balances, transaction history, and final status.
- **Integration support:** Compatibility with shared Core, iOS, and Android, plus a maintained test environment and a responsive technical/security contact.

## Review factors

Meeting the minimums does not guarantee acceptance. We also assess:

- **Adoption:** Prefer assets in CoinMarketCap's top 50 by circulating market cap, with sustained trading volume, usable liquidity, and real user demand. Rank alone is insufficient.
- **Decentralization:** Concentration of hash power, stake, or other consensus authority, as applicable; admin keys, upgrade authority, and sequencer control.
- **Reliability:** Mainnet operating history, outages, incident response, and independent RPC providers or practical self-hosting.
- **Maintenance cost:** Release quality, custom cryptography, mobile performance, dependencies, and long-term support commitments.

Include evidence for each requirement and review factor, plus requested capabilities and known gaps. See [Core Features and Providers](FEATURES.md#core-features-and-providers) for current coverage.
