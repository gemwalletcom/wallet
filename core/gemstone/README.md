# Gemstone

Gemstone is the essential cross platform library used by Gem Wallet clients (mainly iOS and Android).

## Build

iOS

```bash
just prepare-ios-package   # UniFFI Swift sources and headers
just build-ios-lib         # Gemstone static library for the iOS Rust targets
```

`just prepare-ios-package` generates the UniFFI Swift sources and copies them into `tests/ios/Packages/Gemstone`; `just build-ios-lib` builds the native static library for the iOS targets.

Android

```bash
just bindgen-kotlin && just build-android
```

See the [iOS sample](tests/ios) and the [main Android app's Gemstone module](../../android/gemstone) for client integration.
