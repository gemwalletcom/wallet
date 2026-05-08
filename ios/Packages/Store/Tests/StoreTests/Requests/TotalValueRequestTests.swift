// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct TotalValueRequestTests {
    @Test
    func walletBalanceWithPrice() throws {
        let db = DB.mockAssets()
        let fiatRateStore = FiatRateStore(db: db)
        let priceStore = PriceStore(db: db)

        try fiatRateStore.add([FiatRate(symbol: Currency.usd.rawValue, rate: 1)])

        let ethId = AssetId(chain: .ethereum)
        try priceStore.updatePrice(
            price: AssetPrice(assetId: ethId, price: 1100, priceChangePercentage24h: 10, updatedAt: .now),
            currency: Currency.usd.rawValue,
        )

        try db.dbQueue.read { db in
            let result = try TotalValueRequest(walletId: .mock(), type: .wallet).fetch(db)

            #expect(result.value == 3300)
            #expect(result.pnlAmount == 300)
            #expect(result.pnlPercentage == 10)
        }
    }

    @Test
    func walletBalanceWithoutPrice() throws {
        let db = DB.mockAssets()

        try db.dbQueue.read { db in
            let result = try TotalValueRequest(walletId: .mock(), type: .wallet).fetch(db)

            #expect(result.value == 0)
            #expect(result.pnlAmount == 0)
            #expect(result.pnlPercentage == 0)
        }
    }

    @Test
    func walletBalanceZeroChange() throws {
        let db = DB.mockAssets()
        let fiatRateStore = FiatRateStore(db: db)
        let priceStore = PriceStore(db: db)

        try fiatRateStore.add([FiatRate(symbol: Currency.usd.rawValue, rate: 1)])

        let ethId = AssetId(chain: .ethereum)
        try priceStore.updatePrice(
            price: AssetPrice(assetId: ethId, price: 1100, priceChangePercentage24h: 0, updatedAt: .now),
            currency: Currency.usd.rawValue,
        )

        try db.dbQueue.read { db in
            let result = try TotalValueRequest(walletId: .mock(), type: .wallet).fetch(db)

            #expect(result.value == 3300)
            #expect(result.pnlAmount == 0)
            #expect(result.pnlPercentage == 0)
        }
    }

    @Test
    func walletBalanceIncludesPerpetualCollateralAndExcludesDisabled() throws {
        let db = try DB.mockAssetsWithPerpetualCollateralBalance()

        try db.dbQueue.read { db in
            let result = try TotalValueRequest(walletId: .mock(), type: .wallet).fetch(db)

            // ethereum (3 * 100) + perpetual (50 + 25); bnb is disabled
            #expect(result.value == 375)
            #expect(result.pnlAmount == 0)
            #expect(result.pnlPercentage == 0)
        }
    }

    @Test
    func perpetualBalanceUsesCollateralOnly() throws {
        let db = try DB.mockAssetsWithPerpetualCollateralBalance()

        try db.dbQueue.read { db in
            let result = try TotalValueRequest(walletId: .mock(), type: .perpetual).fetch(db)

            #expect(result.value == 75)
            #expect(result.pnlAmount == 0)
            #expect(result.pnlPercentage == 0)
        }
    }

    @Test
    func perpetualWalletBalanceSplitsTotalAndAvailable() throws {
        let db = try DB.mockAssetsWithPerpetualCollateralBalance()

        try db.dbQueue.read { db in
            let result = try PerpetualWalletBalanceRequest(walletId: .mock()).fetch(db)

            #expect(result.total == 75)
            #expect(result.available == 50)
        }
    }

    @Test
    func earnBalanceSumsStakedAndEarn() throws {
        let db = try DB.mockAssetsWithEarnBalance()

        try db.dbQueue.read { db in
            let result = try TotalValueRequest(walletId: .mock(), type: .earn).fetch(db)

            #expect(result.value == 330)
            #expect(result.pnlAmount == 30)
            #expect(result.pnlPercentage == 10)
        }
    }
}
