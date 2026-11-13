// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct PriceUsdQueryTests {
    @Test
    func returnsUsdPriceIgnoringSelectedCurrency() throws {
        let db = DB.mock(chains: [.ethereum])
        let priceStore = PriceStore(db: db)

        try priceStore.saveRatesAndPrices([FiatRate(symbol: .jpy, rate: 150)], conversion: nil, prices: [])

        let ethId = AssetId(chain: .ethereum)
        try priceStore.updatePrices([.mock(assetId: ethId, price: 1100, rate: 150)])

        try db.dbPool.read { db in
            let result = try PriceUsdQuery(assetId: ethId).fetch(db)
            #expect(result == 1100)
        }
    }
}
