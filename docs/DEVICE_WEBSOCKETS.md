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

### Asset Price Lifecycle

- Both apps use `GemStreamService` for connection preparation, session eligibility, device synchronization, reconnect subscription resets, and event application using the current Core currency. Native observers own foreground/session observation, socket I/O, cancellation, and ordered event forwarding.
- Foreground observers prepare each wallet session in Core. Losing the session clears retained subscription intent; wallet changes cancel and finish the prior observer before opening its replacement. Session changes while backgrounded cannot open a connection.
- Wallet setup and asset enable/disable changes rebuild price subscriptions from stored enabled assets; they do not call `POST /v1/prices`.
- The first price request on each connection is `subscribePrices`, which returns current USD prices and fiat rates. Later `addPrices` requests return prices for the expanded subscription without rates.
- Core retains additional requests from asset details and swaps while disconnected and across reconnects. Switching wallets clears those extra requests and builds the subscription from the new wallet's enabled assets and price alerts.
- Subscription changes are serialized. A failed send keeps the requested assets pending, and each new connection resets the sent subscription state before resubscribing.
- Clients process received events in order, saving the initial exchange rates before applying later price updates. Both apps cancel event handling when backgrounded and finish transport cleanup before a replacement observer starts.
- The server closes the connection when a client message cannot be processed, allowing clients to reconnect and replay subscriptions instead of keeping a silently failed subscription open.

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

Transaction updates include affected assets so clients can refresh the corresponding balances. The server does not emit separate balance events for these transactions.

**Balance Update:**
```json
{
  "event": "balances",
  "data": {
    "walletId": "multicoin_0x742d35...",
    "assetIds": ["ethereum", "ethereum_0xtoken..."]
  }
}
```

Both update types require `assetIds`. Balance updates no longer include the legacy singular `assetId`. iOS and Android releases starting with [2.104](https://github.com/gemwalletcom/wallet/releases/tag/2.104) consume the arrays for both event types.

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
