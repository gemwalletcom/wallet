// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletRulesService
import Foundation
import Gemstone
import Primitives

private let walletRules = GemWalletRulesService()

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
        walletRules.showsPnl(total: Gemstone.TotalFiatValue(value: value, pnlAmount: pnlAmount, pnlPercentage: pnlPercentage))
    }
}
