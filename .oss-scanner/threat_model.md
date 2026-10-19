# Threat model: Gem Wallet

## What this project does and where untrusted input enters

Gem Wallet is a self-custodial multi-chain crypto wallet for iOS and Android. Both apps are thin UI over one Rust library, Gemstone (`core/gemstone`), exposed through UniFFI. Gemstone owns key storage, account derivation, transaction construction, signing, message signing, WalletConnect, payment links, swaps and the confirmation screen content. The same Cargo workspace also holds the backend (`core/apps/api`, `core/apps/daemon`) and an RPC proxy (`core/apps/dynode`).

The wallet's job is to sign exactly what the user approved, with keys nobody else can read. Treat all of the following as attacker-controlled:

- **dApps over WalletConnect**: session proposals, namespaces and chains, `eth_sendTransaction` / `eth_signTypedData` / `personal_sign`, Solana, Sui, TON, Tron and other chain-specific sign requests, SIWE / SIWS messages, and the peer metadata shown to the user (`core/crates/gem_wallet_connect`, `core/gemstone/src/wallet_connect`).
- **Links and QR codes**: deep links, payment URIs (BIP-21, EIP-681, Solana Pay, TON, WalletConnect Pay) and pasted addresses (`core/crates/payment`, `core/gemstone/src/deeplink.rs`, `core/gemstone/src/url_action.rs`, `core/gemstone/src/payment.rs`).
- **Swap and bridge provider APIs**: quotes, routers, spenders, approval targets, calldata, deposit addresses, Permit2 data and status responses (`core/crates/swapper`). A provider can be compromised or wrong; its response must never choose who gets spending authority.
- **Chain RPC nodes and indexers**: balances, nonces, fees, simulation and trace results, token metadata, contract code and transaction status. A node can lie.
- **Name resolution** (ENS, SNS and others, `core/crates/name_resolver`) and token or NFT metadata shown next to amounts and recipients.
- **Backend clients**: anything that reaches `core/apps/api` over HTTP or the WebSocket, signed with a device key it controls, and RPC requests to `core/apps/dynode`.
- **Files on disk the app reads back**: keystore v3 / v4 JSON files and legacy migration inputs.

The device itself (OS, secure enclave / Keystore, the user's lock screen) is trusted.

## Components that matter most / least

Most important, in this order:

1. **Key material**: `core/crates/gem_keystore` (v4 file format, v3 reader, Argon2id + AES-256-GCM), `core/crates/gem_derivation`, `core/crates/gem_crypto`, `core/crates/gem_hash`, `core/gemstone/src/keystore`, `core/gemstone/src/mnemonic.rs`. `docs/KEYSTORE_V4.md` is the contract. Invariants: the decrypted key never crosses the UniFFI boundary except through the explicit export calls, secrets are zeroized, entropy comes from the OS CSPRNG, every file path goes through a validated keystore id.
2. **Signing and transaction construction**: `core/gemstone/src/signer`, `core/crates/signer`, and the signer modules of every `core/crates/gem_*` chain crate (EVM, Bitcoin family, Solana, Sui, TON, Tron, Aptos, Cosmos, NEAR, XRP, Stellar, Algorand, Polkadot, Cardano, Hypercore, Tempo). What is signed must match the chain, asset, amount, recipient, fee, nonce, chain id and calldata that were confirmed.
3. **What the user is shown before signing**: `core/gemstone/src/services/confirm`, `core/gemstone/src/message` (EIP-712 and other typed messages), `core/gemstone/src/signer/decode.rs`, `core/crates/simulation`, `core/crates/security` (transaction scanning). A mismatch between the confirmation screen and the signed bytes is as serious as a signing bug.
4. **Swap authority**: `core/crates/swapper/src/approval`, `core/crates/swapper/src/permit2_data.rs`, and each provider's route, router and spender handling. Spenders and transaction targets must be authorized against a policy that does not come from the same provider response.
5. **External input parsers**: WalletConnect request decoding and validation, payment URI decoders, deep links, address parsing and checksum handling, chain RPC response mappers.
6. **Device and wallet authentication**: `core/crates/gem_auth` (shared by client and backend), `core/apps/api/src/auth`, and `docs/DEVICE.md`.
7. **Backend**: `core/apps/api` (HTTP + WebSocket, Diesel / Postgres in `core/crates/storage`), `core/apps/daemon` (indexers and push notifications driven by chain data), `core/apps/dynode` (RPC proxy that must only forward allowlisted methods and paths to configured upstreams).
8. **App code, source only**: `ios/Packages` (Swift) and `android/` (Kotlin), mainly secure storage, the lock and biometric gates, deep link routing and WalletConnect UI. These are not built in this image.

Out of scope: `core/bin/*` (developer tools), `scripts/`, `maestro/`, `localization/`, `docs/`, CI configuration, `core/Settings.yaml` defaults, `testkit` modules and anything under `tests/` or `testdata/` (except as fixtures to reproduce with), `ios/Submodules/*` (third-party forks with their own upstreams), and bugs wholly inside a dependency. Report a dependency bug only when Gem Wallet's use of it makes it reachable, and show the path.

## How to exercise it

The image has no network. Every crate is in the cargo cache, `CARGO_NET_OFFLINE` is set, and the debug workspace and its test binaries are built in `/src/core` with `--all-features`.

- **Tests**: from `/src/core`, run `cargo test --workspace --all-features --lib --bins -- <filter> --skip integration_tests`. Keep `--workspace --all-features` so cargo reuses the built artifacts; narrowing with `-p` changes the feature set and recompiles. Tests whose names contain `integration_tests` need the network or Postgres and will fail here.
- **Signer tests** live in `#[cfg(test)]` modules next to each chain's signer and in `core/gemstone/src`. Some are feature-gated (for example `gem_bitcoin`'s signer behind `signer`), which `--all-features` already enables.
- **Mocks**: providers and RPC clients have `testkit` mocks (`core/crates/*/src/testkit.rs`, `core/gemstone/src/testkit.rs`). A reproducer that feeds a hostile RPC, swap or WalletConnect response through a mock is the expected form.
- **Keystore**: `core/crates/gem_keystore/testdata` holds v3 and v4 fixtures; `core/crates/gem_keystore/README.md` documents the format.
- The backend apps build but cannot run end to end here (no Postgres, Redis or upstream nodes). Exercise their handlers, guards and parsers through unit tests.

## How you rate severity

A finding counts when one of the attackers above can cause it against a default install, with the user doing nothing more than what the flow asks of them (opening a link, connecting to a dApp, approving what the confirmation screen shows).

- **Critical**: recovery phrase, private key, keystore password or device key readable by an attacker, logged, persisted outside the keystore, or returned across the FFI boundary outside the explicit export flows. A keystore file accepted after tampering, or decrypted without the password. Predictable key or nonce generation. A signature over different bytes from those confirmed: another recipient, amount, asset, chain id, calldata, approval spender or Permit2 authority. Any remote input that makes the wallet sign or broadcast without a confirmation.
- **High**: a confirmation screen, simulation or security warning that a dApp, provider or RPC node can make lie about outflows, recipient, spender or chain, so that an approval the user reasonably accepts loses funds. Signatures replayable on another chain or contract. A swap provider able to pick an approval spender or transaction target that the independent policy does not authorize. Backend: acting as another device or wallet (reading its data, changing its subscriptions or push targets, claiming its rewards), or dynode forwarding to hosts or methods outside its allowlist. Memory corruption reachable from any untrusted input.
- **Medium**: a remote input that crashes the app (for example a panic that crosses UniFFI) or locks a user out of their funds until they intervene. Wrong but visible amounts, fees or decimals that do not by themselves move funds. A cheap unauthenticated denial of service against the backend. Linking a device to its wallet addresses for a third party.
- **Low**: anything that needs a rooted or jailbroken device, malware already on the device, physical access to an unlocked phone, or a malicious operator of the backend's own configuration. Hardening suggestions with no demonstrated attack.

## How to report

- Give a reproducer that runs in this image: a Rust test (preferably next to the code it targets, using the existing `testkit` mocks), the exact hostile input (WalletConnect payload, URI, RPC or provider JSON, keystore file), and what was signed, shown or leaked compared with what should have been.
- Name the chain and the flow (transfer, token transfer, swap, approval, stake, WalletConnect sign, message sign, payment link).
- Propose a minimal patch against `main` with a regression test. Tests use `.unwrap()`, not `.expect()`. Keep unrelated refactoring out of it.
- Report one root cause once. The same flaw on several EVM chains or several providers is one finding that lists each path.

## Anything to leave alone

- Signing arbitrary dApp payloads (EIP-712, Solana instructions, TON messages, raw calldata) is a feature. Report it only when the confirmation shows something different from what is signed, or a simulation or scan warning that should fire does not.
- Quote quality, price impact and slippage within the tolerance the user set are economics, not vulnerabilities.
- The v3 keystore reader accepts an empty scrypt salt and relies on the MAC (`docs/KEYSTORE_V4.md`, "Migration Semantics"); legacy WalletCore files need it.
- The Argon2id parameters are deliberately light: the keystore password is a 256-bit random device secret, so the KDF is defence in depth.
- iOS Keychain items survive app removal; Android removal wipes the password store. Both are documented platform behaviour.
- Screen-capture protection does not apply in the iOS simulator; only report it with a device.
- Missing rate limits, TLS or authentication on local development servers and `docker-compose.yml` services.
