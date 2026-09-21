// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.SwapperQuote
import struct Gemstone.SwapQuote
import func Gemstone.swapQuoteSummary
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Swap

public extension SwapDetailsViewModel {
    static func mock(selectedQuote: SwapQuote = SwapperQuote.mock().swapQuote) -> SwapDetailsViewModel {
        SwapDetailsViewModel(
            fromAssetPrice: AssetPriceValue(asset: .mockEthereum(), price: .mock()),
            toAssetPrice: AssetPriceValue(asset: .mockEthereumUSDT(), price: .mock()),
            summary: swapQuoteSummary(quote: selectedQuote, fromAsset: Asset.mockEthereum().toGem(), toAsset: Asset.mockEthereumUSDT().toGem()),
            slippagePercent: nil,
            currency: Currency.usd.rawValue,
            swapPriceImpact: nil,
        )
    }
}
