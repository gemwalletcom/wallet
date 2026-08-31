// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct PriceUsdRequestTests {
    @Test
    func returnsUsdPriceIgnoringSelectedCurrency() throws {
        let db = DB.mockAssets()
        let fiatRateStore = FiatRateStore(db: db)
        let priceStore = PriceStore(db: db)

        try fiatRateStore.add([FiatRate(symbol: .jpy, rate: 150)])

        let ethId = AssetId(chain: .ethereum)
        try priceStore.updatePrices([.mock(assetId: ethId, price: 1100, rate: 150)])

        try db.dbQueue.read { db in
            let result = try PriceUsdRequest(assetId: ethId).fetch(db)
            #expect(result == 1100)
        }
    }
}
