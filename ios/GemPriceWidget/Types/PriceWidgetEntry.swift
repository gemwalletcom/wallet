// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Primitives
import Style
import WidgetKit

struct PriceWidgetEntry: TimelineEntry {
    let date: Date
    let coinPrices: [CoinPrice]
    let error: String?

    init(
        date: Date,
        coinPrices: [CoinPrice],
        error: String? = .none,
    ) {
        self.date = date
        self.coinPrices = coinPrices
        self.error = error
    }

    static func error(error: String) -> PriceWidgetEntry {
        PriceWidgetEntry(
            date: Date(),
            coinPrices: [],
            error: error,
        )
    }

    static func placeholder(widgetFamily: WidgetFamily = .systemMedium) -> PriceWidgetEntry {
        let placeholderCoins = [
            CoinPrice.placeholder(chain: .bitcoin, name: "Bitcoin", symbol: "BTC", price: 69000, change: 2.5),
            CoinPrice.placeholder(chain: .ethereum, name: "Ethereum", symbol: "ETH", price: 3500, change: 1.2),
            CoinPrice.placeholder(chain: .solana, name: "Solana", symbol: "SOL", price: 150, change: -0.8),
        ]
        return PriceWidgetEntry(
            date: Date(),
            coinPrices: widgetFamily == .systemSmall ? Array(placeholderCoins.prefix(1)) : placeholderCoins,
            error: .none,
        )
    }
}

private extension CoinPrice {
    static func placeholder(chain: Chain, name: String, symbol: String, price: Double, change: Double) -> CoinPrice {
        CoinPrice(
            assetId: AssetId(chain: chain, tokenId: nil),
            name: name,
            symbol: symbol,
            priceText: price.formatted(.currency(code: "USD").precision(.fractionLength(2))),
            changeText: PercentFormatter().string(change),
            changeTone: WidgetValueTone(change: change),
            image: Images.name(chain.rawValue),
        )
    }
}
