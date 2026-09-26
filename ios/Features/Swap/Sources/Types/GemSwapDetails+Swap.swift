// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemFormattedNumber
import struct Gemstone.GemSwapDetails
import PrimitivesComponents

public extension GemSwapDetails {
    func rateText(isInverse: Bool) -> String? {
        summary.rate.map { AssetRateViewModel(rate: $0).text(isInverse: isInverse) }
    }

    var summaryPriceImpact: GemFormattedNumber? {
        summary.priceImpactRow.flatMap { $0.showsInSummary ? $0.value : nil }
    }

    var priceImpactWarning: String? {
        summary.priceImpactRow?.warning?.text
    }
}
