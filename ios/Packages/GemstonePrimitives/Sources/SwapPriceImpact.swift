// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public func calculateSwapPriceImpact(payFiatValue: Double, receiveFiatValue: Double) -> Gemstone.SwapPriceImpact? {
    Gemstone.calculateSwapPriceImpact(payFiatValue: payFiatValue, receiveFiatValue: receiveFiatValue)
}
