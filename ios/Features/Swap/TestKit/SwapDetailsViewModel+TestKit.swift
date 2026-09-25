// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.SwapperQuote
import struct Gemstone.SwapQuote
import func Gemstone.swapQuoteDetails
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Swap

public extension SwapDetailsViewModel {
    static func mock(selectedQuote: SwapQuote = SwapperQuote.mock().swapQuote) -> SwapDetailsViewModel {
        SwapDetailsViewModel(
            details: swapQuoteDetails(quote: selectedQuote, fromAsset: Asset.mockEthereum().toGem(), toAsset: Asset.mockEthereumUSDT().toGem(), fromPrice: nil, toPrice: nil, currency: Currency.usd.toGem()),
        )
    }
}
