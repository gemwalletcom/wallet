# gem_keystore

`gem_keystore` owns recovery-phrase helpers, encrypted file storage, and legacy v3 import support.

## Features

| Feature | Purpose |
| --- | --- |
| `mnemonic` | BIP-39 generation, sanitizing, entropy, and seed helpers |
| `storage` | File-backed encrypted keystore |
| `v3` | v3 JSON import for migration |

## File Format

Keystore files are stored under:

```text
<base_dir>/<keystore_id>.json
```

`keystore_id` is a lowercase UUID — deterministically derived from the wallet id via `KeystoreId::from_wallet_id` (UUID v5), so a wallet always maps to the same file and the id can be recomputed instead of persisted. Managed reads reject files where the header `keystore_id` does not match the filename.

All v4 files are JSON:

```json
{
  "version": 4,
  "id": "70b3b599-4bf1-4f7a-9ad4-a7746cc38ab3",
  "kind": "mnemonic",
  "crypto": {
    "kdf": {
      "algorithm": "argon2id",
      "memory_kib": 19456,
      "iterations": 2,
      "parallelism": 1,
      "salt": "<16 bytes hex>",
      "output_len": 32
    },
    "cipher": {
      "algorithm": "aes-256-gcm",
      "nonce": "<12 bytes hex>",
      "tag_len": 16
    },
    "ciphertext": "<hex, AES-256-GCM ciphertext with the 16-byte tag appended>"
  }
}
```

The metadata is plaintext and can be inspected without a password, but it is trusted only after successful AES-GCM authentication: every field except `crypto.ciphertext` is bound as AES-GCM AAD via a canonical re-serialization of the parsed values, so reformatting the JSON does not break authentication while changing any value does.

## Fields

| Field | Description |
| --- | --- |
| `id` | Lowercase UUID string (v5, derived from the wallet id) |
| `kind` | `mnemonic` or `private_key` |
| `crypto.kdf` | Argon2id parameters and random salt |
| `crypto.cipher` | AES-256-GCM nonce and tag length |
| `crypto.ciphertext` | Hex ciphertext with the GCM tag appended |

Current defaults:

| Parameter | Value |
| --- | ---: |
| Argon2id memory | 19,456 KiB |
| Argon2id iterations | 2 |
| Argon2id parallelism | 1 |
| Argon2id salt length | 16 bytes |
| Derived key length | 32 bytes |
| AES-GCM nonce length | 12 bytes |
| AES-GCM tag length | 16 bytes |

The implementation caps the ciphertext at 64 KiB, the whole file at 128 KiB, and password input at 1 MiB.

## Payload

The plaintext inside the ciphertext is the raw secret bytes, interpreted by the authenticated `kind`:

| Kind | Payload |
| --- | --- |
| `mnemonic` | Sanitized BIP-39 recovery phrase, UTF-8 |
| `private_key` | Raw private-key bytes |

## Migration

Legacy v3 JSON is decoded only for migration. Importing v3 decrypts the legacy secret, validates the secret shape, and writes a new v4 file. The v3 result types are not part of the public keystore API.

## Benchmarks

```sh
cargo bench -p gem_keystore --features storage
```

Measures encrypt (import) and decrypt through the public API with the production KDF defaults; Argon2id dominates both. Debug builds are several times slower than release — benchmark numbers are only meaningful from `cargo bench` (release profile).
