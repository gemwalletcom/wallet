# Onboarding

Getting a wallet into the app: create a new one, or import one with a Secret Phrase, a private key or an address.

## Create wallet

```mermaid
flowchart LR
    A[Accept Terms] --> B[Security reminder] --> C[Secret Phrase] --> D[Secret Phrase verification] --> E[Wallet created] --> F[Wallet screen]
```

1. The user accepts the terms and reads the security reminder.
2. The user sees a new 12-word Secret Phrase in two numbered columns and can copy it.
3. The user verifies it on the "Confirm" screen by tapping the words back in order, shuffled inside groups of four.
4. The wallet is created, named "Wallet #N", selected, and the wallet screen opens.

| When | Expected | Why |
|---|---|---|
| The terms were already accepted on this install | they are not asked again; the security reminder still shows on every create | |
| The user copies the Secret Phrase | the copy expires after one minute | |

## Import wallet

```mermaid
flowchart LR
    A[Import Wallet] --> B[Multi-Coin or a network] --> C[Secret Phrase, private key or address] --> D{Already in the app?}
    D -- yes --> E[Opens the existing wallet]
    D -- no --> F[Wallet imported] --> G[Wallet screen]
```

1. The user taps Import Wallet and picks Multi-Coin or a network.
2. The user enters a Secret Phrase, a private key or an address.
3. The wallet is imported, named and selected in one step, and the wallet screen opens.

| When | Expected | Why |
|---|---|---|
| The user picks Multi-Coin | it takes a Secret Phrase | |
| The user picks a single network | it takes a Secret Phrase, a private key where the network supports one, or an address for a watch-only wallet | |
| The user types a Secret Phrase | word suggestions complete the last word, and a tap replaces it | |
| The cursor is inside the phrase | no suggestions show | a tap never changes the wrong word |
| An address is typed as a name | the resolved name becomes the wallet name | |
| The wallet already exists | it is simply opened | |

## App lock offer

```mermaid
flowchart LR
    A[Wallet created or imported] --> B{Offered before, already on, or unavailable?}
    B -- no --> C[Enable Face ID or Passcode, or Skip] --> D[Wallet screen]
    B -- yes --> D
```

| When | Expected | Why |
|---|---|---|
| A wallet is created or imported, the device has biometrics or a passcode, and app authentication is off | before the wallet screen, the app offers to turn it on with Enable and Skip | a new wallet should not stay open to anyone holding the unlocked phone |
| The user taps Enable | the device asks for authentication, then app authentication is on as if turned on in Settings → Security | |
| The authentication prompt is cancelled or fails | the offer stays open | |
| The user taps Skip or goes back | the wallet screen opens | Settings → Security still turns it on |
| The offer was shown once on this install | it is not shown again, whatever was chosen | |
| App authentication is already on, or the device has no biometrics or passcode | no offer | |

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

| When | Expected | Why |
|---|---|---|
| The wallet was created | its default assets, live prices and the welcome banner with Buy and Receive; nothing is fetched until its second refresh | a created wallet has no history |
| The wallet was imported | it fetches its balances, discovers its tokens, and loads its transactions and NFTs, four separate requests at the same time | |
| An imported wallet's token discovery has not completed | a "Loading" row stays above the list | |

## Platform differences

| When | iOS | Android | Expected |
|---|---|---|---|
| The user types an invalid word while importing a Secret Phrase | no per-word highlight | highlights the invalid word | Intentional: a one-sided feature, added to iOS only when required |
| The user takes a screenshot of the Secret Phrase or private key screen | the screenshot is detected and the user is warned | the screenshot is blocked | Intentional |

## Rules

- The Secret Phrase never leaves the device and is never written to a log.
- The Secret Phrase and private key screens hide their content during screen recording and whenever the app is not active.
