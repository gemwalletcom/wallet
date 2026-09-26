// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSwapDetails
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
@testable import Swap
import SwapTestKit
import Testing

@MainActor
struct GemSwapDetailsTests {
    @Test
    func switchRate() {
        let model = GemSwapDetails.mock(selectedQuote: .mock(fromValue: 1_000_000_000_000_000_000, toValue: 250_000_000_000, slippageBps: 50))

        #expect(model.rateText(isInverse: false) == "1 ETH ≈ 250,000.00 USDT")
        #expect(model.rateText(isInverse: true) == "1 USDT ≈ 0.000004 ETH")
    }
}
