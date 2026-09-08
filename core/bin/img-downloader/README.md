# Image downloader

Run from `core/`. Providers: `coingecko`, `coinmarketcap`, `jupiter`, and `dexscreener`.

All providers implement `ImageProvider` for ID lookups and `ImageListProvider` for top and trending lists. The downloader selects the requested operation before starting a download.

## Provider contract

- `ImageProvider::get_asset_images(id)` uses the selected provider's identifier and returns zero or more token images. A successful response with no usable matching logo returns an empty list; request failures propagate as errors.
- `ImageListProvider: ImageProvider` adds top and trending selections. Implement it only when both selections have an intentional data source; do not add unsupported-method stubs.
- An explicit ID takes precedence over list mode. Without an ID, provider selection requires list support before creating output files.
- Providers fetch metadata and map it to `AssetImage`. The downloader owns address formatting, output paths, existing-image checks, image conversion, and compression.
- Top and trending are provider-specific selections, not a common ranking algorithm. Paid boosts must be identified explicitly if added as a source.

| Provider | ID | Top source | Trending source |
| --- | --- | --- | --- |
| CoinGecko | Coin ID | Markets ordered by market cap | Search trending |
| CoinMarketCap | Coin ID or symbol | Latest listings | Latest trending |
| Jupiter | Token mint | Verified-token list, limited by count | Top trending for the configured interval, filtered to verified tokens |
| DexScreener | Gem asset ID (`chain_token-address`) | Tokens from trending metas ranked by 24-hour volume | Tokens from trending metas ranked by 24-hour volume |

## DexScreener

Fetch a missing token logo by Gem asset ID (`<chain>_<token-address>`):

```sh
cargo run --package img-downloader -- --source dexscreener \
  --id robinhood_0x15d36b6a28d8327abc7afabf0f106ae2c9af5c4d \
  --folder ../../assets/blockchains
```

Uses the public [token-pairs API](https://docs.dexscreener.com/api/reference) without an API key. Use Gem chain names, such as `smartchain` for BSC and `avalanchec` for Avalanche C-Chain.

List modes fetch `/metas/trending/v1`, then `/metas/meta/v1/{slug}` for each distinct meta. Both modes use that candidate set; they do not represent a global token ranking. Requests are spaced one second apart.

Configuration in `config.yml`:

```yaml
dexscreener:
  top:
    count: 50
  trending:
    count: 50
```

Top and trending both rank tokens by 24-hour volume, with separate counts. There are no minimum market cap, liquidity, or volume filters. The pool with the most known liquidity supplies each token's volume and image; pools without liquidity data remain eligible. Duplicate meta entries do not multiply activity. Liquidity, then chain/address, break ranking ties. Tokens without daily volume or a usable image are excluded. The count is applied after deduplication. Explicit ID lookups do not use this ranking.

Keep these controls in `config.yml`.

Only a matching chain and base-token address can supply the logo. Missing logo metadata or unmatched tokens return no images, consistent with the other providers. Existing logos are preserved. Images use the shared PNG conversion, 256×256 resizing, compression, and address formatting pipeline.

In the assets repository, select `dexscreener` in **Download Provider Asset**. Leave ID empty and select `top` or `trending` for lists, or supply one asset ID or comma-separated IDs for explicit lookups. The wallet change must be merged before running that workflow, because it checks out the wallet default branch.
