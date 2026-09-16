# Project Overview

Use when you need the iOS module layout, navigation model, or where a new feature package belongs.
Gem Wallet iOS is a modular SwiftUI application backed by the shared Rust core.

## Structure

The app is organized into four main areas:

1. `Features/` — Independent UI feature modules
2. `Packages/` — Shared primitives, components, store, localization, and utilities
3. `Gem/` and `GemPriceWidget/` — App targets and platform integration
4. `../core/` — Shared Rust source at the monorepo root, exposed through FFI

Feature packages use the directories they need; this layout is not a scaffolding checklist:

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
- Construct screen dependencies in the factory and follow [Architecture § 8](../../docs/ARCHITECTURE.md#8-services-are-injected-never-constructed-at-a-call-site) for environment scope
- Use generated Core service protocols; do not add forwarding app services or protocols solely for testability
- Treat each feature or package as an independent Swift Package Manager module when possible

## Layer Responsibilities

- `Features/` holds feature-specific UI, view models, services, tests, and test kits
- `Packages/` holds shared primitives, components, formatting, localization, storage, and service layers
- `Gem/` and widget targets handle app composition, navigation wiring, and platform integration
- `../core/` owns shared product rules and blockchain behavior; apps consume its generated bindings

## Navigation

- Navigation is tab-based with independent `NavigationPath` stacks per tab
- `NavigationStateManager` coordinates navigation state
- Deep links route through the centralized navigation layer
