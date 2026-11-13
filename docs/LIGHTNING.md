# Lightning

Gem supports Lightning through Spark, using Breez's `spark-wallet` crate directly from Core in the [gem_spark crate](../core/crates/gem_spark/), exposed to the apps as the Bitcoin Lightning coin by the [gemstone lightning module](../core/gemstone/src/lightning/). It does not use `breez-sdk-spark` and does not need a Breez API key. Both apps ship it.

## Product decisions

- Lightning is its own coin, Bitcoin Lightning (`Chain::Lightning`, asset `lightning`, symbol BTC, 8 decimals so base units are sats). It has its own `ChainType` because it is a new family, not a Bitcoin family member. It lives in the wallet list, Manage, the asset screen and Activity like every other coin, and the Bitcoin asset screen links to it.
- Only phrase wallets get the account, through the same missing-chain setup as any new chain. The account address is the Spark address of the identity key (`m/8797555'/1'/0'`). Lightning is off by default; the user turns it on in Manage.
- The price follows Bitcoin: the backend prices the `lightning` asset from Bitcoin's CoinGecko and Pyth feeds.
- Spark is not named in the product beyond the explorer link.
- Lightning Address (`name@gemwallet.com`) is deferred.

## Sessions

Spark reads are authenticated with the identity key and incoming payments are claimed by signing, so the coin cannot refresh from an address alone. Core keeps open Spark wallets in one process registry keyed by Spark address. Opening is single-flight, so two callers never open the same wallet twice. Each open reads the lock generation before its first await, so an open that started or waited before `lock_all` is discarded instead of registered. The Lightning chain provider reads the balance, the send fee and a transfer's status from the registry and reports a wallet without a session as locked; the balance updater keeps the last stored balance for a locked wallet instead of failing the refresh. Transactions sync maps Spark payments into the normal transaction store on a whole-wallet or Lightning sync; a Lightning failure during a whole-wallet sync is ignored and does not stop the API sync. Device subscriptions and the backend transaction sync leave out chains the backend does not index (`Chain::is_indexed`), which is Lightning.

On iOS the session opens without a second prompt: the app unlock context is kept for the unlocked period (cleared when the app relocks) and only the Lightning password provider reads the keystore password with it. `GemLightningService.open_enabled` opens the current wallet when Lightning is enabled, on wallet setup or switch and on return to the foreground, and syncs, claims and writes the balance and payments only when it opened the wallet; a wallet that is already open is left alone. Each sync step runs on its own: Spark's sync claims incoming payments itself, a failed sync still claims them separately, a failed resumed send does not stop the others, and the balance and payments are still written. Turning Lightning on in Manage (`GemAssetSelectionService.set_assets_enabled`) calls it too, so the balance appears without a manual refresh; a failed or cancelled open there is ignored and the toggle still succeeds. Every move to the background locks all sessions and discards the signer keys; a wallet with a send in flight locks when that send returns, so the payment Confirm keeps running in the background is not cut off halfway. A session that is locked when the receive screen opens, or when a poll finds it locked, shows an Unlock action that authenticates once and reopens it.

On Android the keystore password sits behind an Android Keystore key that needs user authentication within a 10-second window, so there is no unlock context to reuse. `LightningSessionObserver` is the one entry point (the `OpenLightning` case in `:gemcore`): it calls `open_enabled` for the current wallet on return to the foreground, after the app unlocks, and on wallet refresh, and every move to the background calls `lock_all`. Only `UserNotAuthenticatedException` reads the password as cancelled; any other keystore failure surfaces as an error. So an open outside the window, such as a return to the foreground without a relock or turning Lightning on in Manage long after unlocking, stays closed until the next unlock. The receive screen asks for authentication when the session is locked, once per lock: a cancelled open right after authenticating shows the Unlock button instead of prompting again.

Send uses the standard flow. The recipient is a BOLT11 invoice, with or without a `lightning:` prefix: an invoice with an amount goes straight to confirm, an amountless one goes to the amount screen. The provider's transaction load quotes the fee through the open session, the Lightning signer packs the sender's Spark address, invoice, confirmed amount and quoted fee into the signed data after the usual authorization, signing a digest prefixed with a fixed Gem Lightning transfer domain, and broadcast pays it with the quoted fee as the maximum only when the 65-byte signature verifies against the open wallet's identity key and the amount matches an invoice that carries one and returns the Spark payment id as the transaction hash. Status checks look up that one transfer by id and the payments sync reads the same id, so the pending row and the synced payment are one transaction. A locked wallet leaves the row pending instead of failing it. A Lightning row times out after 16 days, the expiry Spark puts on the leaves it hands the gateway for a send: by then the gateway has either paid or the transfer has returned. Receive on the coin opens one screen with a QR of an any-amount invoice, copy and share; an amount typed above the QR replaces the invoice. Core owns the screen's logic through `GemLightningService.receive` and `receive_state`: the app polls `receive_state` at the `Receive` refresh interval while the screen is visible, replaces the invoice when it expires, and closes with a Received toast once it is paid.

## Payment types

What `spark-wallet` supports, and what Gem has to build on top:

| Type | Send | Receive | Notes |
|---|---|---|---|
| BOLT11 invoice with amount | Yes, verified on mainnet | Yes, verified on mainnet | `pay_lightning_invoice`, `create_lightning_invoice` |
| BOLT11 invoice without amount | Supported, untested | Invoice creation works on mainnet, payment untested | Send passes `amount_to_send`; receive creates the invoice with amount 0 |
| Lightning Address / LNURL-pay (`name@domain`) | Gem code | Needs a Gem server | Send: resolve the address, fetch the invoice from the callback, check its amount and description hash, pay the BOLT11. Receive: see Lightning Address below |
| LNURL-withdraw | — | Gem code | Mint a BOLT11 invoice and submit it to the service's callback |
| LNURL-auth | — | — | Login, not a payment; needs its own key derivation from the seed and is not Spark-specific |
| BOLT12 offer, BIP-353 (`₿name@domain`) | No | No | No BOLT12 code in `spark-wallet` or Lightspark's gateway |
| Keysend (spontaneous payment) | No | No | Not in `spark-wallet` |
| Spark address or Spark invoice (`spark1…`) | Yes | Yes | Spark-to-Spark transfer, no Lightning hop; currently free. `prefer_spark` pays a BOLT11 invoice that embeds a Spark address directly |
| Unified QR (`bitcoin:<address>?lightning=lnbc…`) | Gem code | Gem code | The payment decoder reads `lightning:` URIs and bare invoices but not the `lightning=` parameter of a `bitcoin:` URI; pay the BOLT11 part when present |
| On-chain Bitcoin in or out | Yes | Yes | Not Lightning: deposit addresses that are claimed into Spark, and cooperative exit to an on-chain address |

### Lightning Address

A Lightning Address such as `name@gemwallet.com` works without the app being online. The payer's wallet calls `/.well-known/lnurlp/name` on our domain. Our server looks up the name's Spark identity public key and asks Lightspark's gateway for an invoice for that key, which `create_lightning_invoice` supports through its `public_key` argument. The payment lands as a pending transfer to the user's own key, and the app claims it on the next unlock; the server never holds funds. Breez's LNURL server does the same (`crates/breez-sdk/lnurl/src/routes.rs` in `breez/spark-sdk`), but it requires a Breez API key on mainnet, so Gem would serve the endpoints from its own API with `spark-wallet`.

It needs a server-side Spark wallet that requests invoices, a table of names to identity public keys, registration signed with the Spark identity key, and name rules (characters, reserved names, squatting, renames). The server decides which key a name resolves to, so a compromised server could redirect new payments, as with any Lightning Address provider. If Spark gains BOLT12, the same name can also be published as a BIP-353 DNS record.

## Trust model

Spark is not plain Lightning and must not be presented as plain Bitcoin. Three permissioned operators (Lightspark, Breez, Flashnet) co-sign with 2-of-3 threshold signatures, and security after a transfer relies on operators deleting old key shares, which cannot be proven. Spark still calls itself beta mainnet and has no published audit. Users can exit to Bitcoin on their own, but exits take days, need an external UTXO for fees, and do not pay off for small balances. Lightning send and receive go through Lightspark's gateway (the SSP): 0.25% to send, free to receive.

## Partner attribution

Breez attributes Spark traffic to Gem through an `x-partner-jwt` header on every operator and gateway request. The Breez API key stays on the backend (`breez.key.secret`, `BREEZ_KEY_SECRET` on the API server): `GET /v1/devices/lightning/token` returns Breez's partner token as `{"token", "expiresAt"}`, cached in Redis for half its remaining life (tokens last 7 days). The token names Gem, not the user, so the app keeps one in memory for every wallet. It asks for a new one beside each session open, so a slow or failing backend never delays Lightning, and on each foreground `open_enabled`, whenever the one it holds has less than a day left; it stops sending a token in that last day. Without a token requests go out without the header, which Spark does not require.

## Keys

The mnemonic never crosses UniFFI. The wallet opens only through `GemKeystore.lightning_account_key(keystore_id, password)`, which decrypts the v4 keystore file in Rust, rejects private-key wallets, derives the Spark account key, and drops the phrase and seed straight away ([Keystore v4](KEYSTORE_V4.md)).

The returned wallet holds the account key in memory, because every operator login, claim and send needs it. Apps open wallets only while unlocked and in the foreground and call `GemLightningService.lock_all` on background. Gem erases its own copy of the account key once the wallet is open. Spark's library reaches keys only through the `Signer` trait and otherwise sees public keys, signatures and secrets encrypted to the signer; the one raw key it asks for is a static deposit key, which the deposit claim hands to the gateway by design. Gem wraps Spark's `DefaultSigner` in a lockable signer, so `lock` discards the key in Core even while the app or background tasks still hold the wallet; a call already in flight finishes first, and a send in flight holds the lock back until it returns while new sends are refused. `DefaultSigner` does not yet erase its master key on drop, so until [breez/spark-sdk#1272](https://github.com/breez/spark-sdk/pull/1272) ships in a tag Gem pins, lock releases the memory without zeroing it.

A locked app shows the balance it cached while unlocked. Operator sessions stay in memory, so each open logs in to the operators again, and refreshing or claiming needs the unlocked key.

## Core API

Balance, activity and sends reach the apps through the usual chain services. The Lightning-specific surface is small:

- `GemLightningService`: `open_enabled()` opens the current wallet when it has Lightning enabled, `receive(wallet_id, amount, format)` opens the wallet if needed and returns the receive screen's QR value, copy value and amount field (an empty or zero amount creates an amountless invoice), `receive_state(wallet_id, invoice)` syncs once and answers pending, received or expired, updating the balance and payments when received, and `lock_all` stops background work and discards every signer key, so calls that need a signature fail until a wallet is opened again.
- Deleting a wallet closes its Lightning session, and deletes its keystore secrets while holding the registry's opening lock, so no open can recreate the session in between; its pending sends go with the wallet row they reference.
- Private-key export never offers Lightning (`supports_private_key_import` is false for it) and auth messages are signed with the fixed auth chain, so the identity key signs only Lightning transfers and never leaves Core.
- In Core, invoice decoding accepts mainnet invoices only, the send path (`amount_to_send`) rejects an expired invoice, an amount that differs from the invoice's, and an amountless invoice without an amount, and `sync` resumes interrupted sends. The payment decoder reads `lightning:` URIs and bare invoices, so scanning or pasting an invoice with an amount goes straight to confirm.

## Derivation

Spark keys hang off one account key. Wallets disagree on which account key a seed maps to:

| Convention | Identity key | Used by |
|---|---|---|
| Account 1 | `m/8797555'/1'/0'` | Lightspark JS SDK and Breez SDK defaults on mainnet, so any app that does not override them; Gem |
| Account 0 | `m/8797555'/0'/0'` | Xverse |
| BIP44 coin 998 | `m/44'/998'/<networkId>'/0/<index>` | Tether WDK (`wdk-wallet-spark`) |

Under `m/8797555'/<account>'` the children are identity `0'`, leaf signing `1'`, deposit `2'`, static deposit `3'` and HTLC preimage `4'`. The WDK puts its identity key at the root of its path and the other keys at `0'` to `3'` under it. Wallet of Satoshi and Privy are unverified.

Rules:

- Gem derives account 1 for every wallet, created or imported. There is no path choice in the UI.

## Storage and recovery

Spark keeps its leaves in memory for as long as a wallet is open and reads them from the operators again on the next open, so a wallet that never turns Lightning on holds nothing. The only Lightning state Gem stores is its pending sends, in the app database (GRDB on iOS, Room on Android) behind `GemLightningStore`: the `lightning_pending_sends` table, keyed by wallet id and deleted with the wallet. Before each Lightning send, Gem records the transfer id, invoice and amount. A send for an invoice that already has a record finishes that transfer instead of starting another, so confirming again after a failed or interrupted send cannot open a second transfer for the same invoice; a different amount for the same amountless invoice is refused, and a new transfer starts only once the gateway reports the recorded one never existed or failed. A send whose outcome is unknown after an error, because the operators committed it or the commit check itself failed, returns its transfer id, so the app tracks it as pending and the next sync finishes it; only a send the operators never saw fails. A status check reads a transfer that is gone and has no record as failed, so a send that never happened does not sit pending until the 16-day timeout. `sync` resumes every recorded send through `resume_lightning_send`, which the gateway keys on the transfer id. A record stays while its transfer is pending or paid, so confirming the same invoice again keeps returning that transfer until the invoice expires; it is dropped once the operators have no such transfer or it failed, or once the invoice has expired. A record that fails to resume is kept and the rest still run, and a send still running in this process is never resumed or dropped, even by a second session opened on the same wallet. The operators return the balance, exit chains and history from the seed alone. Gem does not keep exit chains on the device, so if the operators are gone the seed cannot recover funds; unilateral exit needs the chains stored while the operators are still reachable, which comes with on-chain withdrawal.

## Open items

- App work: `bitcoin:?lightning=` QR parsing, Spark-address sends, and a wallet event callback so incoming payments update the app without polling.
- Backend: the backend learns the chain from the deployed Core image; search, prices by asset id and the chain logo need that deploy plus a `lightning` logo in the assets repository.
- On-chain deposit claiming, withdrawal to on-chain Bitcoin, and import discovery: checking account 0 and the WDK path for a balance on import and offering to bring in what it finds, so an Xverse or WDK user does not see an empty Spark balance. Cooperative exits are left out of payments until withdrawal ships and maps the exit's acceptance.
- `spark-wallet` is pinned to tag 0.26.1. It is version 0.1.0, unpublished, and changes often. Its `spark-primitives` dependency compiles protobuf definitions at build time, so every machine and CI runner that builds `gemstone` needs `protoc`, including the F-Droid build and the release setup actions. Spark makes its own operator gRPC and gateway HTTPS calls (`reqwest` through `platform-utils`), so Lightning traffic does not go through the app's native HTTP provider; the Core CI check that keeps `reqwest` out of `gemstone` prunes that one crate. `WalletBuilder::with_ssp_http_client` could route the gateway calls through the native provider later.
