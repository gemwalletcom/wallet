# Device and Subscriptions

## Overview

Every install registers a **device** with the backend and tells it which **wallet addresses** to watch. The backend uses that registration to push notifications, to stream balance and price updates, and to answer wallet-scoped queries such as `/v2/devices/assets`.

Two pieces of state drive everything:

- **Device** — one record per install: push token, locale, currency, app version, price-alert flag, subscriptions version.
- **Subscriptions** — the set of `(walletId, chain, address)` the device watches, derived from the local wallets.

**A wallet-scoped endpoint returns 404 until that wallet is subscribed.** Both apps must therefore subscribe a wallet before requesting anything scoped to it.

## Data Model

`Device` is a shared primitive generated from core:

| Field | Notes |
|---|---|
| `id` | device id, hex Ed25519 public key |
| `platform`, `platformStore`, `os`, `model`, `version` | install identity |
| `token` | push token, empty when push is disabled |
| `locale`, `currency` | user settings |
| `isPushEnabled`, `isPriceAlertsEnabled` | user settings |
| `subscriptionsVersion` | bumped when the subscription set changes |

`subscriptionsVersion` is how the backend and the client detect that their idea of the subscription set diverged: the client stores its own version and compares it with the one on the returned device record.

## Endpoints

```
GET    /v2/devices/is_registered     is this device known
GET    /v2/devices                   fetch the device record
POST   /v2/devices                   register the device
PUT    /v2/devices                   update the device record
GET    /v2/devices/subscriptions     current subscriptions
POST   /v2/devices/subscriptions     add subscriptions
DELETE /v2/devices/subscriptions     remove subscriptions
```

Authentication for all of these: [Device Authentication](../core/docs/DEVICE_AUTHENTICATION.md).

## Flows

### Device registration

Runs on first launch, and again whenever the local "registered" flag is false.

```
is_registered / local flag → not registered
  POST /v2/devices          register
  → mark registered locally
```

If registration fails the device stays unregistered, so the next sync retries. Nothing else may assume the device exists.

### Device update

Currency, locale, push token, app version and the price-alert flag live in the device record. Changing any of them requires a `PUT /v2/devices`. The client compares the record it would send against the last known server state and skips the request when they are equal.

### Subscription update

Triggered by anything that changes the wallet or account set: wallet import, wallet creation, wallet deletion, and chain backfill for existing multi-coin wallets.

```
GET  /v2/devices/subscriptions       what the backend has
diff against local wallets
POST /v2/devices/subscriptions       add what is missing
DELETE /v2/devices/subscriptions     remove what is gone
PUT  /v2/devices                     publish the new subscriptions version
```

The diff is computed per wallet: addresses grouped by chain. Wallets present remotely but missing locally are removed in full.

### Ordering rule

The wallet must be subscribed **before** any wallet-scoped request runs for it. A fresh install imports a wallet and immediately asks for its assets, so the subscription must not be racing that request. Both apps satisfy this by making the sync run as part of, or ahead of, the first wallet-scoped fetch.

### Concurrency

Many places can trigger a sync at the same time (app start, import, stream connect, currency change, push token arrival). Concurrent triggers must collapse into a **single** network sync — callers join the in-flight one instead of queueing another.

### Failure

Local "this was published" state is written **only after** a successful sync. A failed sync therefore leaves the divergence in place and the next trigger retries it. No trigger is allowed to record success it did not achieve.

## Platform Implementations

| Concern | iOS | Android |
|---|---|---|
| Sync entry point | `DeviceService.update()` / `synchronizeIfNeeded()` | `SyncDevice.syncDevice()` |
| Needs-sync decision | `isSynchronized` = registered && !`subscriptionsVersionHasChange` | `needsSynchronization()` = registered && device and subscriptions equal the last pushed state |
| Change tracking | `Preferences.invalidateSubscriptions()` sets a dirty flag at each mutation site | none — divergence is derived by comparison |
| Subscription diff | `SubscriptionService.calculateChanges` | `List<Wallet>.subscriptionsDiff` |
| Concurrency | `DeviceSyncCoordinator` joins the in-flight task | `DeviceSyncCoordinator` joins the in-flight task |
| Reacting to wallet changes | `SubscriptionsObserver` (GRDB, accounts table) → `DeviceObserverService` | `DeviceObserverService` (Room, wallets + accounts) |
| Registration state | `preferences.isDeviceRegistered` | DataStore `device_registered` |

### Known divergences

These are deliberate and worth knowing before "making it consistent":

1. **Change tracking.** iOS marks a dirty flag at every mutation site; Android compares current state against the last successfully pushed state and needs no marking. The Android form cannot lose a mark and needs no ordering discipline at call sites; the iOS form is one flag read.
2. **Idle cost.** Android skips the sync entirely when nothing diverged, so a relaunch with no changes makes zero device requests. iOS re-checks its flag, which is also cheap, but a relaunch still fetches the device record when the flag is set.
3. **Fetch-site guard.** Android calls the sync at the top of the device-assets fetch, so a wallet cannot be queried before it is subscribed. iOS relies on the sync having started earlier (wallet insert observer) and does not guard the fetch.
4. **Observer scope.** iOS observes the accounts table and drops the first emission; Android observes wallets with their accounts. Changes made while the observer is stopped are picked up on Android at the next trigger because the comparison is stateless.

## Rules for Changes

Anything touching this subsystem on either platform must keep these true:

- A wallet-scoped request never runs for a wallet the backend has not been told about.
- Concurrent triggers produce one network sync, not one per caller.
- "Published" state is recorded only after a successful sync.
- Adding a wallet must not remove subscriptions of other wallets.
- Deleting a wallet removes its subscriptions.
- Nothing changed since the last sync means no requests.

## Implementation

**iOS**
- Device sync: [`ios/Packages/FeatureServices/DeviceService/DeviceService.swift`](../ios/Packages/FeatureServices/DeviceService/DeviceService.swift)
- Subscriptions: [`ios/Packages/FeatureServices/DeviceService/SubscriptionService.swift`](../ios/Packages/FeatureServices/DeviceService/SubscriptionService.swift)
- Observer: [`ios/Packages/FeatureServices/DeviceService/DeviceObserverService.swift`](../ios/Packages/FeatureServices/DeviceService/DeviceObserverService.swift)
- Mutation sites: [`ios/Packages/FeatureServices/WalletService/WalletService.swift`](../ios/Packages/FeatureServices/WalletService/WalletService.swift)

**Android**
- Device sync and subscriptions: [`android/data/repositories/.../device/DeviceRepository.kt`](../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/device/DeviceRepository.kt)
- Concurrency: [`android/data/repositories/.../device/DeviceSyncCoordinator.kt`](../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/device/DeviceSyncCoordinator.kt)
- Observer: [`android/data/repositories/.../device/DeviceObserverService.kt`](../android/data/repositories/src/main/kotlin/com/gemwallet/android/data/repositories/device/DeviceObserverService.kt)
- Case: [`android/gemcore/.../cases/device/SyncDevice.kt`](../android/gemcore/src/main/kotlin/com/gemwallet/android/cases/device/SyncDevice.kt)

**Backend**
- Device authentication: [`core/docs/DEVICE_AUTHENTICATION.md`](../core/docs/DEVICE_AUTHENTICATION.md)
- Streaming: [`core/docs/DEVICE_WEBSOCKETS.md`](../core/docs/DEVICE_WEBSOCKETS.md)
