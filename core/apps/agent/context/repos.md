# Where the product lives

Canonical, public sources of truth for anything Gem Wallet. Anything else (third-party blogs,
AI summaries, old screenshots) is unverified.

- **`gemwallet.com`** — marketing site, public-facing positioning.
- **`docs.gemwallet.com`** — public product docs: chain support, stablecoins, DeFi, end-user
  security guidance.
- **`github.com/gemwalletcom`** — source-code org. All product behavior is decided here. When
  code and docs disagree, code wins.

## Product repos

- **`gemwalletcom/wallet`** — the mobile monorepo. iOS (SwiftUI) + Android (Kotlin/Compose) plus
  the Rust engine under `core/` (`apps/`, `crates/`, `bin/`, `gemstone/`). Handles transaction
  indexing, prices/charts, fiat ramps, name resolution, NFTs, swaps, staking. Look here for UI
  bugs, app-store behavior, protocol/chain support, and anything backend or API-shaped.
- **`gemwalletcom/docs`** — source for https://docs.gemwallet.com/.

## How to use

- Chain support / feature behavior → grep `core/` and the chain crates.
- Repo references in replies use a label-up-front markdown link, never a bare URL.
