// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct AssetFiatValuesRequestTests {
    @Test
    func walletBalanceWithPrice() throws {
        let db = try DB.mockAssetsWithPrice(priceChangePercentage24h: 10)

        try db.dbQueue.read { db in
            let result = try AssetFiatValuesRequest(walletId: .mock()).fetch(db)

            #expect(result.contains(AssetFiatValue(amount: 3, price: 1100, priceChangePercentage24h: 10)))
            #expect(result.filter { $0.price == 0 }.map(\.amount).sorted() == [0, 1, 2, 4])
        }
    }

    @Test
    func walletBalanceWithoutPrice() throws {
        let db = DB.mockAssets()

        try db.dbQueue.read { db in
            let result = try AssetFiatValuesRequest(walletId: .mock()).fetch(db)

            #expect(result.filter { $0.price == 0 }.map(\.amount).sorted() == [0, 1, 2, 3, 4])
        }
    }

    @Test
    func walletBalanceListsEnabledAssetsOnly() throws {
        let db = try DB.mockAssetsWithPerpetualCollateralBalance()

        try db.dbQueue.read { db in
            let result = try AssetFiatValuesRequest(walletId: .mock()).fetch(db)

            #expect(result == [AssetFiatValue(amount: 3, price: 100, priceChangePercentage24h: 0)])
        }
    }

    @Test
    func perpetualWalletBalanceCarriesTheStoredCollateral() throws {
        let db = try DB.mockAssetsWithPerpetualCollateralBalance()

        try db.dbQueue.read { db in
            let result = try PerpetualWalletBalanceRequest(walletId: .mock(), assetId: Asset.mockHypercoreUSDC().id).fetch(db)

            #expect(result?.available == 50)
            #expect(result?.reserved == 25)
        }
    }
}
