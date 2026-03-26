# Gem Wallet

[![iOS CI](https://github.com/gemwalletcom/wallet/actions/workflows/ios-ci.yml/badge.svg)](https://github.com/gemwalletcom/wallet/actions/workflows/ios-ci.yml)
[![Android CI](https://github.com/gemwalletcom/wallet/actions/workflows/android-ci.yml/badge.svg)](https://github.com/gemwalletcom/wallet/actions/workflows/android-ci.yml)
[![License](https://badgen.net/github/license/gemwalletcom/wallet)](https://github.com/gemwalletcom/wallet/blob/main/LICENSE)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/gemwalletcom/wallet)
[![Gem Wallet Discord](https://img.shields.io/discord/974531300394434630?style=plastic)](https://discord.gg/aWkq5sj7SY)
[![X (formerly Twitter) Follow](https://img.shields.io/twitter/follow/GemWallet)](https://x.com/GemWallet)

Gem Wallet is an open-source mobile wallet for iOS and Android. This repository is the monorepo for both apps and the shared Rust core submodule they build against.

- `ios/`: SwiftUI application, packages, tests, and iOS-only submodules
- `android/`: Kotlin/Compose application and Android build tooling
- `core/`: shared Rust submodule used by both mobile apps

📲 [iOS on the App Store](https://apps.apple.com/app/apple-store/id6448712670?ct=github&mt=8)

🤖 [Android on Google Play](https://play.google.com/store/apps/details?id=com.gemwallet.android&utm_campaign=github&utm_source=referral&utm_medium=github)

## Features

- Open source, self-custodial wallet with multi-chain support
- Native iOS and Android apps with shared Rust-based blockchain functionality
- Swaps, staking, WalletConnect, fiat on/off ramp, alerts, and market data

## Getting Started

1. Clone the repository with submodules:

```bash
git clone --recursive https://github.com/gemwalletcom/wallet.git
cd wallet
```

2. If needed, initialize submodules later:

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

If you are using an Intel Mac, update `core` and run `just generate-stone` to build the additional `x86_64` Gemstone artifacts.

### Android

```bash
cd android
just bootstrap
just build-test
```

Add GitHub Packages credentials to `android/local.properties` before Android builds that need private package access:

```properties
gpr.username=<your-github-username>
gpr.token=<your-github-token>
```

## Developer Shortcuts

The repo root exposes monorepo commands plus module access to each platform:

```bash
just build
just generate
just localize
just bump patch
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

## Contributing

- Browse [GitHub Issues](https://github.com/gemwalletcom/wallet/issues)
- Track work on the [GitHub Project Board](https://github.com/orgs/gemwalletcom/projects/2)
- See the public [Roadmap](https://github.com/orgs/gemwalletcom/projects/4)

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines and [android/reproducible/README.md](android/reproducible/README.md) for Android release verification details.

## Community

- Install the app at [gemwallet.com](https://gemwallet.com) or Join [Discord](https://discord.gg/aWkq5sj7SY), [X](https://twitter.com/GemWallet), [Telegram](https://t.me/GemWallet)

## License

Gem Wallet is open-sourced software licensed under the [GPL-3.0](LICENSE).
