// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Primitives.SwapPriceImpact {
    static func calculate(payFiatValue: Double, receiveFiatValue: Double, swapQuoteService: GemSwapQuoteService) throws -> Primitives.SwapPriceImpact? {
        try swapQuoteService.priceImpact(payFiatValue: payFiatValue, receiveFiatValue: receiveFiatValue)
            .map { try Primitives.SwapPriceImpact($0) }
    }
}
