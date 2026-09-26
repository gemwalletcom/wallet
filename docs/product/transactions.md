# Transactions

The Activity tab: everything the wallet sent, received, swapped or staked, grouped by day, with every pending transaction tracked until the network settles it.

```mermaid
flowchart LR
    A[Transaction sent] --> B[Pending] --> C{Network result}
    C --> D[Successful]
    C --> E[Failed or Reverted]
    B --> F[Push notification] --> G[Transaction details]
```

- Activity lists transactions newest first under Today, Yesterday or the date, each with its type (Sent, Received, Swap, Stake and so on), the asset, the amount and its value, and a Pending badge while it is unconfirmed; filters narrow by type and network.
- A pending transaction is tracked in the background, even after the app is closed, and its status and the balances update when the network confirms it.
- Transaction details show the header with the amount and value, Date, Status, Recipient or Sender, the network fee, a memo when there is one, and "View on" the explorer; a swap shows its progress and offers Swap Again.
- A push about a transaction opens its details.
- An empty wallet reads "Your activity will appear here. Make your first transaction".

```mermaid
flowchart LR
    A[Activity] --> B[Filter by type or network] --> C[Row] --> D[Transaction details] --> E[View on explorer]
    D --> F[Swap Again]
```

## Rules

- A sent transaction appears as Pending at once and stays tracked until the network settles it, whatever screen the user is on.
- Amounts and status come from the network, never from the app's guess; a failed or reverted transaction is shown as such.
- An asset's transactions are the ones that move that asset, so a swap is listed under both the asset paid and the asset received, because the user looks for it under either.
- Tapping an address opens its page with the name the user already sees for it, so the address is recognisable: a contact or own wallet name comes first, a contact shows its picture or initials as on the confirm screen, and a token or validator shows its logo. The page also shows the full address, copied with a tap, the type and the balances; a flagged address shows a "Suspicious address" warning under its picture, the same warning as on the confirm screen, even when it is a contact, and only a plain address shows balances: a contract, token or validator shows none, and the backend does not fetch them.
