// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

private let balanceCalculator = BalanceCalculator()

public extension Gemstone.TotalFiatValue {
    func map() -> Primitives.TotalFiatValue {
        Primitives.TotalFiatValue(
            value: value,
            pnlAmount: pnlAmount,
            pnlPercentage: pnlPercentage,
        )
    }
}

public extension Primitives.TotalFiatValue {
    var showsPnl: Bool {
        balanceCalculator.showsPnl(self)
    }
}
