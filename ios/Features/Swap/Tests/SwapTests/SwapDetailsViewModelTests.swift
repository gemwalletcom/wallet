// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import struct Gemstone.SwapperQuote
import Preferences
import Primitives
import PrimitivesTestKit
import SwapServiceTestKit
import Testing

@testable import Swap

@MainActor
struct SwapDetailsViewModelTests {
    @Test
    func swapEstimationField() throws {
        #expect(
            try SwapDetailsViewModel
                .mock(selectedQuote: SwapperQuote.mock(etaInSeconds: nil).map()).swapEstimationField == nil,
        )
        #expect(try SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(etaInSeconds: 30).map()).swapEstimationField == nil)
        #expect(try SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(etaInSeconds: 180).map()).swapEstimationField?.value.text == "≈ 3 min")
    }

    @Test
    func switchRate() throws {
        let model = try SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(toValue: "250000000000").map())

        #expect(model.rateText == "1 ETH ≈ 250,000.00 USDT")

        model.switchRateDirection()
        #expect(model.rateText == "1 USDT ≈ 0.000004 ETH")
    }
}

extension SwapDetailsViewModel {
    static func mock(selectedQuote: SwapQuote = try! SwapperQuote.mock().map()) -> SwapDetailsViewModel {
        SwapDetailsViewModel(
            state: .data([SwapperQuote.mock()]),
            fromAssetPrice: AssetPriceValue(asset: .mockEthereum(), price: .mock()),
            toAssetPrice: AssetPriceValue(asset: .mockEthereumUSDT(), price: .mock()),
            selectedQuote: selectedQuote,
            preferences: .mock(),
            swapProviderSelectAction: nil,
        )
    }
}
