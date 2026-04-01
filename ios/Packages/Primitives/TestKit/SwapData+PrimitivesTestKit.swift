// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension SwapData {
    static func mock(
        quote: SwapQuote = .mock(),
        data: SwapQuoteData = .mock(),
    ) -> SwapData {
        SwapData(quote: quote, data: data)
    }
}
