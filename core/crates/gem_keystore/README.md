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
<base_dir>/v4/<keystore_id>.gemk
```

`keystore_id` is a lowercase UUID — deterministically derived from the wallet id via `KeystoreId::from_wallet_id` (UUID v5), so a wallet always maps to the same file and the id can be recomputed instead of persisted. Managed reads reject files where the header `keystore_id` does not match the filename.

All v4 files use this binary layout:

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 4 | Magic bytes: `GEMK` |
| 4 | 1 | Format version: `4` |
| 5 | 4 | Header length as big-endian `u32` |
| 9 | header length | Borsh-encoded header |
| 9 + header length | remaining bytes | AES-256-GCM ciphertext with 16-byte tag |

The header is plaintext and can be inspected without a password, but header metadata is trusted only after successful AES-GCM authentication. The header bytes are authenticated as AES-GCM AAD. The encrypted body is a Borsh-encoded secret payload.

## Header

The Borsh header contains:

| Field | Description |
| --- | --- |
| `keystore_id` | Lowercase UUID string (v5, derived from the wallet id) |
| `kind` | `Mnemonic` or `PrivateKey` |
| `created_at` | Unix timestamp in seconds |
| `kdf` | Argon2id parameters and random salt |
| `cipher` | AES-256-GCM nonce and tag length |

Current defaults:

| Parameter | Value |
| --- | ---: |
| Argon2id memory | 65,536 KiB |
| Argon2id iterations | 3 |
| Argon2id parallelism | 1 |
| Argon2id salt length | 16 bytes |
| Derived key length | 32 bytes |
| AES-GCM nonce length | 12 bytes |
| AES-GCM tag length | 16 bytes |

The implementation caps header size at 16 KiB, encrypted body size at 64 KiB, whole file size at 128 KiB, and password input at 1 MiB.

## Payload

The encrypted Borsh payload is one of:

| Kind | Payload |
| --- | --- |
| `Mnemonic` | Sanitized BIP-39 recovery phrase string |
| `PrivateKey` | Raw private-key bytes |

After decrypting, readers validate that the header `kind` matches the decrypted payload variant.

## Migration

Legacy v3 JSON is decoded only for migration. Importing v3 decrypts the legacy secret, validates the secret shape, and writes a new v4 `.gemk` file. The v3 result types are not part of the public keystore API.
