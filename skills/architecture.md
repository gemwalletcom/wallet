# Architecture

Gem Wallet uses native UI layers on top of a shared blockchain engine.

| Layer | iOS | Android | Shared |
|-------|-----|---------|--------|
| UI | SwiftUI + MVVM | Jetpack Compose + MVVM | — |
| DI | Environment injection | Hilt | — |
| Data | Store (SQLite) + Services | Room + Repositories | — |
| Blockchain | Gemstone (FFI) | Gemstone (JNI) | Rust `core/` |

Changes in shared domain models or blockchain flows often need coordinated updates across both apps.
