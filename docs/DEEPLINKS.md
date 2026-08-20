# Deep links

A deep link opens a screen inside the app. The same paths are served under two schemes: `gem://` is handled only by the app, `https://gemwallet.com/` is a universal link on iOS and an Android App Link.

## Supported links

| Screen | Path | Parameters |
|---|---|---|
| Asset | `/tokens/{chain}[/{token_id}]` | Asset is required |
| Perpetuals | `/perpetuals` | — |
| Rewards | `/rewards?code={code}`, `/join/{code}` | Referral code is optional |
| Receive | `/receive[/{chain}[/{token_id}]]` | Without an asset the receive asset list opens |
| Buy | `/buy/{chain}[/{token_id}]?amount={fiat}` | Asset is required, amount is optional |
| Sell | `/sell/{chain}[/{token_id}]?amount={fiat}` | Asset is required, amount is optional |

Examples:

```
gem://receive
gem://receive/bitcoin
gem://buy/bitcoin?amount=100
gem://sell/ethereum/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48?amount=49
https://gemwallet.com/buy/solana?amount=25
```

`amount` is a fiat amount in whole USD, not a crypto amount. Values that are not a positive whole number, including fractional ones like `49.5`, are ignored and the screen opens with its default amount. A locale segment in front of the path is accepted and skipped, so `https://gemwallet.com/zh-cn/buy/bitcoin` resolves like `https://gemwallet.com/buy/bitcoin`. A link with an unknown path, an unknown chain, or a missing required asset is not a deep link and opens in the browser.

Sell also requires the asset to be sellable. A sell link for an asset without sell support falls back to buy on the same screen.

## Links in the support chat

Support chat messages are parsed in Core, and a link whose URL is a deep link is rendered as an in-app row and opens inside the app instead of the browser. This is the path chat agents use: emit a normal markdown link with a `gem://` or `https://gemwallet.com/` URL and the app routes it.

```markdown
[Buy Bitcoin](gem://buy/bitcoin?amount=100)
```

## Implementation

- [Deep link parsing and building](../core/crates/primitives/src/deeplink.rs)
- [URL action routing](../core/crates/primitives/src/url_action.rs) — WalletConnect and deep links are routed before payments
- [UniFFI bridge](../core/gemstone/src/deeplink.rs)
- [Support message links](../core/crates/support/src/text.rs)
- iOS: [`NavigationHandler`](../ios/Gem/Navigation/NavigationHandler.swift), schemes and associated domains in `ios/Gem/Resources/Info.plist` and `ios/Gem/Resources/Gem.entitlements`
- Android: [`WebDeepLinks`](../android/app/src/main/kotlin/com/gemwallet/android/WebDeepLinks.kt), intent filters in `android/app/src/main/AndroidManifest.xml`

## Web requirements

`https://gemwallet.com/` links only reach the app when the website serves them and publishes the app association files. Both platforms verify the association from the domain, so a new path works as a web link before it works as an app link.

- `https://gemwallet.com/.well-known/apple-app-site-association` must list the paths for the iOS app
- `https://gemwallet.com/.well-known/assetlinks.json` must list the Android package and signing certificate
- The site should serve `/receive`, `/buy/*`, and `/sell/*` so the link is not a dead page for people without the app
