# Onboarding

Getting a wallet into the app: create a new one, or import one with a Secret Phrase, a private key or an address.

## Create wallet

```mermaid
flowchart LR
    A[Accept Terms] --> B[Security reminder] --> C[Secret Phrase] --> D[Secret Phrase verification] --> E[Wallet created] --> F[Wallet screen]
```

- The user accepts the terms once per install and reads the security reminder on every create.
- The user sees a new 12-word Secret Phrase in two numbered columns and can copy it; the copy expires after one minute.
- The user verifies it on the "Confirm" screen by tapping the words back in order, shuffled inside groups of four.
- The wallet is created, named "Wallet #N", selected, and the wallet screen opens.

## Import wallet

```mermaid
flowchart LR
    A[Import Wallet] --> B[Multi-Coin or a network] --> C[Secret Phrase, private key or address] --> D{Already in the app?}
    D -- yes --> E[Opens the existing wallet]
    D -- no --> F[Wallet imported] --> G[Wallet screen]
```

- Multi-Coin takes a Secret Phrase; a single network takes a Secret Phrase, a private key where the network supports one, or an address for a watch-only wallet.
- While typing a Secret Phrase, word suggestions complete the last word and a tap replaces it; with the cursor inside the phrase no suggestions show, so a tap never changes the wrong word.
- An address can be typed as a name; the resolved name becomes the wallet name.
- The wallet is imported, named and selected in one step; a wallet that already exists is simply opened.

## After create and import

```mermaid
flowchart TD
    A[Wallet screen opens] --> B{Created here?}
    B -- yes --> C[Default assets at zero, live prices, welcome banner]
    B -- no --> D[Fetch balances]
    B -- no --> E[Discover tokens]
    B -- no --> F[Load transactions]
    B -- no --> G[Load NFTs]
    D --> H[List updates]
    E --> H
    F --> H
    G --> H
```

- A created wallet has no history, so nothing is fetched until its second refresh; it shows its default assets, live prices and the welcome banner with Buy and Receive.
- An imported wallet fetches its balances, discovers its tokens, and loads its transactions and NFTs, four separate requests at the same time; a "Loading" row stays above the list until token discovery completes.

## Rules

- The Secret Phrase never leaves the device and is never written to a log.
- The Secret Phrase and private key screens hide their content during screen recording and whenever the app is not active; on iOS a screenshot is detected and warned about, Android blocks it.
