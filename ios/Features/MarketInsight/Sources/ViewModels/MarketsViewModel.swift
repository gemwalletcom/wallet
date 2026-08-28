// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Style

struct MarketsViewModel {
    let markets: Markets

    private let currencyCode: String

    init(
        markets: Markets,
        currencyCode: String,
    ) {
        self.markets = markets
        self.currencyCode = currencyCode
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
                currencyCode: currencyCode,
            ),
        )
    }
}
