# Gem Wallet

[![Core CI](https://github.com/gemwalletcom/wallet/actions/workflows/core-ci.yml/badge.svg)](https://github.com/gemwalletcom/wallet/actions/workflows/core-ci.yml)
[![iOS CI](https://github.com/gemwalletcom/wallet/actions/workflows/ios-ci.yml/badge.svg)](https://github.com/gemwalletcom/wallet/actions/workflows/ios-ci.yml)
[![Android CI](https://github.com/gemwalletcom/wallet/actions/workflows/android-ci.yml/badge.svg)](https://github.com/gemwalletcom/wallet/actions/workflows/android-ci.yml)
[![License](https://badgen.net/github/license/gemwalletcom/wallet)](https://github.com/gemwalletcom/wallet/blob/main/LICENSE)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/gemwalletcom/wallet)
[![Gem Wallet Discord](https://img.shields.io/discord/974531300394434630?style=plastic)](https://discord.gg/aWkq5sj7SY)
[![X (formerly Twitter) Follow](https://img.shields.io/twitter/follow/GemWallet)](https://x.com/GemWallet)

Gem Wallet is an open-source mobile wallet for iOS and Android. This repository is the monorepo for both apps and the shared Rust core they build against.

## Install

- [App Store](https://apps.apple.com/app/apple-store/id6448712670?ct=github&mt=8)
- [Google Play](https://play.google.com/store/apps/details?id=com.gemwallet.android&utm_campaign=github&utm_source=referral&utm_medium=github)
- [F-Droid](https://f-droid.org/en/packages/com.gemwallet.android/)
- [Android APK releases](https://github.com/gemwalletcom/wallet/releases/latest)

## Features

- Open source, self-custodial wallet with multi-chain support
- Native iOS and Android apps with shared Rust-based blockchain functionality
- Swaps, staking, WalletConnect, fiat on/off ramp, alerts, and market data

## Repository

- `ios/`: SwiftUI application, packages, tests, and iOS-only submodules
- `android/`: Kotlin/Compose application and Android build tooling
- `core/`: shared Rust source used by both mobile apps

## Getting Started

1. Clone the repository with iOS submodules:

```bash
git clone --recursive https://github.com/gemwalletcom/wallet.git
cd wallet
```

2. If needed, initialize iOS submodules later:

```bash
just setup-git
```

### iOS

> [!NOTE]
> iOS builds require macOS. Apple silicon is the default supported environment for Gemstone builds.

```bash
cd ios
just bootstrap
just spm-resolve
just build-for-testing
```

`just bootstrap` also creates the local Gemstone UniFFI Swift/header sources that SwiftPM needs for package resolution. Xcode builds the Gemstone Rust static library automatically after that. Intel Macs are not supported for iOS Gemstone builds.

### Android

```bash
cd android
just bootstrap
just build-test
```

## Developer Shortcuts

The repo root exposes monorepo commands plus module access to each platform:

```bash
just build
just generate
just localize
just bump patch
just bump minor
just bump major
just ios bootstrap
just ios build
just ios build-for-testing
just ios test-without-building
just android bootstrap
just android build
just android build-test
just android test
```

Platform-specific commands remain available through the [`ios`](ios/justfile) and [`android`](android/justfile) just modules.

## Security

Gem Wallet is self-custodial, and keeping user funds safe is our highest priority. See the [Security Overview](https://gemwallet.com/security/) for our practices around key material, signing, and secure storage.

- [Responsible Disclosure](https://gemwallet.com/security/bug-bounty/) — report security vulnerabilities responsibly
- [CertiK Security Audit (April 2026)](https://static.gemwallet.com/audits/Gem-Wallet-CertiK-Security-Audit-April-2026.pdf) — latest third-party audit report

## Contributing

- Browse [GitHub Issues](https://github.com/gemwalletcom/wallet/issues)
- Follow the public [Roadmap](https://github.com/orgs/gemwalletcom/projects/4)

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## Community

- Website: [gemwallet.com](https://gemwallet.com)
- Chat: [Discord](https://discord.gg/aWkq5sj7SY), [Telegram](https://t.me/GemWallet)
- Updates: [X](https://x.com/GemWallet)

## License

Gem Wallet is open-sourced software licensed under the [GPL-3.0](LICENSE).
