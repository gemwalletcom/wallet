# Gemstone

Gemstone is the essential cross platform library used by Gem Wallet clients (mainly iOS and Android).

## Build

iOS

```bash
just bindgen-swift         # UniFFI Swift sources and headers in generated/swift
just build-ios-lib         # Gemstone static library for the iOS Rust targets
```

The iOS app builds both through `just generate-stone` from the repo root.

Android

```bash
just bindgen-kotlin && just build-android
```

See the [iOS app's Gemstone package](../../ios/Packages/Gemstone) and the [Android app's Gemstone module](../../android/gemstone) for client integration.
