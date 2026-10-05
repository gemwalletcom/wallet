# Product behavior

This is the high-level definition of how the wallet behaves, from the user's side. The principles below apply to every screen, and so do the shared sections after them: numbers, asset logos, the Network Fee, names, charts, secrets and feature availability. Each area has its own short page under [product/](product/): the success path as steps, a diagram where it reads faster than words, the expected results and the platform differences as tables, and the few rules no table can hold. How the code delivers it lives in [ARCHITECTURE.md](ARCHITECTURE.md); the gaps live in [TODO.md](TODO.md).

Before changing how an area works, read its page. A change that breaks a rule written there is a product decision, not a cleanup: ask first, and update the page in the same change when the intent moves.

## Principles

**Secure.** The user always sees what they are about to sign: amount, recipient, network and fee. Nothing is signed or sent without that screen. A Secret Phrase or private key never leaves the device's secure storage and never reaches a log, a screenshot or an unprotected clipboard. When something security-related is missing or unsupported, the wallet refuses instead of guessing.

**Fast.** Whatever the app already knows shows at once; a refresh updates it in place and never blanks the screen. Independent requests start together and none waits for a slower one. When one source fails, the others still show; the failed one keeps its last known values.

**Native.** Each app looks and feels like a first-class app of its platform: system navigation, sheets, share, paste, biometrics and the user's locale for numbers and dates. Both apps follow the same rules and make the same decisions; only the drawing is platform-specific. Where the apps differ, the area's Platform differences table says so and why; any other difference is a bug.

**Simple.** Every screen uses patterns the user already knows: a list of rows, a sheet, a segmented picker, a button that says what it does. The same thing looks and works the same everywhere in the app: one asset row, one address row, one confirmation screen, one way to copy, one way to pick an asset. A new screen copies an existing one rather than inventing a layout. Each screen asks for one decision at a time, shows the outcome where the action happened, and needs no explanation to use. If something needs a tooltip to be understood, the screen is wrong, not the user.

**Smooth.** Scrolling, typing and transitions hold the frame rate. The app never freezes or flickers. A price update changes only the rows it touches, and a result for a wallet or asset the user has already left is dropped, not drawn. A visible hitch, a flash of empty content, or a glimpse of another wallet's data is a defect.

## Look

**Feature symbol.** A screen or sheet that introduces one feature or asks for one decision shows one large symbol from the platform's own set, in the brand blue with no background, above a bold title and a short description, at the size of an info sheet. Every such screen uses this one look so the user recognizes it; a new one copies it rather than drawing an icon.

## Numbers

Both apps write every number the same way. The examples are in US English and US dollars; separators and where the currency symbol goes follow the user's locale.

**Money**

| When | Expected | Why |
|---|---|---|
| A value of $0.99 or more: a fee, a price, a balance's value | two places: `$1,234.50`, `$12.30`, `$1.00` | |
| A value under $0.99 | at least two places, up to four significant digits: `$0.90`, `$0.50`, `$0.1235`, `$0.0039`, `$0.00000783` | money always shows its cents, so it never reads `$0.9`; a small price keeps the digits that tell it apart from zero |
| A value of zero, or under $0.0000000001 | `$0.00` | |
| A value with more digits than it shows | rounded to the nearest: `$1.89999` reads `$1.90` | |
| A gain or a loss | with its sign: `+$5.00`, `-$1.20`, `+$0.90` | |
| A price or value in a list row: the wallet's assets, an asset list, stake, perpetual markets | as above, and under $0.0001 `<$0.0001` | a row has no room for `$0.00000783` |
| The wallet total and its gain or loss, a Buy or Sell amount, a perpetual's margin, the medium and large price widgets | always two places: `$1,234.56`, `$0.90`, and `$0.00` under half a cent | a total or a payment is counted in cents |
| Market cap, fully diluted valuation, trading volume, the small price widget | from $100,000 shortened: `$1.24M`, `$2.45B`; under it as above: `$99,999.00` | |

**Amounts**

| When | Expected | Why |
|---|---|---|
| An amount of 1 or more | up to two places, without trailing zeros: `5.2 ATOM`, `1 ATOM`, `1,239,999 ATOM` | an amount is not money; `5.20 ATOM` adds nothing |
| An amount under 1 | up to four significant digits: `0.1992 ATOM`, `0.09999 ATOM`, `0.00000546 ATOM` | |
| An amount with more digits than it shows | cut, never rounded up: `0.099999 ATOM` reads `0.09999 ATOM` | the wallet never shows more than the user has |
| A balance change or an approval amount on Confirm | every digit: `1,234.567891 ETH`, `0.000021 ETH` | the user sees exactly what they sign |
| An amount in a list row: the wallet's assets, activity, stake, supply | from 0.1 up to two places: `5.2 BTC`, `0.5 BTC`; under 0.1 up to four places: `0.0345 BTC`; under 0.0001 `<0.0001 BTC`; from 100,000 shortened: `267.12K BTC` | a row has room for a short number only |

**Percentages and other numbers**

| When | Expected | Why |
|---|---|---|
| A change, a gain or a loss: price change, profit and loss, funding rate | two places with its sign: `+2.50%`, `-2.00%` | |
| A rate: APR, slippage | two places: `5.00%`, `12.50%`, `0.08%` | |
| A preset the user picks, such as a price alert step | up to two places: `5%`, `2.5%` | |
| A gas price in Gwei, an exchange rate, a chart candle's price | two places from 0.99: `24.00`, `3,456.79`; under it up to four significant digits: `0.1235`, `0.0008382` | |
| Leverage | up to two places: `5x`, `2.5x` | |

## Asset logos

Every list, header and sheet draws an asset's logo the same way. The logos that ship with the app show at once; any other logo is loaded.

| When the asset is | Expected | Why |
|---|---|---|
| A network's coin | its logo, at once | |
| ETH on an Ethereum layer 2, such as Base | the Ether logo with the network as a badge | the coin is ETH; the badge tells the networks apart |
| A token | its logo with its network as a badge | |
| USDT, USDC, USDS, USDe, USD1, USDG or PYUSD on a network its issuer deploys it to; USDT0 counts as USDT | its logo, at once | these are the tokens most users hold, so they never wait for a logo |
| USDT or USDC on BNB Smart Chain | its logo, at once | it is the USDT and USDC users hold on BNB Smart Chain, though Binance issues it |
| Any other token, bridged copies included, such as `USDC.e` | its logo once loaded, and its token standard, such as `ERC20`, until then | only a logo that ships with the app can show at once |

## Network Fee

Confirm and Transaction details show the Network Fee the same way.

| When | Expected | Why |
|---|---|---|
| The fee's coin has a price | Network Fee shows its value only: `$0.01` | the value is the number the user weighs |
| The fee's coin has no price | the fee in that coin: `0.000021 ETH` | |
| There is not enough of the coin that pays the fee | the fee in that coin above its value: `0.0000129 BNB` over `$0.01` | that is the amount the user has to add |
| The user can pick which asset pays the fee | its value and the asset: `$0.01` with `USDC` | |
| The user taps Network Fee on Confirm | the fee in its coin above its value, and faster, slower or custom fees where the network allows | |
| Confirm refreshes while the user types a custom fee | the typed value stays until they confirm it or leave the custom fee screen | a refresh never takes back what the user typed |
| The fee rows are built on an EVM network | the price that is signed: the network's current base fee with 20% headroom, plus a tip | the base fee can rise by up to 12.5% per block and Confirm refreshes once a minute, so without headroom a transaction that misses the next block gets stuck; the user pays only the actual base fee, so the headroom is never charged |
| The user sets a custom fee on an EVM network | two fields in Gwei that open empty with what Normal signs as the placeholder: Base Fee, the most they agree to pay the network, never under the base fee Normal signs; and Priority Fee, the tip, never under the network's minimum tip; an empty field means the suggested value and keeps following the network after a refresh, only typed values are kept; each field is capped at ten times the Normal price; a network whose base fee is zero (BNB Chain) shows only the Priority Fee field | a custom fee never has less headroom than Normal, so it does not get stuck more easily, and the tip is what makes it faster |
| Transaction details show the fee | as on Confirm: `$0.01`, or without a price `0.000021 ETH`; a tap shows `0.000021 ETH` over `$0.01` | the fee the user approved reads the same afterwards |

## Names

Wherever the user types an address (Send, Import Wallet, Contacts), a name such as `vitalik.eth` can stand in for it.

| When | Expected | Why |
|---|---|---|
| A typed name is not registered, or has no address for the network | not found | a name never becomes an empty or zero address |
| A `.sol` name is typed while SRS resolution is paused | not found | `.sol` is never guessed from `.sns`; the two names can have different owners |
| The name service cannot be reached | an error, not a missing name | |

## Charts

| When | Expected | Why |
|---|---|---|
| The price chart, a portfolio chart or the price widget cannot load | it shows that there is no data; only being offline shows an error | server text is not written for users |
| The user pinches the price chart or a market's candlestick chart | it zooms toward the newest point while that point is on screen, and around the fingers once panned back, down to 14 points on screen; a zoomed chart pans back in time with a swipe; pinching out stops at the whole period | the newest point is what the user watches, and fewer points stop reading as a trend |

## Secrets

A Secret Phrase or private key shows when a wallet is created and from "Show Secret Phrase" or "Show Private Key".

| When | Expected | Why |
|---|---|---|
| The user copies the Secret Phrase or private key | it leaves the clipboard after one minute and is kept off other devices; whatever the user copies after it stays | only a secret is dangerous to leave behind |
| The user copies an address or anything else | it stays until something replaces it | only a secret is dangerous to leave behind |

| When | iOS | Android | Expected |
|---|---|---|---|
| The user takes a screenshot of the Secret Phrase or private key screen | the screenshot is detected and the user is warned | the screenshot is blocked | Intentional |

- The Secret Phrase and private key screens hide their content during screen recording and whenever the app is not active.

## Feature availability

If an action is unavailable, show the Not Available sheet and stop. Learn More opens [regional availability](https://docs.gemwallet.com/faq/feature-availability/).

| Feature | Show the sheet when |
|---|---|
| Buy | Continue on the Buy screen, before opening the provider's checkout |
| Sell | Continue on the Sell screen, before opening the provider's checkout |
| Swap | Continue / Swap on the Swap screen, before preparing the trade |
| Perpetuals | Deposit, Long, Short, Increase or Reduce before the amount screen; Modify before the options |
| Staking | Stake on the staking screen, before opening the amount screen |
| Rewards | Invite Friends, before opening the share sheet |

Load config at startup and use the stored flags for actions. Features default to enabled until config is loaded. Keep quotes visible, and Withdraw and Close available.

## Areas

- [Onboarding](product/onboarding.md) — create a wallet, import a wallet, and what happens right after
- [Wallet](product/wallet.md) — the wallet screen, balances, search, and the wallets list: switching, pinning, rename, avatar, secrets, delete
- [Assets](product/assets.md) — the asset screen with its price chart and market data, manage tokens, recent assets
- [Transfer](product/transfer.md) — send, recipient, amount, confirm, and scanned payment codes and links
- [Transactions](product/transactions.md) — activity, filters, transaction details, pending tracking
- [Swap](product/swap.md) — quotes, providers, slippage, price impact, approval
- [Buy and Sell](product/fiat_connect.md) — buy and sell through providers
- [Stake](product/stake.md) — validators, stake, unstake, rewards, earn
- [Perpetuals](product/perpetuals.md) — markets, positions, leverage, take profit and stop loss
- [NFT](product/nft.md) — collections, an NFT's details and actions, unverified collections
- [WalletConnect](product/wallet_connector.md) — connecting dapps, requests, signing, WalletConnect Pay
- [Price Alerts](product/price_alerts.md) — alerts on an asset's price
- [In-app Notifications](product/in_app_notifications.md) — the notifications list in Settings
- [Settings](product/settings.md) — preferences, security and lock, push notifications, networks, about, app update
- [Contacts](product/contacts.md) — saved addresses and the recipient picker
- [Rewards](product/rewards.md) — referral codes, points, redeeming
- [Support](product/support.md) — support chat and help

## Writing a page

One page per area, named after the area's feature module on both apps (`product/<android_module>.md`, [Cross-Platform Awareness rule 7](../skills/cross-platform-awareness.md)). Use the product's own names from the app strings. No internals, no code, no test names. A page has, in order:

1. One sentence on what the area is for, and at most two diagrams of the main flow with one-line labels.
2. **Steps:** the success path as numbered user stories, one line each.
3. **Expected results:** a `When | Expected | Why` table, one row per situation, with example values in the app's wording (`$0.01`, `0.000021 ETH`, "Not supported"). A rule someone might simplify away becomes a row, and its reason goes in Why.
4. **Platform differences:** a `When | iOS | Android | Expected` table with every recorded difference in the area; an intentional one says Intentional, an open one names the [TODO.md](TODO.md) item that removes it, and a page without any says "None recorded."
5. **Rules:** only the always and never invariants that no single situation triggers, one line each with its reason.

A rule that holds in more than one area lives once in a shared section of this page, and each area points to it instead of repeating it. A page with more than one flow gives each flow its own steps and table. Transfer ([transfer.md](product/transfer.md)) is the worked example. Edge cases and open questions stay in [TODO.md](TODO.md).
