// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Style

struct MarketsViewModel {
    let markets: Markets

    private let currencyFormatter: CurrencyFormatter

    init(
        markets: Markets,
        currencyFormatter: CurrencyFormatter = CurrencyFormatter(currencyCode: Preferences.standard.currency),
    ) {
        self.markets = markets
        self.currencyFormatter = currencyFormatter
    }

    var marketCapViewModel: PriceListItemViewModel {
        PriceListItemViewModel(
            title: Localized.Asset.marketCap,
            model: PriceViewModel(
                price: Price(
                    price: Double(markets.marketCap),
                    priceChangePercentage24h: Double(markets.marketCapChangePercentage24h),
                    updatedAt: .now,
                ),
                currencyCode: currencyFormatter.currencyCode,
            ),
        )
    }
}
