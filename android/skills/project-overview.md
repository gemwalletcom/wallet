# Project Overview

Gem Wallet Android is a Kotlin and Jetpack Compose application backed by the shared Rust core.

## Technology Stack

- Kotlin
- Jetpack Compose
- Android SDK
- Gradle with Kotlin DSL
- Hilt for dependency injection
- Rust core via JNI
- Docker for reproducible/release build workflows

## Repository Structure

```
android/
├── app/             # Main application module
├── blockchain/      # Blockchain-specific logic
├── data/            # Repositories, services, Room storage
├── features/        # Feature modules
├── gemcore/         # JNI bindings to the Rust core
├── gemstone/        # Shared library integration
├── ui/              # Shared UI components and themes
├── flavors/         # Store and distribution variants
└── reproducible/    # Reproducible build tooling
```

## Module Responsibilities

- `app/` hosts the application entrypoints and Android integration
- `ui/` contains shared Compose UI, themes, and reusable screens
- `data/` contains repositories, storage, and service implementations
- `blockchain/` contains blockchain-specific implementations and models
- `features/` contains feature-focused modules
- `gemcore/` bridges Android and the Rust core through JNI

## Build Flavors

- `google`
- `huawei`
- `samsung`
- `solana`
- `universal`

## Key Dependencies

- Jetpack Compose
- Hilt
- Room
- Retrofit
- WalletConnect / Reown
- Rust core via JNI
- Wallet Core
- Gemstone
- Coil
- Ktor

## Architecture Patterns

- Use Jetpack Compose for UI
- Use MVVM-style state handling for screens and features
- Use Hilt for dependency injection
- Use repository and service layers for data access
- Keep shared blockchain and model behavior aligned with the Rust core
