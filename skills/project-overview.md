# Project Overview

Use when you need the repo layout, the layer split between apps and Core, or which guide owns an area.
Gem Wallet is an open-source, multi-chain cryptocurrency wallet monorepo with native iOS and Android apps backed by a shared Rust core.

```
wallet/
├── ios/          # SwiftUI app (Swift, SPM, Xcode)
├── android/      # Kotlin/Compose app (Gradle, Hilt)
├── core/         # Shared Rust blockchain engine (FFI -> Swift, JNI -> Kotlin)
└── justfile      # Root task runner
```

The apps share the same product domains: chains, assets, wallets, transactions, staking, swaps, and WalletConnect.

## Layers

| Layer | iOS | Android | Shared |
|-------|-----|---------|--------|
| UI | SwiftUI + MVVM | Jetpack Compose + MVVM | — |
| DI | Environment injection | Hilt | — |
| Data | Store (SQLite) + Services | Room + Repositories | — |
| Blockchain and domain services | Gemstone (FFI) | Gemstone (JNI) | Rust `core/` |

Domain decisions live in Core-owned Gemstone services; each app implements the store trait and maps the returned record to its UI. The reference is [docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md); the migration backlog is [docs/SERVICES.md](../docs/SERVICES.md).

Read the repo root guide first, then load the relevant platform guides:

- [`ios/AGENTS.md`](../ios/AGENTS.md)
- [`android/AGENTS.md`](../android/AGENTS.md)
- [`core/AGENTS.md`](../core/AGENTS.md)
