// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public struct BalanceCalculator: Sendable {
    private let calculator = Gemstone.BalanceCalculator()

    public init() {}

    public func totalFiatValue(_ balances: [Primitives.AssetFiatValue]) -> Primitives.TotalFiatValue {
        calculator.totalFiatValue(balances: balances.map { $0.map() }).map()
    }
}
