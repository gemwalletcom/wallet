// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.formattedAdaptive
import struct Gemstone.GemAssetRate
import struct Gemstone.GemSwapRate
import GemstonePrimitives
@testable import PrimitivesComponents
import Testing

struct AssetRateViewModelTests {
    @Test
    func textShowsOneBaseUnitInEachDirection() {
        let model = AssetRateViewModel(
            rate: GemSwapRate(
                direct: GemAssetRate(baseSymbol: "ETH", value: formattedAdaptive(value: 250_000, symbol: "USDT")),
                inverse: GemAssetRate(baseSymbol: "USDT", value: formattedAdaptive(value: 0.000004, symbol: "ETH")),
            ),
            locale: Locale(identifier: "en_US"),
        )

        #expect(model.text(isInverse: false) == "1 ETH ≈ 250,000.00 USDT")
        #expect(model.text(isInverse: true) == "1 USDT ≈ 0.000004 ETH")
    }
}
