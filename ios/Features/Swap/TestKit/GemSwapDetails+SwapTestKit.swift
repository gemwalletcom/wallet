// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSwapDetails
import struct Gemstone.SwapQuote
import func Gemstone.swapQuoteDetails
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit

public extension GemSwapDetails {
    static func mock(selectedQuote: SwapQuote = .mock()) -> GemSwapDetails {
        swapQuoteDetails(
            quote: selectedQuote,
            fromAsset: Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(),
            toAsset: Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20).toGem(),
            fromPrice: nil,
            toPrice: nil,
            currency: Currency.usd.toGem(),
        )
    }
}
