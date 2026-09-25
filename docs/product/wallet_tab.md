# Wallet

The home screen: what the selected wallet holds, what it is worth, and the way into every action.

## Wallet screen

- The screen opens instantly with what the app already knows: wallet name, total with its 24h change, the action buttons, the asset rows with balance, price and value, and any banner.
- The wallet and asset screens show at most one banner, the most important one; closing it brings up the next.
- Send, Receive and Buy show for every wallet that can sign; Swap where a swap is possible; a watch-only wallet shows a notice instead of buttons, on the wallet and asset screens and in an empty transaction list.
- Pinned assets come first, then the rest by value; a native coin is titled by its network.
- One switch hides balances everywhere.
- Pull-to-refresh fetches balances, discovers new tokens, and loads new transactions and NFTs, all as separate requests at the same time; prices arrive live, with no timer.
- Switching wallet rebuilds the screen for the new wallet.
- One search covers assets, perpetuals and NFTs, grouped, and an asset the wallet does not have can be added from the result.

## Balances

```mermaid
sequenceDiagram
    participant Screen as Wallet screen
    participant App
    participant Chain as Each network
    Screen->>App: refresh
    par every network at once
        par coin
            App->>Chain: coin balance
        and staking
            App->>Chain: staking balance
        and tokens
            App->>Chain: token balances
        and earn
            App->>Chain: earn balances
        end
        App-->>Screen: that network's rows update as soon as it has answered
    end
```

- Every network is asked at the same time, and on each network the coin, staking, token and earn balances are separate requests, so the fastest answer is never held back by the slowest.
- Each network updates the list as soon as it has answered, in one write for that network, so the fastest network shows first; unchanged rows are not touched.
- A request that fails holds nothing back: the other balances of that network still update, and a network or balance that did not answer keeps its last values.
- Prices are kept in USD and converted once to the chosen currency, so every screen shows the same value.

## Rules

- Coin, staking, token and earn balances are separate requests; never merge them into one, because one slow or failing request must not hold back the others.
- A network that fails keeps its last values on screen; there is no "unknown" state.
- An answer always belongs to the wallet it was requested for, never the wallet on screen.
- A portfolio chart, or the price widget, that cannot load shows that there is no data, and only being offline shows an error, because server text is not written for users.
