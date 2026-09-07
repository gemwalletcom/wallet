// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import class Gemstone.GemSwapQuoteSummary
import GemstonePrimitives
import struct Gemstone.SwapperQuote
import struct Gemstone.SwapQuote
import Primitives
import PrimitivesTestKit
@testable import Swap
import GemstoneServicesTestKit
import Testing

@MainActor
struct SwapDetailsViewModelTests {
    @Test
    func swapEstimationField() throws {
        #expect(
            try SwapDetailsViewModel
                .mock(selectedQuote: SwapperQuote.mock(etaInSeconds: nil).swapQuote).swapEstimationField == nil,
        )
        #expect(SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(etaInSeconds: 30).swapQuote).swapEstimationField == nil)
        #expect(SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(etaInSeconds: 180).swapQuote).swapEstimationField?.value.text == "≈ 3 min")
    }

    @Test
    func switchRate() throws {
        let model = SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(toValue: 250_000_000_000).swapQuote)

        #expect(model.rateText == "1 ETH ≈ 250,000.00 USDT")

        model.switchRateDirection()
        #expect(model.rateText == "1 USDT ≈ 0.000004 ETH")
    }

    @Test
    func minReceiveAppliesSlippageBasisPoints() throws {
        let model = SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(toValue: 250_000_000_000).swapQuote)

        #expect(model.minReceiveField.value.text == "248,750 USDT")
    }
}

extension SwapDetailsViewModel {
    static func mock(selectedQuote: Gemstone.SwapQuote = SwapperQuote.mock().swapQuote) -> SwapDetailsViewModel {
        let summary = GemSwapQuoteSummary(quote: selectedQuote)
        return SwapDetailsViewModel(
            fromAssetPrice: AssetPriceValue(asset: .mockEthereum(), price: .mock()),
            toAssetPrice: AssetPriceValue(asset: .mockEthereumUSDT(), price: .mock()),
            selectedQuote: selectedQuote,
            slippage: .auto,
            currency: Currency.usd.rawValue,
            swapPriceImpact: nil,
            minReceiveValue: BigInt(summary.minReceiveValue()),
            etaMinutes: summary.etaMinutes(),
            swapProviderSelectAction: nil,
        )
    }
}
