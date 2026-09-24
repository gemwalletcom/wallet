// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.formattedAmount
import enum Gemstone.GemSwapErrorDisplay
@testable import Swap
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

    @Test
    func userFacingMessages() {
        #expect(GemSwapErrorDisplay.notSupportedAsset.errorDescription == "Not supported asset.")
        #expect(GemSwapErrorDisplay.noQuote.errorDescription == "No quote available.")
        #expect(GemSwapErrorDisplay.offline.errorDescription == "The Internet connection appears to be offline.")
        #expect(GemSwapErrorDisplay.amountTooSmall.errorDescription == "Amount too small")
    }
}
