# Gem Keystore v4 Implementation Source of Truth

Last checked against implementation: 2026-06-08.

This is the short as-built reference for Gem Keystore v4. The long design doc keeps history and rationale; this file records the current implementation contract. The code remains canonical.

## One-Line Model

Gem Keystore v4 stores one encrypted secret file per controlled wallet. Wallet/account metadata stays in the mobile databases. The keystore file id is deterministic from the wallet id.

## Core Ownership

- `gem_keystore`: BIP-39 helpers, v4 encrypted file format, v3 WalletCore reader, raw secret storage.
- `gem_derivation`: wallet id derivation, account derivation, private-key import validation, chain address creation.
- `gem_auth`: shared device-auth header format (Ed25519 build + verify), used by both the client and the backend.
- `gemstone`: UniFFI boundary over `gem_keystore` and `gem_derivation`, plus keystore-internal signing (`GemKeystore.sign`/`sign_auth`, `MessageSigner.sign_with_keystore`) routed over the per-chain `gem_*` signer crates, and the client device-auth wrappers.
- Mobile apps own wallet name, order, current wallet, account rows, duplicate checks, subscriptions, UI, and secure password storage.

`gem_keystore` must stay secret-storage-only. It must not depend on app primitives, chain crates, signer, or account derivation. The signing dispatch lives in `gemstone`, not in `gem_keystore`.

## Keystore ID

- The v4 `keystore_id` is `keystore_id_for_wallet(wallet.id)`.
- It is a deterministic UUID v5 from the wallet id, using the Rust `KeystoreId::from_wallet_id` namespace.
- It is a filename and authenticated header field only. It is not secret material and is not KDF input.
- Public keystore id inputs accept only canonical lowercase UUID v4 or v5.
- Real v4 wallet files should use v5. UUID v4 is accepted for legacy tests/temp/direct-id cases.
- Never infer migration state from "is this string a UUID". WalletCore v3 ids can also be UUIDs.

File path:

```text
<base_dir>/v4/<keystore_id>.gemk
```

## v4 File Format

Binary layout:

```text
GEMK | version=4 | header_len_be_u32 | borsh_header | aes_256_gcm_ciphertext_and_tag
```

Header fields:

- `keystore_id`
- `kind`: `Mnemonic` or `PrivateKey`
- Argon2id KDF params and random salt
- AES-256-GCM params and random nonce

Security rules:

- Header is plaintext but is authenticated as AES-GCM AAD.
- Managed reads require authenticated header `keystore_id` to match the filename id.
- Header metadata from `list`, `get_meta`, or `inspect` is not proof of migration success.
- Passworded `verify` or a real decrypt is required before trusting a file for migration state.
- Reads cap file/header/body sizes before parsing.
- Unknown versions, invalid ids, invalid KDF/cipher params, malformed files, and auth failures return typed errors.

Current crypto:

- Argon2id, 64 MiB, 3 iterations, parallelism 1.
- Random 16-byte salt per encryption.
- AES-256-GCM with random 12-byte nonce and 16-byte tag.
- Temp-file write, fsync, atomic rename, directory sync.
- Secret files are created with owner read/write permissions.

## Concurrency

`gem_keystore` serializes every file operation (create, read, delete, migrate) through a process-global lock (`OnceLock<Mutex<()>>` in `storage/queue.rs`); each `FileKeystore` operation takes the lock before touching disk.

- The guarantee holds regardless of how callers construct keystores. Android operators open a fresh `GemKeystore(baseDir)` per call and iOS shares one instance; both serialize through the same lock because it is static, not per-instance.
- Concurrent create/read/delete on one wallet — across threads and across separate keystore instances through the UniFFI binding — cannot corrupt the file or produce duplicates, and create stays idempotent (deterministic id, atomic rename).
- A poisoned lock surfaces as a typed `KeystoreError`, not a panic across the FFI boundary.
- Coverage: Android instrumented `GemKeystoreConcurrencyTest` races 8 threads through create/read/delete on one wallet from independent `GemKeystore` instances.

## Secret Payload

The encrypted payload is exactly one of:

- sanitized BIP-39 mnemonic phrase
- raw private-key bytes

v4 does not store wallet names, app wallet ids, account lists, addresses, public keys, derivation paths, xpubs, or WalletCore `activeAccounts`.

## Recovery Phrase Generation

New mnemonics are generated in `gem_keystore` (`Mnemonic::generate`):

- Entropy comes from `gem_crypto::random::bytes`, which calls `getrandom::fill` — reading the OS CSPRNG directly (`getrandom(2)` / `getentropy` / `BCryptGenRandom`), not a userspace or seeded PRNG.
- 32 random bytes are drawn and truncated to the BIP-39 entropy length for the requested word count (16 bytes / 12 words … 32 bytes / 24 words), then encoded as an English BIP-39 phrase.
- Entropy is held in `Zeroizing` and wiped once the phrase is built.
- Imported phrases are NFKD-normalized, lowercased, and validated against the BIP-39 English wordlist before use.

## Password Boundary

iOS:

- Password is stored as a lowercase hex string in Keychain.
- WalletCore v3 migration passes UTF-8 bytes of that hex string.
- v4 APIs pass decoded raw bytes.

Android:

- Password is stored per wallet id as a hex string.
- WalletCore v3 migration passes decoded raw bytes.
- v4 APIs pass decoded raw bytes.

Empty v4 passwords are rejected. v3 empty passwords are accepted only for legacy compatibility.

## Keystore-Internal Signing

Routine signing runs inside Rust. The decrypted key never crosses the UniFFI/JNI boundary. The app passes the keystore id, chain, prepared input, and password bytes, and receives only signatures.

- `GemKeystore.sign(keystore_id, chain, input, password) -> [signature]`: loads the key internally, routes by transaction input type (transfer, token transfer, swap, stake, token approval, perpetual, account action, data, etc.), signs, and returns signatures. Multi-signature chains return more than one.
- `GemKeystore.sign_auth(keystore_id, chain, hash, password) -> signature`: signs a device / WalletConnect auth hash.
- `MessageSigner.sign_with_keystore(keystore, keystore_id, password) -> signature`: typed/personal message signing, selected by `signType` (EIP-191, EIP-712, SIWE, Sui/Ton/Tron personal, base58).

Boundaries:

- Raw-key signers are not on the UniFFI surface: `GemChainSigner`, `MessageSigner.sign(private_key)`, and `sign_auth_message_hash` are internal Rust only (used by `GemKeystore` and tests).
- `private_key`, `export_private_key`, and `export_recovery_phrase` remain for explicit reveal/backup and v3 migration only, never for routine signing.
- The signing router (`GemChainSigner`) lives in `gemstone`, over the per-chain `gem_*` signer crates. `gem_keystore` stays storage-only.
- App-side password bytes are zeroized after each call (Android `withGemKeystore`, iOS `withV4Password`).

App entrypoints:

- iOS: `keystore.sign(wallet:input:)`, `keystore.signMessage(signer:wallet:)`, `keystore.signAuthMessageHash(wallet:chain:hash:)`. The old `getPrivateKey` / `ChainSigner` / `SwapSigner` paths are removed.
- Android: `GemSignTransactionOperator`, `GemSignMessageOperator`, `GemSignAuthOperator`, all via `withGemKeystore`. The old `SignClient` / `SignService` paths are removed.

## Device Authentication

The `Gem <base64>` `Authorization` header format lives in the shared `gem_auth` crate, used by both the client and the backend — see [DEVICE_AUTHENTICATION.md](DEVICE_AUTHENTICATION.md) for the wire format.

Keystore side:

- `gemstone` exposes the client wrappers over UniFFI (over `gem_auth`): `generate_device_key_pair`, `device_public_key`, `sign_device_auth`.
- The device key pair is generated in Rust so both apps stay consistent. The private key is a 32-byte Ed25519 seed drawn from `getrandom` (the OS CSPRNG, the same source as recovery-phrase entropy), held in the app's secure storage; the signature is produced by Rust.

## iOS App Contract

Source paths:

- `ios/Packages/Keystore/Sources/Extensions/Wallet+Keystore.swift`
- `ios/Packages/Keystore/Sources/LocalKeystore.swift`
- `ios/Packages/FeatureServices/WalletService/WalletService.swift`
- `ios/Packages/GemstonePrimitives/Sources/Extensions/GemKeystore+GemstonePrimitives.swift`

Rules:

- `Wallet.keystoreId` is always computed from `wallet.id.id`.
- v4 wallets should keep `Wallet.externalId == nil`.
- `Wallet.externalId` is legacy-only and exposed as `legacyV3Id = externalId ?? id.id`.
- iOS v3 migration locates the WalletCore file using `legacyV3Id`.
- iOS writes v4 at `Wallet.keystoreId`.
- iOS does not update the wallet DB after v3 migration.
- After v4 write succeeds, iOS leaves the v3 file in place; the migration marker is the v4 file's presence (see Migration Semantics).
- If migration fails before v4 write, the v3 file remains and startup retries later.
- If the app crashes after v4 write, future reads still work because the v4 id is deterministic.
- Startup logs per-wallet migration failures instead of swallowing them silently.
- Deleting a wallet removes every copy: the v4 file and the in-place v3 file.

Important test coverage:

- v3 file keyed by `wallet.id.id`
- v3 file keyed by real legacy WalletCore `externalId` UUID
- migration idempotency
- v4 read after migration without DB `externalId` update

## Android App Contract

Source paths:

- `android/gemcore/src/main/kotlin/com/gemwallet/android/ext/Wallet.kt`
- `android/app/src/main/kotlin/com/gemwallet/android/services/MigrateV3KeystoreService.kt`
- `android/app/src/main/kotlin/com/gemwallet/android/services/CheckAccountsService.kt`

Rules:

- Android computes `Wallet.keystoreId` from `wallet.id.id` on demand.
- Android does not persist an `external_id` for v4 lookup.
- Android v3 files are located at `<dataDir>/<wallet.id.id>`.
- Android writes v4 at the deterministic id and leaves the v3 file in place; later launches skip migration once `v4/<keystore_id>.gemk` exists.
- There is no v3-to-v4 DB update.
- Migration failures are logged and retried next launch while the v3 file remains.
- Account backfill uses Rust `add_accounts`; it should not export mnemonic/private key into Kotlin.
- Deleting a wallet removes every copy (the v4 file and the in-place v3 file); the stored password is cleared only when both are gone.

## Migration Semantics

- Normal v4 decrypt APIs do not silently upgrade or rewrite v3 files.
- v3 support exists only for explicit migration.
- `migrate_v3` requires the app-provided deterministic `keystore_id`.
- If the target v4 file already exists, Rust authenticates it with the supplied new password and returns metadata.
- Idempotent import verifies id and password only; it does not compare the incoming v3 payload with the existing v4 payload. This relies on wallet-id derivation and duplicate detection being correct.
- Wrong password never overwrites an existing v4 file.
- The migration marker is the presence of the v4 file, not the absence of the v3 file. Both apps skip migration once `v4/<keystore_id>.gemk` exists, so Argon2id is never re-run.
- The v3 file is left at its original path after a successful v4 write (downgrade safety): an older or rolled-back build still finds the wallet where it expects it. The two encrypted copies (WalletCore v3 + v4) coexist during the rollout window.
- v3 cleanup is deferred to a later release, after v4 is proven and a downgrade/rollback to a pre-v4 build is no longer supported.
- Deleting a wallet is app-owned and must remove every on-disk copy (the v4 file and any v3 file) so no secret is orphaned; the password is cleared only after all copies are gone.

## Mobile Import/Create Flow

- Preview/import planning derives wallet id and accounts without writing a keystore file.
- Create/import writes v4 under `keystore_id_for_wallet(wallet_id)`.
- The app inserts wallet/account metadata into its DB.
- New v4 wallets never write WalletCore v3 files.
- iOS maps new v4 wallets with `externalId: nil`.
- Android stores no external id for v4 lookup.

## Security Invariants

- Do not log, print, persist, snapshot, or expose mnemonic/private-key material.
- Routine maintenance paths must call Rust APIs returning non-secret outputs.
- Routine signing runs inside Rust (`GemKeystore.sign`/`sign_auth`, `MessageSigner.sign_with_keystore`); the decrypted key does not cross the FFI boundary, and raw-key signers are not exported.
- Mnemonic/private-key export is only for explicit backup/recovery UI.
- Deleting a wallet must remove every on-disk secret copy (v4 and any v3); clear the stored password only after all copies are gone.
- Header inspection is diagnostic only and never migration proof.
- All path construction must go through validated keystore ids.
- Keep v3 parsing capped and typed-error based.
- Keep WalletCore references out of production keystore flows except explicit legacy migration support during rollout.

## Misc

### v4 Header Dump Script

Purpose:

- Dumps plaintext v4 `.gemk` headers to JSON without requiring a Rust build.
- Never decrypts the body and never asks for a password.
- Reports encrypted body length, ciphertext length, and tag length only.
- Flags whether the filename stem matches the plaintext header `keystore_id`.
- Supports either one `.gemk` file or a directory of `.gemk` files.

Usage:

```sh
python3 core/scripts/dump_v4_keystore.py <file.gemk>
python3 core/scripts/dump_v4_keystore.py <dir>/v4/
python3 core/scripts/dump_v4_keystore.py <dir>/v4/ -o dump.json
```

The script lives at [`../scripts/dump_v4_keystore.py`](../scripts/dump_v4_keystore.py).

Verified behavior:

- Parsed a real v4 file generated by `gem_keystore::FileKeystore`.
- Correctly decoded `GEMK`, version `4`, Borsh header fields, deterministic v5 `keystore_id`, `Mnemonic` kind, Argon2id params, AES-256-GCM params, body lengths, and filename/header match.
- Directory mode and `-o` JSON output work.
- Malformed files are reported as per-file JSON errors instead of aborting a batch.

Caveats:

- Header output is unauthenticated diagnostic metadata. Do not use this script to decide migration success, wallet ownership, or secret validity.
- A passworded `verify` or decrypt path in Rust is required before trusting header metadata.
- The script exits `0` even when individual files produce JSON `error` entries; inspect output if using it in automation.

## Known Follow-Ups

- Migration failures are logged and retried on next launch; decide whether they also need durable telemetry or user-visible recovery.
- v3 files are left in place after migration for downgrade safety, so the secret exists as two encrypted copies. Schedule a cleanup release that deletes the v3 file once a downgrade/rollback to a pre-v4 build is no longer supported.
