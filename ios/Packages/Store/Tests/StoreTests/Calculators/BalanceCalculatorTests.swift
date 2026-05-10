// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Store
import Testing

struct BalanceCalculatorTests {
    @Test
    func totalFiatValueWithEmptyBalances() {
        let result = BalanceCalculator.totalFiatValue([])

        #expect(result.value == 0)
        #expect(result.pnlAmount == 0)
        #expect(result.pnlPercentage == 0)
    }

    @Test
    func totalFiatValueWithPositiveChange() {
        let result = BalanceCalculator.totalFiatValue([
            AssetFiatValue(amount: 3, price: 1100, priceChangePercentage24h: 10),
        ])

        #expect(result.value == 3300)
        #expect(result.pnlAmount == 300)
        #expect(result.pnlPercentage == 10)
    }

    @Test
    func totalFiatValueWithZeroChange() {
        let result = BalanceCalculator.totalFiatValue([
            AssetFiatValue(amount: 3, price: 1100, priceChangePercentage24h: 0),
        ])

        #expect(result.value == 3300)
        #expect(result.pnlAmount == 0)
        #expect(result.pnlPercentage == 0)
    }

    @Test
    func totalFiatValueAggregatesMultipleBalances() {
        let result = BalanceCalculator.totalFiatValue([
            AssetFiatValue(amount: 2, price: 100, priceChangePercentage24h: 25),
            AssetFiatValue(amount: 1, price: 200, priceChangePercentage24h: 0),
        ])

        #expect(result.value == 400)
        #expect(result.pnlAmount == 40)
        #expect(result.pnlPercentage == 11.11111111111111)
    }

    @Test
    func totalFiatValueIgnoresZeroAmount() {
        let result = BalanceCalculator.totalFiatValue([
            AssetFiatValue(amount: 0, price: 100, priceChangePercentage24h: 10),
            AssetFiatValue(amount: 1, price: 100, priceChangePercentage24h: 0),
        ])

        #expect(result.value == 100)
        #expect(result.pnlAmount == 0)
        #expect(result.pnlPercentage == 0)
    }
}
