// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives

public enum BalanceCalculator {
    private static let priceChangeCalculator = PriceChangeCalculator()
    public static func totalFiatValue(_ balances: [AssetFiatValue]) -> TotalFiatValue {
        let (total, pnl) = balances.reduce((0.0, 0.0)) { result, balance in
            let fiat = balance.amount * balance.price
            let pnlAmount = priceChangeCalculator.amount(percentage: balance.priceChangePercentage24h, value: fiat)
            return (result.0 + fiat, result.1 + pnlAmount)
        }
        return TotalFiatValue(
            value: total,
            pnlAmount: pnl,
            pnlPercentage: priceChangeCalculator.percentage(from: total - pnl, to: total),
        )
    }
}
