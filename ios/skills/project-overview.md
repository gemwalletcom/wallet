# Project Overview

Gem Wallet iOS is a modular SwiftUI application backed by the shared Rust core.

## Structure

The app is organized into four main areas:

1. `Features/` — Independent UI feature modules
2. `Packages/` — Shared primitives, components, store, localization, and utilities
3. `Gem/` and `GemPriceWidget/` — App targets and platform integration
4. `core/` — Shared Rust blockchain engine exposed through FFI

Each feature package typically follows this structure:

```text
Features/[FeatureName]/
├── Package.swift
├── Sources/
│   ├── Scenes/
│   ├── ViewModels/
│   ├── Protocols/
│   ├── Types/
│   ├── Views/
│   └── Services/
├── Tests/
└── TestKit/
```

## Architecture

- Use MVVM with SwiftUI and `@Observable` view models
- Inject dependencies through the app resolver and environment values
- Keep services protocol-based for testability
- Treat each feature or package as an independent Swift Package Manager module when possible

## Layer Responsibilities

- `Features/` holds feature-specific UI, view models, services, tests, and test kits
- `Packages/` holds shared primitives, components, formatting, localization, storage, and service layers
- `Gem/` and widget targets handle app composition, navigation wiring, and platform integration
- `core/` remains the shared blockchain and cryptography engine consumed through generated bindings

## Navigation

- Navigation is tab-based with independent `NavigationPath` stacks per tab
- `NavigationStateManager` coordinates navigation state
- Deep links route through the centralized navigation layer
