// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.TotalFiatValue {
    func map() -> Primitives.TotalFiatValue {
        Primitives.TotalFiatValue(
            value: value,
            pnlAmount: pnlAmount,
            pnlPercentage: pnlPercentage,
        )
    }
}
