// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSwapErrorDisplay
import Primitives
import PrimitivesTestKit
@testable import Swap
import Testing

struct GemSwapErrorDisplayTests {
    @Test
    func minimumAmountMessage() {
        #expect(
            GemSwapErrorDisplay.minimumAmount(asset: Asset.mockBNB().toGem(), minAmount: 120_966_091_866_986).errorDescription ==
                "Minimum trade amount is **0.0001209 BNB**. Please enter a higher amount.",
        )
        #expect(
            GemSwapErrorDisplay.minimumAmount(asset: Asset.mock(symbol: "USDT", decimals: 6).toGem(), minAmount: 123_456).errorDescription ==
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
