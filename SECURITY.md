# Security Policy

Gem Wallet is a self-custodial wallet: keys are generated, encrypted and used for signing on the user's device, and keeping them safe is our highest priority. This repository holds the iOS and Android apps, their shared Rust core (Gemstone) and the backend services.

## Reporting a vulnerability

Report privately, by either channel:

- Email [security@gemwallet.com](mailto:security@gemwallet.com)
- [GitHub private vulnerability reporting](https://github.com/gemwalletcom/wallet/security/advisories/new)

Do not open a public issue or pull request with exploitable details. We aim to respond within 48 hours.

Please include:

- The affected app version, platform and chain
- The flow involved (send, swap, WalletConnect, import, export, etc.)
- A proof of concept against the latest public app or an in-scope service, and the impact it shows
- A proposed fix, if you have one

## Scope and rules

Scope, out-of-scope issues, the requirements for AI-assisted findings and the disclosure rules are in our [Responsible Disclosure policy](https://gemwallet.com/security/bug-bounty/). Only the latest public iOS and Android releases are supported.

## Audits and practices

- [Security overview](https://gemwallet.com/security/): key material, signing and secure storage
- [CertiK Security Audit (April 2026)](https://static.gemwallet.com/audits/Gem-Wallet-CertiK-Security-Audit-April-2026.pdf)
