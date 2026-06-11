// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.SwapperError
import Primitives
import PrimitivesTestKit
@testable import Swap
import Testing

struct SwapperErrorTests {
    @Test
    func isRetryAvailable() {
        #expect(SwapperError.NoQuoteAvailable.isRetryAvailable == true)
        #expect(SwapperError.ComputeQuoteError("error").isRetryAvailable == true)

        #expect(SwapperError.NotSupportedChain.isRetryAvailable == false)
        #expect(SwapperError.InputAmountError(minAmount: nil).isRetryAvailable == false)
    }

    @Test
    func inputAmountErrorMessage() {
        #expect(
            SwapperError.InputAmountError(minAmount: "120966091866986").message(asset: .mockBNB()) ==
                "Minimum trade amount is **0.0001209 BNB**. Please enter a higher amount.",
        )
        #expect(
            SwapperError.InputAmountError(minAmount: "123456").message(asset: .mock(symbol: "USDT", decimals: 6)) ==
                "Minimum trade amount is **0.1234 USDT**. Please enter a higher amount.",
        )
    }

    @Test
    func userFacingMessages() {
        let asset = Asset.mockBNB()

        #expect(SwapperError.NotSupportedAsset.message(asset: asset) == "Not supported asset.")
        #expect(SwapperError.NotSupportedChain.message(asset: asset) == "Not supported asset.")
        #expect(SwapperError.NoQuoteAvailable.message(asset: asset) == "No quote available.")
        #expect(SwapperError.NoAvailableProvider.message(asset: asset) == "No quote available.")
        #expect(SwapperError.InvalidRoute.message(asset: asset) == "No quote available.")
        #expect(SwapperError.ComputeQuoteError("HTTP error: status 500").message(asset: asset) == "No quote available.")
        #expect(SwapperError.TransactionError("failed to decode transaction").message(asset: asset) == "No quote available.")
    }
}
