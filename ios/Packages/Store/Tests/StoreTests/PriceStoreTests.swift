// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
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

    @Test
    func getPricesReportsAStoredZeroPrice() throws {
        let db = DB.mockWithChains([.ethereum])
        let priceStore = PriceStore(db: db)
        let assetId = Chain.ethereum.assetId

        try priceStore.updatePrices([.mock(assetId: assetId, price: 0)])

        #expect(try priceStore.getPrices(for: [assetId.identifier]).first?.price == 0)
    }

    @Test
    func insertKeepsTheUpdateTimestamp() throws {
        let db = DB.mockWithChains([.ethereum])
        let priceStore = PriceStore(db: db)
        let assetId = Chain.ethereum.assetId
        let updatedAt = Date(timeIntervalSince1970: 1_700_000_000)

        try priceStore.updatePrices([.mock(assetId: assetId, updatedAt: updatedAt)])

        #expect(try priceStore.getPrices(for: [assetId.identifier]).first?.updatedAt == updatedAt)
    }
}
