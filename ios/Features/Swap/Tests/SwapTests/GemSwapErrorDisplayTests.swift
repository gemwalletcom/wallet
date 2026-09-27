// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.formattedAmount
import enum Gemstone.GemSwapErrorDisplay
import Testing

struct GemSwapErrorDisplayTests {
    @Test
    func minimumAmountMessage() {
        #expect(
            GemSwapErrorDisplay.minimumAmount(minimum: formattedAmount(value: 0.000120966091866986, symbol: "BNB", style: .auto)).errorDescription ==
                "Minimum trade amount is **0.0001209 BNB**. Please enter a higher amount.",
        )
        #expect(
            GemSwapErrorDisplay.minimumAmount(minimum: formattedAmount(value: 0.123456, symbol: "USDT", style: .auto)).errorDescription ==
                "Minimum trade amount is **0.1234 USDT**. Please enter a higher amount.",
        )
    }
}
