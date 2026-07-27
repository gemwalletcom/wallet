// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.calculateSwapPriceImpact
import Primitives

public func calculateSwapPriceImpact(payFiatValue: Double, receiveFiatValue: Double) -> Primitives.SwapPriceImpact? {
    Gemstone.calculateSwapPriceImpact(payFiatValue: payFiatValue, receiveFiatValue: receiveFiatValue)?.map()
}
