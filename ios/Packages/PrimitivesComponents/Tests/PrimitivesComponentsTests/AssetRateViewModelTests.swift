// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemAssetRate
import struct Gemstone.GemSwapRate
import GemstonePrimitives
import GemstonePrimitivesTestKit
@testable import PrimitivesComponents
import Testing

struct AssetRateViewModelTests {
    @Test
    func textShowsOneBaseUnitInEachDirection() {
        let model = AssetRateViewModel(
            rate: GemSwapRate(
                direct: GemAssetRate(baseSymbol: "ETH", value: .mock(value: 250_000, unit: .symbol(symbol: "USDT"), notation: .plain)),
                inverse: GemAssetRate(baseSymbol: "USDT", value: .mock(value: 0.000004, unit: .symbol(symbol: "ETH"), display: .number(precision: .significant(max: 4)), notation: .plain)),
            ),
            locale: Locale(identifier: "en_US"),
        )

        #expect(model.text(isInverse: false) == "1 ETH ≈ 250,000.00 USDT")
        #expect(model.text(isInverse: true) == "1 USDT ≈ 0.000004 ETH")
    }
}
