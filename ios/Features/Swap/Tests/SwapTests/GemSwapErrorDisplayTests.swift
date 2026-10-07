// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemSwapErrorDisplay
import Testing

struct GemSwapErrorDisplayTests {
    @Test
    func minimumAmountMessage() {
        #expect(
            GemSwapErrorDisplay.minimumAmount(minimum: minimum(0.000120966091866986, symbol: "BNB")).errorDescription ==
                "Minimum trade amount is **0.0001209 BNB**. Please enter a higher amount.",
        )
        #expect(
            GemSwapErrorDisplay.minimumAmount(minimum: minimum(0.123456, symbol: "USDT")).errorDescription ==
                "Minimum trade amount is **0.1234 USDT**. Please enter a higher amount.",
        )
    }

    private func minimum(_ value: Double, symbol: String) -> GemFormattedNumber {
        GemFormattedNumber(value: value, unit: .symbol(symbol: symbol), display: .number(precision: .significant(max: 4)), notation: .plain, tone: .plain, rounding: .towardZero, exact: nil)
    }
}
