# Product behavior

This is the high-level definition of how the wallet behaves, from the user's side. The principles below apply to every screen. Each area has its own short page under [product/](product/): the success path as steps, a diagram where it reads faster than words, the expected results and the platform differences as tables, and the few rules no table can hold. How the code delivers it lives in [ARCHITECTURE.md](ARCHITECTURE.md); the gaps live in [TODO.md](TODO.md).

Before changing how an area works, read its page. A change that breaks a rule written there is a product decision, not a cleanup: ask first, and update the page in the same change when the intent moves.

## Principles

**Secure.** The user always sees what they are about to sign: amount, recipient, network and fee. Nothing is signed or sent without that screen. A Secret Phrase or private key never leaves the device's secure storage and never reaches a log, a screenshot or an unprotected clipboard. When something security-related is missing or unsupported, the wallet refuses instead of guessing.

**Fast.** Whatever the app already knows shows at once; a refresh updates it in place and never blanks the screen. Independent requests start together and none waits for a slower one. When one source fails, the others still show; the failed one keeps its last known values.

**Native.** Each app looks and feels like a first-class app of its platform: system navigation, sheets, share, paste, biometrics and the user's locale for numbers and dates. Both apps follow the same rules and make the same decisions; only the drawing is platform-specific. Where the apps differ, the area's Platform differences table says so and why; any other difference is a bug.

**Simple.** Every screen uses patterns the user already knows: a list of rows, a sheet, a segmented picker, a button that says what it does. The same thing looks and works the same everywhere in the app: one asset row, one address row, one confirmation screen, one way to copy, one way to pick an asset. A new screen copies an existing one rather than inventing a layout. Each screen asks for one decision at a time, shows the outcome where the action happened, and needs no explanation to use. If something needs a tooltip to be understood, the screen is wrong, not the user.

**Smooth.** Scrolling, typing and transitions hold the frame rate. The app never freezes or flickers. A price update changes only the rows it touches, and a result for a wallet or asset the user has already left is dropped, not drawn. A visible hitch, a flash of empty content, or a glimpse of another wallet's data is a defect.

## Areas

- [Onboarding](product/onboarding.md) — create a wallet, import a wallet, and what happens right after
- [Wallet](product/wallet_tab.md) — the wallet screen, balances, search
- [Assets](product/assets.md) — the asset screen, manage tokens, recent assets
- [Market](product/market.md) — an asset's price chart and market data
- [Wallets](product/wallets.md) — the wallets list, switching, pinning, rename, avatar, secrets, delete
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

A page with more than one flow gives each flow its own steps and table. Transfer ([transfer.md](product/transfer.md)) is the worked example. Edge cases and open questions stay in [TODO.md](TODO.md).
