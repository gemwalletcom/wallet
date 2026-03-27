# Project Overview

Gem Wallet is an open-source, multi-chain cryptocurrency wallet monorepo with native iOS and Android apps backed by a shared Rust core.

```
wallet/
├── ios/          # SwiftUI app (Swift, SPM, Xcode)
├── android/      # Kotlin/Compose app (Gradle, Hilt)
├── core/         # Shared Rust blockchain engine (FFI -> Swift, JNI -> Kotlin)
└── justfile      # Root task runner
```

The apps share the same product domains: chains, assets, wallets, transactions, staking, swaps, and WalletConnect.

Read the repo root guide first, then load the relevant platform guides:

- [`ios/AGENTS.md`](../ios/AGENTS.md)
- [`android/AGENTS.md`](../android/AGENTS.md)
- [`core/AGENTS.md`](../core/AGENTS.md)
