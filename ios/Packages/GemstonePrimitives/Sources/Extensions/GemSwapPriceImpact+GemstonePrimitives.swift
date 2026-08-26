// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Primitives.SwapPriceImpact {
    static func calculate(payFiatValue: Double, receiveFiatValue: Double) throws -> Primitives.SwapPriceImpact? {
        try Gemstone.calculateSwapPriceImpact(payFiatValue: payFiatValue, receiveFiatValue: receiveFiatValue)
            .map { try Primitives.SwapPriceImpact($0) }
    }
}
