# Device WebSocket Streaming

## Overview

Authenticated device WebSocket carrying price, balance, transaction, price-alert, NFT, perpetual, in-app-notification, fiat-transaction, and support updates.

## Endpoint

```
wss://api.gemwallet.com/v2/devices/stream
```

## Authentication

Uses the same device authentication as all `/v2/devices/*` endpoints.

**For complete authentication details, see:** [Device Authentication](DEVICE_AUTHENTICATION.md)

## Protocol

### Client → Server Messages

The active price controls are `getPrices`, `subscribePrices`, `addPrices`, and `unsubscribePrices`.

**Subscribe to Prices:**
```json
{
  "type": "subscribePrices",
  "data": {
    "assets": ["bitcoin", "ethereum"]
  }
}
```

**Get Current Prices Once:**
```json
{
  "type": "getPrices",
  "data": {
    "assets": ["bitcoin", "ethereum"]
  }
}
```

**Add More Assets:**
```json
{
  "type": "addPrices",
  "data": {
    "assets": ["solana"]
  }
}
```

**Unsubscribe from Prices:**
```json
{
  "type": "unsubscribePrices",
  "data": {
    "assets": ["bitcoin"]
  }
}
```

`subscribeRealtimePrices` and `unsubscribeRealtimePrices` remain accepted wire variants for compatibility, but the server currently treats them as no-ops.

### Server → Client Messages

**Price Update:**
```json
{
  "event": "prices",
  "data": {
    "prices": [
      {
        "assetId": "bitcoin",
        "price": 45000.50,
        "priceChangePercentage24h": 2.5,
        "updatedAt": "2024-01-23T12:00:00Z"
      }
    ],
    "rates": [
      {
        "symbol": "USD",
        "rate": 1.0
      }
    ]
  }
}
```

**Transactions Update:**
```json
{
  "event": "transactions",
  "data": {
    "walletId": "multicoin_0x742d35...",
    "transactions": ["ethereum_0xabc123..."],
    "assetIds": ["ethereum", "ethereum_0xtoken..."]
  }
}
```

Transaction updates include affected assets so clients can refresh the corresponding balances.

All server event variants are defined by `StreamEvent`:

| `event` | Payload |
|---|---|
| `prices` | prices and optional fiat rates |
| `balances` | wallet and affected asset IDs |
| `transactions` | wallet, transaction IDs, and affected asset IDs |
| `priceAlerts` | affected asset IDs |
| `nft`, `perpetual`, `fiatTransaction` | affected wallet ID |
| `inAppNotification` | wallet ID and notification |
| `support` | support-stream event |

## Notes

- Authentication happens once during WebSocket upgrade
- Reconnects replay at most the configured history limit (`DeviceStreamHistoryLimit`, default `25`)
- Price updates are batched every 5 seconds
- Run as separate service: `api websocket_stream`

## Implementation

- Stream handler: [`apps/api/src/websocket_stream/stream.rs`](../core/apps/api/src/websocket_stream/stream.rs)
- Client logic: [`apps/api/src/websocket_stream/client.rs`](../core/apps/api/src/websocket_stream/client.rs)
- Message types: [`crates/primitives/src/stream.rs`](../core/crates/primitives/src/stream.rs)
- Price payload: [`crates/primitives/src/websocket.rs`](../core/crates/primitives/src/websocket.rs)
