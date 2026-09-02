// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import class Gemstone.GemSwapQuoteService
import struct Gemstone.SwapperQuote
import Preferences
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
                .mock(selectedQuote: SwapperQuote.mock(etaInSeconds: nil).map(swapQuoteService: GemSwapQuoteService())).swapEstimationField == nil,
        )
        #expect(try SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(etaInSeconds: 30).map(swapQuoteService: GemSwapQuoteService())).swapEstimationField == nil)
        #expect(try SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(etaInSeconds: 180).map(swapQuoteService: GemSwapQuoteService())).swapEstimationField?.value.text == "≈ 3 min")
    }

    @Test
    func switchRate() throws {
        let model = try SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(toValue: "250000000000").map(swapQuoteService: GemSwapQuoteService()))

        #expect(model.rateText == "1 ETH ≈ 250,000.00 USDT")

        model.switchRateDirection()
        #expect(model.rateText == "1 USDT ≈ 0.000004 ETH")
    }

    @Test
    func minReceiveAppliesSlippageBasisPoints() throws {
        let model = try SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(toValue: "250000000000").map(swapQuoteService: GemSwapQuoteService()))

        #expect(model.minReceiveField.value.text == "248,750 USDT")
    }
}

extension SwapDetailsViewModel {
    static func mock(selectedQuote: SwapQuote = try! SwapperQuote.mock().map(swapQuoteService: GemSwapQuoteService())) -> SwapDetailsViewModel {
        SwapDetailsViewModel(
            fromAssetPrice: AssetPriceValue(asset: .mockEthereum(), price: .mock()),
            toAssetPrice: AssetPriceValue(asset: .mockEthereumUSDT(), price: .mock()),
            selectedQuote: selectedQuote,
            slippage: .auto,
            currency: Currency.usd.rawValue,
            swapPriceImpact: nil,
            minReceiveValue: (try? BigInt.from(string: GemSwapQuoteService().minReceiveValue(value: selectedQuote.toValue, slippageBps: selectedQuote.slippageBps))) ?? .zero,
            etaMinutes: selectedQuote.etaInSeconds.flatMap { GemSwapQuoteService().etaMinutes(seconds: $0) },
            swapProviderSelectAction: nil,
        )
    }
}
