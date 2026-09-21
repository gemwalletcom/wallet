// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.SwapperQuote
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Swap
import SwapTestKit
import Testing

@MainActor
struct SwapDetailsViewModelTests {
    @Test
    func swapEstimationField() {
        #expect(
            SwapDetailsViewModel
                .mock(selectedQuote: SwapperQuote.mock(etaInSeconds: nil).swapQuote).swapEstimationField == nil,
        )
        #expect(SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(etaInSeconds: 30).swapQuote).swapEstimationField?.value.text == "≈ 30 sec")
        #expect(SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(etaInSeconds: 90).swapQuote).swapEstimationField?.value.text == "≈ 1 min, 30 sec")
        #expect(SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(etaInSeconds: 180).swapQuote).swapEstimationField?.value.text == "≈ 3 min")
    }

    @Test
    func switchRate() {
        let model = SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(toValue: 250_000_000_000).swapQuote)

        #expect(model.rateText == "1 ETH ≈ 250,000.00 USDT")

        model.switchRateDirection()
        #expect(model.rateText == "1 USDT ≈ 0.000004 ETH")
    }

    @Test
    func minReceiveAppliesSlippageBasisPoints() {
        let model = SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(toValue: 250_000_000_000).swapQuote)

        #expect(model.minReceiveField.value.text == "248,750 USDT")
    }
}
