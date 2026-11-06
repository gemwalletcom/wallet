# Lightning

Gem supports Lightning through Spark, using Breez's `spark-wallet` crate directly from Core in the [gem_spark crate](../core/crates/gem_spark/), exposed to the apps as the Bitcoin Lightning coin by the [gemstone lightning module](../core/gemstone/src/lightning/). It does not use `breez-sdk-spark` and does not need a Breez API key. Both apps ship it.

## Why Spark

Researched 2026-10-20 to 2026-10-26:

- Breez SDK – Liquid and – Greenlight are deprecated and their repositories are gone; Breez points every integrator to Spark.
- Boltz stopped all swaps on 2026-08-03, and Liquid peg-outs have been paused since the 2026-09-06 exploit, so nothing built on Boltz or Liquid works today.
- LDK Node is real self-custodial Lightning, but needs a liquidity provider agreement, a channel and opening fee per wallet, background wake-up to receive, and an alpha backup service.
- Ark (Bark, Arkade) is too young: no published audits, a 2026-10 exploit at Second, and Lightning receive missing on Arkade.
- Lexe hosts a real Lightning node per user in Intel SGX, but its node code is under a noncommercial licence and runs only on Lexe's servers.
- Paying invoices only, through a swap provider: none of our existing providers supports Lightning. Flashnet Orchestra can pay a BOLT11 invoice from stablecoins through `/v1/orchestration/quote`, but a real 2,000-sat test from Ethereum ETH on 2026-10-26 stalled because its Relay deposit leg delivered less than the exact-out order needed. Flashnet completed it 82 minutes after the deposit, after the invoice had expired, as a 2,092-sat Spark transfer to the invoice's Spark identity rather than a Lightning payment. Its in-house routing for Lightning is still rolling out. Revisit once fixed-amount invoices pay reliably within a typical invoice expiry, failures refund automatically, and the status API returns the preimage.

## Product decisions

Decided 2026-11-05, replacing the 2026-10-27 Lightning view:

- Lightning is its own coin, Bitcoin Lightning (`Chain::Lightning`, asset `lightning`, symbol BTC, 8 decimals so base units are sats). It has its own `ChainType` because it is a new family, not a Bitcoin family member. It lives in the wallet list, Manage, the asset screen and Activity like every other coin, and the Bitcoin asset screen links to it.
- Only phrase wallets get the account, through the same missing-chain setup as any new chain. The account address is the Spark address of the identity key (`m/8797555'/1'/0'`). Lightning is off by default; the user turns it on in Manage.
- The price follows Bitcoin: Core observes the Bitcoin price for the Lightning asset and mirrors each Bitcoin price update onto it.
- Spark is not named in the product beyond the explorer link.
- Lightning Address (`name@gemwallet.com`) is deferred.

## Sessions

Spark reads are authenticated with the identity key and incoming payments are claimed by signing, so the coin cannot refresh from an address alone. Core keeps open Spark wallets in one process registry keyed by Spark address. Opening is single-flight, so two callers never open the same `spark.sqlite` twice, and a wallet that finishes opening after `lock_all` is discarded instead of registered. The Lightning chain provider reads balance and payments from the registry and reports a wallet without a session as locked; the balance updater keeps the last stored balance for a locked wallet instead of failing the refresh. Transactions sync maps Spark payments into the normal transaction store on a whole-wallet or Lightning sync; a Lightning failure during a whole-wallet sync is ignored and does not stop the API sync. Device subscriptions leave Lightning out because the backend cannot index it.

On iOS the session opens without a second prompt: the app unlock context is kept for the unlocked period (cleared when the app relocks) and only the Lightning password provider reads the keystore password with it. `GemLightningService.open_enabled` opens the current wallet when Lightning is enabled, on wallet refresh and on return to the foreground, then syncs, claims and writes the balance and payments. Each sync step runs on its own: a failed operator sync, claim or resumed send does not stop the others, the balance and payments are still written, and `open_enabled` still reports the wallet as open. Turning Lightning on in Manage (`GemAssetSelectionService.set_assets_enabled`) calls it too, so the balance appears without a manual refresh; a failed or cancelled open there is ignored and the toggle still succeeds. Every move to the background locks all sessions and discards the signer keys; a wallet with a send in flight locks when that send returns, so the payment Confirm keeps running in the background is not cut off halfway. The receive screen polls only while the app is active; a poll that finds the session locked keeps the invoice on screen, since the payer can still pay it, and shows an Unlock button under it that authenticates once and lets the next poll reopen the session. A session that is locked when the screen opens shows the same Unlock action in place of the amount entry and opens after it. An invoice that passes its expiry is checked for a payment one last time before it reads as expired.

On Android the keystore password sits behind an Android Keystore key that needs user authentication within a 10-second window, so there is no unlock context to reuse. `LightningSessionObserver` is the one entry point (the `OpenLightning` case in `:gemcore`): it calls `open_enabled` for the current wallet on return to the foreground, after the app unlocks, and on wallet refresh, skipping a wallet whose open is still running, and every move to the background calls `lock_all`. Only `UserNotAuthenticatedException` reads the password as cancelled; any other keystore failure surfaces as an error. So an open outside the window, such as a return to the foreground without a relock or turning Lightning on in Manage long after unlocking, stays closed until the next unlock. The receive screen opens the session itself and asks for authentication when it is locked, once per lock: a cancelled open right after authenticating shows the unlock message instead of prompting again. It reopens the session on every poll so it survives a background lock, pauses polling while hidden, stops at the invoice expiry, and on payment syncs the wallet, closes and shows a Payment received toast.

Send uses the standard flow. The recipient is a BOLT11 invoice, with or without a `lightning:` prefix: an invoice with an amount goes straight to confirm, an amountless one goes to the amount screen. The provider's transaction load quotes the fee through the open session, the Lightning signer packs the sender's Spark address, invoice, confirmed amount and quoted fee into the signed data after the usual authorization, signing a digest prefixed with a fixed Gem Lightning transfer domain, and broadcast pays it with the quoted fee as the maximum only when the 65-byte signature verifies against the open wallet's identity key and the amount matches an invoice that carries one and returns the Spark payment id as the transaction hash. Status checks look up that one transfer by id and the payments sync reads the same id, so the pending row and the synced payment are one transaction. A locked wallet leaves the row pending instead of failing it. A Lightning row times out after 16 days, the expiry Spark puts on the leaves it hands the gateway for a send: by then the gateway has either paid or the transfer has returned. Receive on the coin opens the invoice screen, which polls the session until the invoice is paid.

## Payment types

What `spark-wallet` 0.26.1 supports, and what Gem has to build on top:

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
| Unified QR (`bitcoin:<address>?lightning=lnbc…`) | Gem code | Gem code | The payment decoder currently rejects Lightning-only URIs; pay the BOLT11 part when present |
| On-chain Bitcoin in or out | Yes | Yes | Not Lightning: deposit addresses that are claimed into Spark, and cooperative exit to an on-chain address |

### Lightning Address

A Lightning Address such as `name@gemwallet.com` works without the app being online. The payer's wallet calls `/.well-known/lnurlp/name` on our domain. Our server looks up the name's Spark identity public key and asks Lightspark's gateway for an invoice for that key, which `create_lightning_invoice` supports through its `public_key` argument. The payment lands as a pending transfer to the user's own key, and the app claims it on the next unlock; the server never holds funds. Breez's LNURL server does the same (`crates/breez-sdk/lnurl/src/routes.rs` in `breez/spark-sdk`), but it requires a Breez API key on mainnet, so Gem would serve the endpoints from its own API with `spark-wallet`.

It needs a server-side Spark wallet that requests invoices, a table of names to identity public keys, registration signed with the Spark identity key, and name rules (characters, reserved names, squatting, renames). The server decides which key a name resolves to, so a compromised server could redirect new payments, as with any Lightning Address provider. If Spark gains BOLT12, the same name can also be published as a BIP-353 DNS record.

## Trust model

Spark is not plain Lightning and must not be presented as plain Bitcoin. Three permissioned operators (Lightspark, Breez, Flashnet) co-sign with 2-of-3 threshold signatures, and security after a transfer relies on operators deleting old key shares, which cannot be proven. Spark still calls itself beta mainnet and has no published audit. Users can exit to Bitcoin on their own, but exits take days, need an external UTXO for fees, and do not pay off for small balances. Lightning send and receive go through Lightspark's gateway (the SSP): 0.25% to send, free to receive.

## Keys

The mnemonic never crosses UniFFI. The wallet opens only through `GemKeystore.lightning_wallet(keystore_id, password, storage_dir)`, which decrypts the v4 keystore file in Rust, rejects private-key wallets, derives the mainnet Spark account key, and drops the phrase and seed straight away ([Keystore v4](KEYSTORE_V4.md)).

The returned wallet holds the account key in memory, because every operator login, claim and send needs it. Apps open wallets only while unlocked and in the foreground and call `GemLightningService.lock_all` on background. Gem erases its own copy of the account key once the wallet is open. Spark's library reaches keys only through the `Signer` trait and otherwise sees public keys, signatures and secrets encrypted to the signer; the one raw key it asks for is a static deposit key, which the deposit claim hands to the gateway by design. Gem wraps Spark's `DefaultSigner` in a lockable signer, so `lock` discards the key in Core even while the app or background tasks still hold the wallet; a call already in flight finishes first, and a send in flight holds the lock back until it returns while new sends are refused. `DefaultSigner` does not yet erase its master key on drop, so until [breez/spark-sdk#1272](https://github.com/breez/spark-sdk/pull/1272) ships in a tag Gem pins, lock releases the memory without zeroing it.

A locked app shows the balance it cached while unlocked. Operator sessions last about 15 minutes, and refreshing or claiming needs the unlocked key.

## Core API

Balance, activity and sends reach the apps through the usual chain services. The Lightning-specific surface is small:

- `GemLightningService`: `open(wallet_id)` and `open_enabled(wallet_id)` open a wallet through the keystore (the second only when the wallet has Lightning enabled), `sync(wallet_id)` claims and writes the balance and payments, and `lock_all` stops background work and discards every signer key, so calls that need a signature fail until a wallet is opened again.
- `GemLightningWallet`, for the receive screen: `create_invoice(amount_sats, expiry_secs)` (no amount creates an amountless invoice), `payments(limit, offset)` newest first with internal leaf swaps and cooperative exits left out, and `sync`.
- Deleting a wallet closes its Lightning session, removes its storage directory and deletes its keystore secrets while holding the registry's opening lock, so no open can recreate the session or `spark.sqlite` in between.
- `GemKeystore.sign_auth` and `export_private_key` refuse `Chain::Lightning`, so the identity key signs only Lightning transfers and never leaves Core.
- In Core, `prepare_send` rejects an expired invoice, an invoice for another network, an amount that differs from the invoice's, and an amountless invoice without an amount, and `sync` resumes interrupted sends.

## Derivation

Spark keys hang off one account key. Wallets disagree on which account key a seed maps to:

| Convention | Identity key | Used by |
|---|---|---|
| Account 1 | `m/8797555'/1'/0'` | Lightspark JS SDK and Breez SDK defaults on mainnet, so any app that does not override them; Gem |
| Account 0 | `m/8797555'/0'/0'` | Xverse (verified 2026-10-26 with a throwaway mobile wallet seed); also the SDK default on regtest |
| BIP44 coin 998 | `m/44'/998'/<networkId>'/0/<index>` | Tether WDK (`wdk-wallet-spark`) |

Under `m/8797555'/<account>'` the children are identity `0'`, leaf signing `1'`, deposit `2'`, static deposit `3'` and HTLC preimage `4'`. The WDK puts its identity key at the root of its path and the other keys at `0'` to `3'` under it. Wallet of Satoshi and Privy are unverified.

Rules:

- Gem derives account 1 for every wallet, created or imported. There is no path choice in the UI.
- On import, Gem also checks account 0 and the WDK path for a balance and offers to bring in what it finds, so an Xverse or WDK user does not see an empty Spark balance. Each check is one operator login and a balance read.

## Storage and recovery

Spark's `spark-sqlite` asks `rusqlite` for its own bundled SQLite. Android ships that copy inside `libgemstone.so`, where its symbols stay private to the library. iOS links Core statically into the app, where a second SQLite would take over the `sqlite3_*` calls of the app's own GRDB database, so `ios/scripts/generate-stone.sh` builds with `LIBSQLITE3_SYS_USE_PKG_CONFIG=1` and Spark uses the system SQLite the app already links. Each wallet keeps one SQLite file, `spark.sqlite`, in `<storage_dir>/<keystore_id>/` under the storage directory the app passes, removed when the wallet is deleted: Breez's `spark-sqlite` tree store for leaves and exit chains, and Gem's tables for operator sessions and pending sends. Session tokens are stored in plain text; moving funds still needs signatures from the account key. Before each Lightning send, Gem records the transfer id and invoice. A send for an invoice that already has a record finishes that transfer instead of starting another, so confirming again after a failed or interrupted send cannot open a second transfer for the same invoice; a new transfer starts only once the gateway reports the recorded one never existed. `sync` resumes every recorded send through `resume_lightning_send`, which the gateway keys on the transfer id, and drops the record once the gateway has the send or there is nothing left to pay, or once the invoice has expired. A record that fails to resume is kept and the rest still run. After wiping storage, the operators return the balance, exit chains and history from the seed alone. If storage is wiped and the operators are also gone, the seed cannot recover funds, because the stored exit chains are the only way to exit unilaterally.

## Mainnet test

On 2026-10-26 a throwaway wallet received 5,000 sats from Phoenix over Lightning in full, claimed in the background within seconds, then paid a 1,000-sat Phoenix invoice for a 7-sat fee (estimate 7, reported 7, balance 5,000 to 3,993). Both transfers completed as `PreimageSwap` through Lightspark's gateway. On regtest the payment result reported a fee of 0, so fees shown to users come from the estimate or the balance change until that is understood.

## Open items

- App work: `bitcoin:?lightning=` QR parsing, Spark-address sends, and a wallet event callback so incoming payments update the app without polling.
- Backend: the backend learns the chain from the deployed Core image; search, prices by asset id and the chain logo need that deploy plus a `lightning` logo in the assets repository.
- On-chain deposit claiming, withdrawal to on-chain Bitcoin, and the import discovery above. Cooperative exits are left out of payments until withdrawal ships and maps the exit's acceptance.
- `spark-wallet` and `spark-sqlite` are pinned to tag 0.26.1. They are version 0.1.0, unpublished, and change often, so run Breez's store conformance tests (`test-utils`) against both stores on every bump.
