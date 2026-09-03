# Name Resolver

Resolves human-readable names to addresses. Every provider lives in `src/providers/<provider>/` with a `client.rs` (network calls), `model.rs` (response types), and `provider.rs` (`NameResolver` implementation).

## Supported Name Providers

- [Ethereum Name Service](https://ens.domains/) - `.eth`, `.com`, `.xyz`, `.dev`
- [Basenames](https://www.base.org/names) - `.base.eth`
- [Unstoppable Domains](https://unstoppabledomains.com/) - `.crypto`, `.nft`, `.wallet`, `.x`, and other UD TLDs
- [Solana Name Service](https://www.sns.id/) - `.sol`
- [AllDomains](https://alldomains.id/) - `.skr`, `.saga`, `.poor`, `.bonk`, `.solana`
- [TON DNS](https://dns.ton.org/) - `.ton`
- [Aptos Names](https://www.aptosnames.com/) - `.apt`
- [SuiNS](https://suins.io/) - `.sui`
- [Injective Name Service](https://injective.id/) - `.inj`
- [Interchain Nameservice](https://www.icns.xyz/) - `.cosmos`, `.osmo`, `.celestia`, `.sei`
- [d.id](https://d.id/) - `.bit`
- [Lens](https://www.lens.xyz/) - `.lens`
- [Space ID](https://space.id) - `.bnb`, `.arb`
- [Hyperliquid Names](https://hlnames.xyz/) - `.hl`
- NEAR accounts - `.near`
