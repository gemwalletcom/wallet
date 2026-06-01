# Device Authentication

## Overview

All `/v2/devices/*` endpoints require Ed25519 request signing. New clients should use the Gem `Authorization` header. Individual `x-device-*` headers remain supported for existing clients and should be treated as legacy compatibility.

## Gem Authorization Header

```
Authorization: Gem base64(<device_id_hex>.<timestamp_ms>.<wallet_id>.<body_hash_hex>.<signature_hex>)
```

The decoded payload is always 5 dot-separated parts:
- `device_id_hex` - 64-character hex Ed25519 public key
- `timestamp_ms` - Unix timestamp in milliseconds
- `wallet_id` - wallet identifier, or an empty string for non-wallet endpoints
- `body_hash_hex` - 64-character hex SHA256 hash of the request body
- `signature_hex` - 128-character hex Ed25519 signature

When `wallet_id` is empty, the payload contains `..` between timestamp and body hash.

**Signed message:**

```
{timestamp}.{method}.{path}.{walletId}.{bodyHash}
```

Examples:

```
1706000000000.GET./v2/devices..e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
1706000000000.GET./v2/devices/assets.multicoin_0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

## Legacy Individual Headers

The server still accepts individual headers for compatibility:

- `x-device-id`: 64-character hex Ed25519 public key
- `x-device-signature`: Ed25519 signature, hex or base64
- `x-device-timestamp`: Unix timestamp in milliseconds
- `x-device-body-hash`: 64-character hex SHA256 hash of the request body
- `x-wallet-id`: wallet identifier for wallet-scoped endpoints

**Signed message:**

```
v1.{timestamp}.{method}.{path}.{bodyHash}
```

The legacy signed message does not include `walletId`; new clients should not add new dependencies on this format.

## Request Examples

### Gem Wallet-scoped Endpoint

```http
GET /v2/devices/assets?from_timestamp=1234567890
Authorization: Gem base64(abc123...def456.1706000000000.multicoin_0x742d...f0bEb.e3b0c44...b855.aabb11...)
```

### Gem Non-wallet Endpoint

```http
GET /v2/devices
Authorization: Gem base64(abc123...def456.1706000000000..e3b0c44...b855.aabb11...)
```

### Legacy Individual Headers

```http
GET /v2/devices/assets?from_timestamp=1234567890
x-device-id: abc123...def456
x-wallet-id: multicoin_0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
x-device-signature: aabb11...
x-device-timestamp: 1706000000000
x-device-body-hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

## Implementation

- Request signature verification: [`apps/api/src/devices/signature.rs`](../apps/api/src/devices/signature.rs)
- Cryptographic verification: [`crates/gem_auth/src/device_signature.rs`](../crates/gem_auth/src/device_signature.rs)
- Request guards: [`apps/api/src/devices/guard/`](../apps/api/src/devices/guard/)
- Error handling: [`apps/api/src/devices/error.rs`](../apps/api/src/devices/error.rs)
