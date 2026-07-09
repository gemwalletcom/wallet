# Gem Wallet — Rust Core Security Audit

**Date:** 2026-07-09
**Target:** the Rust wallet engine in `core/` — keystore, key derivation, cryptographic primitives, transaction/message signing, and the UniFFI boundary to iOS/Android.
**Scope reference:** Rust Core Audit Scope (secret protection, migration binding, import/derivation correctness, signing-intent preservation, FFI safety).

## Audit objectives

| ID | Objective |
|----|-----------|
| **O1** | Wallet secrets remain protected (mnemonic, private keys, seeds, derived keys, passwords). |
| **O2** | Migrated secrets are bound to the intended wallet. |
| **O3** | Imported/derived keys produce the expected accounts. |
| **O4** | Signing preserves user-visible transaction/message intent (destination, amount, asset, chain/network id, memo, fee). |
| **O5** | Mobile-facing FFI does not expose secret material or an unsafe signing surface. |

**Out of scope** (not reviewed): business logic (pricing, NFTs, portfolios), API authentication, general RPC providers, UI flows, generated code, build infrastructure, and full protocol-depth chain correctness (focus is transaction-**intent** preservation).

## Method

Two waves of scoped auditor agents covered every in-scope subsystem plus all 15 chain signers dispatched by `GemChainSigner`. Each candidate finding was checked by **two independent adversarial verifiers** — one tracing the exact bytes hashed/signed, one assessing attacker model and exploitability. The keystore, cryptographic primitives, v3→v4 migration, derivation, and FFI boundary were additionally reviewed by hand, and the two dimensions whose agents failed mid-run (FFI re-run, Polkadot) were reviewed manually.

---

## Bottom line

**No exploitable vulnerability was found in the Rust core.** The keystore, cryptographic primitives, v3→v4 migration, key derivation, and the FFI boundary are solid and carefully built. Signing preserves user-visible intent — verified byte-exact across all 15 dispatched chains.

All findings below are **low/informational hardening**, a class of **self-inflicted memo-handling gaps** (a deposit can go uncredited, but no attacker benefits), or a **trust-boundary observation** whose exploitation lies outside the Rust-core scope.

---

## Findings

| # | Severity | Obj. | Location | Summary |
|---|----------|------|----------|---------|
| 1 | Low (confirmed) | O4 | `crates/gem_xrp/src/signer/chain_signer.rs:88` | Token-transfer non-numeric memo silently dropped from the signed payment |
| 2 | Low | O4 | `crates/gem_near/src/signer/models.rs:19` | Native NEAR transfer silently discards `input.memo` instead of rejecting |
| 3 | Info | O4 | `crates/gem_stellar/src/models/signing/transaction.rs:77` | Memo builder can only emit `MEMO_TEXT`; `MEMO_ID` unreachable |
| 4 | Info | O4 | `crates/gem_cosmos/src/signer/chain_signer.rs:42` | `sign_swap` ignores `SwapQuoteData.memo` (latent) |
| 5 | Low | O1 | `crates/signer/src/ed25519.rs:16` | Ed25519 secret copied into a non-zeroized stack array on every ed25519 sign |
| 6 | Low | O1 | `crates/gem_hypercore/src/signer/core_signer.rs:72` | Hyperliquid agent private key decoded into a non-zeroized `Vec<u8>` |
| 7 | Low | O4 | `crates/gem_hypercore/src/signer/core_signer.rs:85` | Non-spot swap fallback blind-signs an EIP-712 blob with the main key |
| 8 | Low | O2 | `gemstone/src/keystore/keystore.rs:197` | `single_solana` mnemonic migration fully skips the wallet-id binding check |
| 9 | Info | O5 | `gemstone/src/auth.rs:41` | `sign_auth` signs an opaque 32-byte hash with no domain separation |
| 10 | Info | O4 | `crates/gem_aptos`, `crates/gem_sui` swap/generic paths | Swap/Generic payloads signed without in-core reconstruction (not currently reachable) |

### 1. XRP token-transfer non-numeric memo silently dropped — Low (confirmed) — O4

`sign_token_transfer` routes the user memo through `token_memo()`, whose arm `Ok(0) | Err(_) => XrpPaymentMemo::None` collapses any memo that does not parse as a `u64` into `None`; `append_memo()` then omits it from the signed bytes. This is asymmetric with the native path (`payment_memo()`), which binds a non-numeric memo as `XrpPaymentMemo::Memo`. Both paths read the same user-typed string via `get_memo()`.

**Attack scenario (self-inflicted):** a user sends an issued currency (e.g. RLUSD) to a service whose deposit instructions require a free-text memo, types `ref-8842`, and sees it on the confirm screen. `token_memo("ref-8842") -> Err -> None`, so the signed Payment carries no `Memos` field; on-chain the deposit arrives without the memo and is not credited. Numeric destination tags — the common XRP case — are correctly bound, so real-world impact is limited to free-text memos on token transfers.

**Remediation:** mirror `payment_memo()` on the token path so a non-numeric memo is bound as `Memo`.

### 2. NEAR native transfer silently discards `input.memo` — Low — O4

`NearTransfer::from_input` never reads the generic `TransactionLoadInput.memo` field, and native NEAR transfers cannot carry a memo in-protocol. When a memo is present the safe behavior is to **reject**, not to build and sign a memo-less transfer.

**Attack scenario (self-inflicted):** a user sends native NEAR to a deposit address that requires a routing tag; if the UI shows the memo, the user believes it is included, but the signed `Transfer` action omits it → uncredited deposit.

**Remediation:** reject the transaction when `memo` is non-empty for chains/paths that cannot carry one.

### 3. Stellar memo builder can only emit `MEMO_TEXT`; `MEMO_ID` unreachable — Info — O4

The memo helper maps any non-empty `input.memo` to `Memo::Text`; `Memo::Id` is constructed only in tests (`#[cfg_attr(not(test), allow(unused))]`). The memo **is** faithfully bound into the signed envelope (there is no signing divergence), but a user pasting an exchange `MEMO_ID` value gets a text memo of the same digits.

**Attack scenario (self-inflicted):** deposit instructions specify `MEMO_ID: 1234567890`; the wallet signs a `MEMO_TEXT` "1234567890"; the exchange's memo-id matcher finds no match → not auto-credited.

**Remediation:** allow selecting `MEMO_ID` (numeric memo) for Stellar, or detect all-digit memos and encode as id.

### 4. Cosmos `sign_swap` ignores `SwapQuoteData.memo` — Info (latent) — O4

For swaps, the body memo is derived exclusively from `input.memo`; `swap_data.data.memo` is never read. The memo that *is* present is bound correctly. Not reachable today — the only Cosmos swap provider (Squid) sets `memo: None` — but a future memo-driven provider (THORChain/Maya) would broadcast with an empty memo and the swap could not be routed.

**Remediation:** prefer `swap_data.data.memo` when present, or assert it is `None` for the Cosmos swap path.

### 5. Ed25519 secret left in a non-zeroized stack array — Low — O1

`Ed25519KeyPair::from_private_key` copies the raw 32-byte secret into a plain `[u8; 32]` (`key_bytes`) that is never wiped; `SigningKey::from_bytes` then makes its own drop-zeroizing copy. This is the single ed25519 entrypoint for essentially every ed25519 chain sign and for ed25519 address derivation, so it runs constantly. It is inconsistent with the rest of the engine, which wraps secret bytes in `Zeroizing` throughout.

**Exploitability:** requires a separate memory-disclosure primitive (core/crash dump, swap-to-disk, uninitialized-stack read) to matter — a defense-in-depth gap, not a direct leak. The same Copy-array residue class exists in `gem_derivation` (`mnemonic/cardano.rs`, `mnemonic/crypto.rs`, `lib.rs::read_array`).

**Remediation:** wrap `key_bytes` in `Zeroizing` (or zeroize the local before return).

### 6. Hyperliquid agent private key in a non-zeroized `Vec<u8>` — Low — O1

`decode_hex(&hl_order.agent_private_key)` returns a plain `Vec<u8>` holding a secp256k1 agent key, passed by slice into signing and dropped without zeroization. The source `HyperliquidOrder.agent_private_key: String` also crosses the FFI boundary in plaintext and is never wiped. The main-key path is safer (wrapped in `Zeroizing` during signing).

**Remediation:** wrap the decoded agent key in `Zeroizing`; consider a secret string type for the FFI field.

### 7. HyperCore non-spot swap fallback blind-signs an EIP-712 blob with the main key — Low — O4

For any swap that is not HyperCore→HyperCore, `sign_swap_action` falls through to `sign_typed_action(&swap_data.data.data, private_key)` with the **main** wallet key, signing the entire provider-shaped typed data (domain + types + message) verbatim with no action-type allow-list and no destination/amount re-derivation.

**Exploitability:** *not currently reachable* — the sole caller's EIP-712 blob is constructed in-core (the HyperCore→HyperEVM bridge, hardcoded `HYPERCORE_SYSTEM_ADDRESS`), not fetched from a server. It is a latent sharp edge: if a future provider ever supplies that blob, a tampered `withdraw3`/`spotSend` could be signed by the main key with the user seeing only a bridge prompt.

**Remediation:** add an allow-list on `message.type` and reconstruct the typed data in-core from the approved `SignerInput`.

### 8. `single_solana` mnemonic migration skips the wallet-id binding check — Low — O2

`verify_migrated_secret` is the sole binding gate for v3→v4 migration: it re-derives the account from the decrypted secret and rejects the migration on `derived != expected`. For `wallet_type == Single && chain == Solana && kind == Mnemonic` it returns `Ok(())` on mismatch, **fully skipping** the address-equality check. This is intentional (commit `e8e6f0eac`) because legacy WalletCore single-Solana wallets derive a different address than `gem_derivation` (`m/44'/501'/0'/0'`). But it removes O2's independent safety-net for that case, silently violating the documented invariant in `KEYSTORE_V4.md`.

**Exploitability:** not remotely exploitable — the caller must already hold a v3 file that MAC-verifies with the supplied v3 password (a secret it already owns), and an existing v4 file is never overwritten with a different secret. The residual is a self-inflicted mis-binding that succeeds silently instead of failing loudly.

**Remediation:** re-derive the legacy WalletCore Solana derivation path and compare against **that**, rather than skipping the check entirely.

### 9. `sign_auth` signs an opaque 32-byte hash with no domain separation — Info — O5

`sign_auth_message_hash` validates only `hash.len() == 32`, then signs with `sign_ethereum_digest`, returning `r‖s‖v` — the same primitive/format as an EVM transaction or EIP-191/712 signature. It does not recompute the hash from the auth-message components or apply a domain-separation prefix. The intended `create_auth_message` path is preimage-resistant, so a malicious server nonce cannot steer the hash; and this is not an escalation over the already-exported `sign()`.

**Remediation:** have `sign_auth` accept the auth-message components and recompute the hash internally, or domain-separate the auth digest so it cannot coincide with a transaction/message digest.

### 10. Swap/Generic payloads signed without in-core reconstruction (Aptos, Sui) — Info — O4

Aptos `sign_swap` and Sui Swap/Generic paths sign payloads that are not reconstructed in the signer from independently-trusted intent fields.

**Exploitability — refuted as in-scope exploitable:** in the current code these payloads are either built in-core by the wallet's own swapper crate (Sui Cetus/Mayan reconstruct the PTB and bind sender/recipient; a dry-run is performed) or fetched over TLS from the first-party `api.gemwallet.com` proxy (Aptos Panora). Tampering requires compromising the first-party backend or a TLS-MITM — outside the Rust-core scope, and the identical blind-signing pattern exists on every chain's swap path (EVM/Solana included), so it is not a per-chain escalation.

**Remediation (defense-in-depth):** reconstruct swap transactions in-core from the approved quote where feasible; keep provider transport pinned.

### Cross-cutting theme — memo handling (findings 1–4)

Several chains that share the generic `memo` field either drop it (XRP token, NEAR), cannot represent the exchange-required form (Stellar `MEMO_ID`), or ignore a provider memo channel (Cosmos swap). None is attacker-exploitable, but collectively they are a real funds-safety class: a user's deposit can go uncredited. A unified "memo capability per chain, bind or explicitly reject" pass would close all four.

---

## Verified solid (coverage assurance)

**v4 keystore (`gem_keystore`)**
- AES-256-GCM with a per-write random 12-byte nonce and random 16-byte Argon2id salt.
- Full-header AAD via canonical re-serialization of parsed values — no field except the ciphertext can be altered without failing authentication; unknown JSON fields rejected (`deny_unknown_fields`).
- File/ciphertext size caps enforced before parse and before write; keystore ids are canonical UUID v4/v5 only (path-traversal-safe); the authenticated id must equal the filename id on managed reads.
- On AES-GCM tag failure the unauthenticated plaintext buffer is zeroized before returning.
- Atomic temp-file + `sync_all` + rename + directory sync; process-global lock serializes all file operations.

**v3 → v4 migration (`gem_keystore/v3`, `gemstone` migrate path)**
- v3 scrypt + AES-128-CTR with a Keccak256 MAC verified in **constant time before** the plaintext is trusted; scrypt params validated (n power-of-two, bounded).
- `migrate_v3` derives the keystore id deterministically from the wallet id; `verify_migrated_secret` re-derives and compares (except finding #8).
- On mismatch the staged v4 file is deleted and the v3 file preserved; idempotent retry authenticates an existing staged file with the new password and never lets a wrong password overwrite a real secret.

**Cryptographic primitives (`gem_crypto`, `gem_hash`)**
- PBKDF2-HMAC-SHA512 correct, including multi-block output; OS CSPRNG (`getrandom`) with no seeded fallback; `constant_time_eq` used for MAC comparison; BIP39 NFKD normalization with entropy held in `Zeroizing`.

**Derivation (`gem_derivation`)**
- Correct per-chain coin type / curve / derivation path; private-key import validates length/curve and rejects unsupported chains; wallet-id binding derives from the account address + wallet type.

**FFI boundary (`gemstone`)**
- The decrypted wallet key never crosses the boundary for routine signing (`GemKeystore.sign` / `sign_auth` return only signatures).
- The raw-key reveal `GemKeystore::private_key` is `#[cfg(any(test, debug_assertions))]` and release-excluded (no build profile re-enables `debug-assertions`).
- `GemChainSigner`, `MessageSigner.sign(private_key)`, and `sign_auth_message_hash` are not exported.
- `device.rs` exports only the separate low-value device-auth key (`Zeroizing`, redacted `Debug`); `decode_private_key` returns only caller-supplied bytes (no escalation); password `Vec<u8>` arguments are `Zeroizing`.

**Signing intent — verified byte-exact SAFE (all 15 dispatched chains)**

| Chain | Key property verified |
|-------|-----------------------|
| EVM | EIP-1559 fields + chain id bound; EIP-712 domain separator binds `verifyingContract`/`chainId` |
| Bitcoin family | Value conservation (inputs = outputs + fee), change-to-self, BIP143/segwit value binding, BCH `SIGHASH_FORKID`, Zcash consensus branch id |
| Solana | Instructions built in-core from `SignerInput`; destination/mint/amount bound |
| Tron | Signed txID (sha256 of raw_data protobuf) binds contract/to/amount/TRC20 call data |
| Cosmos | `SIGN_MODE_DIRECT` sign-doc binds `chain_id` + `account_number` + memo (replay-safe) |
| XRP | Destination tag + amount + account/sequence/fee bound into the STX-prefixed digest (see finding #1 for the token-memo edge) |
| Aptos | Domain-separated `APTOS::RawTransaction` vs `APTOS::Message`; BCS binds sender/args/chain id |
| Sui | In-core reconstruction for wallet-built transfers/stake; Intent-prefixed digest |
| Ton | Cell Merkle-hash binding; no message-vs-transaction confusion |
| NEAR | Canonical borsh; `receiver_id`/actions/`block_hash`/nonce bound (replay-safe) |
| Stellar | `network_id` + all operation fields bound; memo bound (see finding #3 for `MEMO_ID`) |
| Algorand | Closed `Operation` enum structurally eliminates `CloseRemainderTo`/rekey sweeps; genesis id/hash bound |
| Polkadot | Call (dest+amount) + era + nonce + genesis hash + spec/tx version bound; sender↔key checked; >256-byte payloads blake2b-hashed |
| Cardano | CBOR body value conservation; change-to-self; extended-key witness over the body hash |
| HyperCore | Withdrawal reconstructed in-core (amount + destination bound); see findings #6/#7 |

---

## Recommended priority

1. **Finding 1 (XRP token memo)** — smallest real correctness bug with a fund-safety consequence; mirror the native `payment_memo` path.
2. **Findings 1–4 (memo handling)** — unify: per-chain memo capability, bind or explicitly reject.
3. **Findings 5, 6 (zeroization)** — cheap defense-in-depth on hot signing paths.
4. **Findings 7, 10 (blind-sign fallbacks)** — action-type allow-list / in-core reconstruction (latent today).
5. **Finding 8 (Solana migration)** — re-derive the legacy WalletCore path instead of skipping the binding check.
6. **Finding 9 (`sign_auth`)** — domain-separate the auth digest.

---

*This report reviews transaction-intent preservation and secret handling in the Rust core; it is not a full protocol-depth audit of each chain. Exploitation of finding #10 depends on trust boundaries (first-party backend, transport security) outside the Rust-core scope.*
