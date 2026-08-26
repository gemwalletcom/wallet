// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct PriceStoreTests {
    @Test
    func convertPricesRecomputesFiatPriceFromUsd() throws {
        let db = DB.mockWithChains([.ethereum])
        let priceStore = PriceStore(db: db)
        let assetId = Chain.ethereum.assetId
        let priceUsd = 2500.0

        try priceStore.updatePrices([.mock(assetId: assetId, price: priceUsd, rate: 90)])
        #expect(try priceStore.getPrices(for: [assetId.identifier]).first?.price == priceUsd * 90)

        try priceStore.convertPrices(rate: 2)
        #expect(try priceStore.getPrices(for: [assetId.identifier]).first?.price == priceUsd * 2)
    }
}
