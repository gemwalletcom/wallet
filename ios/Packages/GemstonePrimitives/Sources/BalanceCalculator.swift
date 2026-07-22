// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public struct BalanceCalculator: Sendable {
    private let calculator = Gemstone.BalanceCalculator()

    public init() {}

    public func totalFiatValue(_ balances: [Primitives.AssetFiatValue]) -> Primitives.TotalFiatValue {
        let result = calculator.totalFiatValue(balances: balances.map {
            Gemstone.AssetFiatValue(amount: $0.amount, price: $0.price, priceChangePercentage24h: $0.priceChangePercentage24h)
        })
        return Primitives.TotalFiatValue(value: result.value, pnlAmount: result.pnlAmount, pnlPercentage: result.pnlPercentage)
    }
}
