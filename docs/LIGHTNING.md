# Lightning

Gem supports Lightning through Spark, using Breez's `spark-wallet` crate directly from Core in the [gem_spark crate](../core/crates/gem_spark/), exposed to the apps as the Bitcoin Lightning coin by the [gemstone lightning module](../core/gemstone/src/lightning/). It does not use `breez-sdk-spark`.

## Product decisions

- Lightning is its own coin, Bitcoin Lightning (`Chain::Lightning`, asset `lightning`, symbol sats with 0 decimals, so one unit is one satoshi), with its own `ChainType`. It lives in the wallet list, Manage, the asset screen and Activity like every other coin, and the Bitcoin asset screen links to it.
- Only phrase wallets get the account. Lightning is off by default; the user turns it on in Manage. Its price is Bitcoin's price per satoshi: the backend shares Bitcoin's price rows and divides by `AssetId::price_scale` wherever it serves a price, chart or candle for the asset.
- Once the user turns it on, Lightning works without prompts: it has its own stored key, separate from the secret phrase.
- Spark is not named in the product beyond the explorer link. Lightning Address is deferred.

## Keys and sessions

Spark keys hang off one account key, `m/8797555'/1'` (identity key `m/8797555'/1'/0'`, the Lightspark and Breez default; Xverse uses account 0 and Tether's WDK uses `m/44'/998'`). The mnemonic never crosses UniFFI: turning Lightning on in Manage derives the account key in Rust from the v4 keystore with the keystore password (`GemKeystore.lightning_account_key`), then stores it in `GemSecureStore` (on iOS `GemstoneEncryptedPreferencesStore`, AES-GCM encrypted in the app container under a Keychain key, so it goes with the app; the Tink-encrypted store on Android) under `lightning_account_<keystore_id>`, without biometrics. Deleting the wallet removes it with the wallet's secrets. Private-key export never offers Lightning and auth messages use the fixed auth chain, so the identity key signs only Lightning transfers.

Spark reads are authenticated with the identity key and incoming payments are claimed by signing, so the coin cannot refresh from an address alone. Core keeps open Spark wallets in one process registry keyed by Spark address. Balance, fee, send, status and payment sync open a wallet on demand from its stored key; opening is single-flight, and a session stays open until the wallet is deleted. `GemLightningService.open_enabled` opens the current wallet from its stored key when Lightning is enabled and syncs it once, on return to the foreground (iOS after the app unlocks) and after setup in Manage. If setup cannot read the password (cancelled, or on Android outside the keystore's authentication window), the wallet has no stored key: those calls fail as cancelled, the balance keeps its last stored value, and the receive screen shows Unlock, which derives and stores the key and retries.

## Send and receive

The recipient is a BOLT11 invoice, with or without a `lightning:` prefix: an invoice with an amount goes straight to confirm, an amountless one goes to the amount screen. Invoice decoding accepts mainnet invoices only, and the send rejects an expired invoice, an amount that differs from the invoice's, and an amountless invoice without an amount. The signer signs the sender's Spark address, invoice, amount and quoted fee as typed JSON behind a fixed Gem domain; broadcast pays only when the signature verifies against the open wallet's identity key, with the quoted fee as the maximum, and returns the Spark transfer id as the transaction hash. Status checks and the payments sync use that id, so the pending row and the synced payment are one transaction. A Lightning row times out after 16 days, the expiry Spark puts on the leaves it hands the gateway.

Receive opens one screen with a QR of an any-amount invoice, copy and share. The QR holds the invoice in uppercase, which BOLT11 allows, so it encodes in the denser alphanumeric mode. The app polls `GemLightningService.receive_state` at the `Receive` refresh interval while the screen is visible, replaces the invoice when it expires, and closes with a Received toast once it is paid.

## Storage and recovery

Spark keeps its leaves in memory while a wallet is open and reads them from the operators on the next open. The only Lightning state Gem stores is a record of each send (transfer id, invoice, amount), in the wallet's preferences under the `lightning_sends` key (Android commits the write before the send goes out), deleted with the wallet. Confirming an invoice that already has a record finishes that transfer instead of paying twice, a send with an unknown outcome stays pending until `sync` resumes it, and a record is dropped once its transfer failed, never existed, or its invoice expired.

The operators return the balance and history from the seed alone. Gem does not keep exit chains on the device, so if the operators are gone the seed cannot recover funds; storing them comes with on-chain withdrawal.

## Trust model

Spark is not plain Lightning and must not be presented as plain Bitcoin. Three permissioned operators (Lightspark, Breez, Flashnet) co-sign with 2-of-3 threshold signatures, and security after a transfer relies on operators deleting old key shares, which cannot be proven. Spark still calls itself beta mainnet and has no published audit. Lightning send and receive go through Lightspark's gateway: 0.25% to send, free to receive. The stored Lightning key is a hot key: anyone who can read the app's secure storage on an unlocked device can spend the Lightning balance, which is the trade-off for prompt-free use.

## Partner attribution

Breez attributes Spark traffic to Gem through an `x-partner-jwt` header on every operator and gateway request. The header carries a token Breez signs for its partners and expires after 7 days, so `PartnerHeaders` in `gem_spark` (`partner/`) fetches it from `nd1.breez.technology/api/jwt` with Gem's Breez API key, the way Breez's own SDK does, and keeps it in memory. The key is a client-side value, like in every app built on the Breez SDK: it is read from the `BREEZ_API_KEY` environment variable when `gemstone` is compiled and is not in the repository. It is fetched in the background on the first Spark request of a run and again once it has less than a day left, retried at most once a minute, so a slow or failing Breez server never delays Lightning; requests made before it arrives go out without the header.

## Open items

- `bitcoin:?lightning=` QR parsing, Spark-address sends, Lightning Address, and a wallet event callback so incoming payments update without polling.
- On-chain deposit claiming, withdrawal to on-chain Bitcoin, and import discovery of account 0 and the WDK path.
- `spark-wallet` is pinned to commit `6e41980da3` (version 0.1.0, unpublished) until Breez tags 0.29.0, and built without its default `tokens` feature, which leaves out `spark-primitives` and `prost-reflect`; the app does not send or issue tokens. The `spark` crate still compiles protobuf definitions, so every machine and CI runner that builds `gemstone` needs `protoc`, including the F-Droid build and the release setup actions. 
