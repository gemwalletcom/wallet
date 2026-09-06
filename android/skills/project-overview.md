# Project Overview

Use when you need the Android module layout, channel list, or where a new feature module belongs.
Gem Wallet Android is a Kotlin and Jetpack Compose application backed by the shared Rust core.

## Technology Stack

- Kotlin
- Jetpack Compose
- Android SDK
- Gradle with Kotlin DSL
- Hilt for dependency injection
- Rust core via JNI

## Repository Structure

```
android/
├── app/             # Main application module
├── data/            # Repositories, services, Room storage
├── features/        # Feature modules
├── gemcore/         # JNI bindings to the Rust core
├── gemstone/        # Shared library integration
├── ui/              # Shared UI components and themes
└── flavors/         # Store and distribution variants
```

## Module Responsibilities

- `app/` hosts the application entrypoints and Android integration
- `ui/` contains shared Compose UI, themes, and reusable screens
- `data/` contains repositories, storage, and service implementations
- `features/` contains feature-focused modules
- `gemcore/` bridges Android and the Rust core through JNI

## Build Flavors

Product flavors are distribution channels: `google` (default), `universal`, `huawei`, `solana`, `samsung`, `emerald`, and `fdroid`. All of them are declared in one matrix, `gradle/channels.gradle.kts`; select one with `-Pchannel=<name>`. See the decision record in `../docs/DECISIONS.md` before changing channel wiring.

## Key Dependencies

- Jetpack Compose
- Hilt
- Room
- Retrofit
- WalletConnect / Reown
- Rust core via JNI
- Gemstone
- Coil
- Ktor

## Architecture Patterns

- Use Jetpack Compose for UI
- Use MVVM-style state handling for screens and features
- Use Hilt for dependency injection
- Use repository and service layers for data access
- Keep shared blockchain and model behavior aligned with the Rust core
