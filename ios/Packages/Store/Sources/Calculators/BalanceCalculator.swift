// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public enum BalanceCalculator {
    public static func totalFiatValue(_ balances: [AssetFiatValue]) -> TotalFiatValue {
        let (total, pnl) = balances.reduce((0.0, 0.0)) { result, balance in
            let fiat = balance.amount * balance.price
            let pnlAmount = PriceChangeCalculator.calculate(.amount(percentage: balance.priceChangePercentage24h, value: fiat))
            return (result.0 + fiat, result.1 + pnlAmount)
        }
        return TotalFiatValue(
            value: total,
            pnlAmount: pnl,
            pnlPercentage: PriceChangeCalculator.calculate(.percentage(from: total - pnl, to: total)),
        )
    }
}
