// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import struct Gemstone.GemAssetRate
import struct Gemstone.GemSwapRate
@testable import PrimitivesComponents
import Testing

struct AssetRateViewModelTests {
    @Test
    func textShowsOneBaseUnitInEachDirection() {
        let model = AssetRateViewModel(
            rate: GemSwapRate(
                direct: GemAssetRate(baseSymbol: "ETH", quoteSymbol: "USDT", value: 250_000),
                inverse: GemAssetRate(baseSymbol: "USDT", quoteSymbol: "ETH", value: 0.000004),
            ),
            formatter: NumericFormatter(locale: Locale(identifier: "en_US")),
        )

        #expect(model.text(isInverse: false) == "1 ETH ≈ 250,000.00 USDT")
        #expect(model.text(isInverse: true) == "1 USDT ≈ 0.000004 ETH")
    }
}
