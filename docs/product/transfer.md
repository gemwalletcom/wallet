# Transfer

Send an asset to an address, a name or a contact, or pay a scanned QR code or payment link, with one confirmation screen that shows everything before it is signed.

## Send

```mermaid
flowchart LR
    A[Send] --> B[Pick asset] --> C[Recipient] --> D[Amount] --> E[Confirm] --> F{Approve?}
    F -- authenticate --> G[Sent] --> H[Pending in Activity] --> I[Confirmed]
    F -- cancel --> C
```

- The user taps Send on the wallet screen or an asset and picks the asset to send.
- Recipient: the user pastes or types an address or a name, scans a QR code, or picks a contact or one of their own wallets; a name is resolved while typing, and a memo field appears on networks that use one.
- Amount: the user types in crypto or in fiat and switches between them, or taps Max, which keeps the network fee back on a native coin; the available balance is shown.
- Confirm shows the amount with its value, the recipient with its name when known, the network, the fee (with faster or slower options and a custom fee where the network allows), the wallet, and any warning from the simulation of the transaction. When the recipient is flagged, a "Suspicious address" warning with one line on why appears under the amount and the transfer cannot be confirmed.
- The user confirms with the device's authentication; the transaction is sent and the app returns to where Send started.
- The transaction appears at once as Pending in Activity and on the asset, is tracked until the network confirms it, and its balance updates then.

## Scanned codes and payment links

```mermaid
flowchart LR
    A[Scan or paste] --> B{What is it?}
    B -- complete payment --> C[Confirm]
    B -- address without an amount --> H[Amount] --> C
    B -- required memo missing --> D[Recipient screen to complete]
    B -- plain address on several networks --> E[Pick the asset] --> B
    B -- WalletConnect Pay link --> F[Merchant request review] --> C
    B -- WalletConnect code --> G[Connection proposal]
```

- Supported codes: a plain address; Bitcoin, Litecoin, Bitcoin Cash, Dogecoin, Dash and Zcash payment codes (amount, memo, label); XRP with a destination tag; Ethereum coin and token transfers; Solana Pay; TON transfers with a comment; WalletConnect Pay links.
- The scanner reads them from the camera or a picked image.
- A complete payment (asset, amount and, where required, memo) opens Confirm directly; one without an amount opens the amount screen (an amount with more decimals than the asset has is dropped, never rounded); the recipient screen opens only when a network that uses a memo has none, or the address is not valid for the asset; a plain address that matches several networks asks which asset to send, then follows the same rule.
- A WalletConnect Pay link opens a review of the merchant's request and, once confirmed, pays it and reports the result back to the merchant.

## Rules

- Nothing is signed or sent without the confirmation screen showing the amount, recipient, network and fee.
- A payment code's amount fills the amount field in the device's number format and keeps its exact value, because a "0.001" read with a comma decimal separator would become 1.
- A simulation that cannot answer never blocks sending; a simulation that finds a risk shows it before the user confirms.
- The balance changes a simulation predicts show every digit of the amount, never a rounded value, so what the user approves is exactly what moves.
- When Confirm loads with a problem the user can act on (not enough balance or network fee, a required memo, a risky transaction), its explanation opens by itself; other load errors stay in the error row.
- When a network rejects a sent transaction, the user sees the network's own reason, because it is often the only explanation there is.
- Dash sends standard non-replaceable payments, which the Dash network automatically attempts to lock with InstantSend when their inputs are eligible; until the provider exposes that lock, Activity remains Pending until the transaction is mined.

## Test codes

Scan these from another device with a test wallet; do not submit the transactions. Token tests need the exact token enabled in the wallet.

<details>
<summary>Payment codes and the expected result</summary>

| | |
|---|---|
| **Bitcoin amount**<br><img src="../data/payments/bitcoin-exact-amount.png" width="180" alt="Bitcoin amount QR code"><br>`bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.0001`<br>Confirm `0.0001 BTC`. | **Bitcoin address only**<br><img src="../data/payments/bitcoin-address-only.png" width="180" alt="Bitcoin address QR code"><br>`bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4`<br>Open the amount screen. |
| **Plain EVM address**<br><img src="../data/payments/evm-address-selection.png" width="180" alt="EVM address QR code"><br>`0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326`<br>Select an asset when multiple EVM chains match, then open the amount screen. | **Ethereum USDC**<br><img src="../data/payments/ethereum-usdc.png" width="180" alt="Ethereum USDC QR code"><br>`ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48@1/transfer?address=0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326&uint256=1500000`<br>Confirm `1.5 USDC`. |
| **Solana USDC**<br><img src="../data/payments/solana-usdc.png" width="180" alt="Solana USDC QR code"><br>`solana:HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5?amount=1&spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`<br>Confirm `1 USDC`. | **XRP destination tag**<br><img src="../data/payments/xrp-destination-tag.png" width="180" alt="XRP destination tag QR code"><br>`ripple:rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh?amount=10&dt=12345`<br>Confirm amount `10` with tag `12345`. |
| **Uppercase parameter keys**<br><img src="../data/payments/bitcoin-uppercase-keys.png" width="180" alt="Bitcoin uppercase parameter QR code"><br>`bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?AMOUNT=0.001`<br>Confirm `0.001 BTC`; the key case is ignored. | **Bitcoin address-less URI**<br><img src="../data/payments/bitcoin-address-less.png" width="180" alt="Bitcoin address-less QR code"><br>`bitcoin:?bc=bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4&amount=0.001`<br>Confirm `0.001 BTC` from the `bc` instruction. |
| **TON comment**<br><img src="../data/payments/ton-comment.png" width="180" alt="TON comment QR code"><br>`ton://transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA?amount=1000000000&text=order+7`<br>Confirm `1 TON` with comment `order 7`. | **Excess BTC precision**<br><img src="../data/payments/bitcoin-too-precise.png" width="180" alt="Bitcoin excessive precision QR code"><br>`bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.000000001`<br>Do not round; the amount is dropped and entered on the amount screen. |

**Partially specified payments** open the recipient screen when a required memo is missing or the address does not fit the asset, the amount screen otherwise.

| | |
|---|---|
| **XRP without destination tag**<br><img src="../data/payments/xrp-amount-only.png" width="180" alt="XRP amount-only QR code"><br>`ripple:rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh?amount=10`<br>Amount `10` is preserved; add a destination tag only if required. | **XRP without amount**<br><img src="../data/payments/xrp-tag-only.png" width="180" alt="XRP destination-tag-only QR code"><br>`ripple:rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh?dt=12345`<br>Tag `12345` is preserved; enter the amount. |

**Scanner rendering**: a code must be read however it is drawn, from the camera and from a picked image alike.

| | |
|---|---|
| **Light on dark**<br><img src="../data/payments/bitcoin-address-only-inverted.png" width="180" alt="Inverted Bitcoin address QR code"><br>`bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4`<br>WalletConnect and dark-mode screenshots draw the code this way; open the amount screen. | |

</details>
