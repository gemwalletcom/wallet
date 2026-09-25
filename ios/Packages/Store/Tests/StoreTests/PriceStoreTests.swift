// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct PriceStoreTests {
    @Test
    func failedRepricingRollsBackRatesAndCanRetry() throws {
        let db = DB.mockWithChains([.ethereum])
        let priceStore = PriceStore(db: db)
        let assetId = Chain.ethereum.assetId
        let rate = FiatRate(symbol: .eur, rate: 0.9)
        try priceStore.saveRates([FiatRate(symbol: .eur, rate: 0.8)])
        try priceStore.updatePrices([.mock(assetId: assetId, price: 100, rate: 0.8)])
        try db.dbQueue.write { db in
            try db.execute(sql: "CREATE TRIGGER reject_repricing BEFORE UPDATE OF price ON prices BEGIN SELECT RAISE(ABORT, 'repricing failed'); END")
        }

        #expect(throws: DatabaseError.self) {
            try priceStore.saveRates([rate], conversion: rate)
        }
        #expect(try priceStore.getRate(currency: "EUR")?.rate == 0.8)
        #expect(try priceStore.getPrices(for: [assetId.identifier]).first?.price == 80)

        try db.dbQueue.write { db in
            try db.execute(sql: "DROP TRIGGER reject_repricing")
        }
        try priceStore.saveRates([rate], conversion: rate)

        #expect(try priceStore.getRate(currency: "EUR")?.rate == 0.9)
        #expect(try priceStore.getPrices(for: [assetId.identifier]).first?.price == 90)
    }

    @Test
    func ratesAndPricesOfATickCommitTogether() throws {
        let db = DB.mockWithChains([.ethereum, .bitcoin])
        let priceStore = PriceStore(db: db)
        let ethereum = Chain.ethereum.assetId
        let bitcoin = Chain.bitcoin.assetId
        let rate = FiatRate(symbol: .eur, rate: 0.9)
        let price = { (assetId: AssetId) in try priceStore.getPrices(for: [assetId.identifier]).first?.price }
        try priceStore.saveRates([FiatRate(symbol: .eur, rate: 0.8)])
        try priceStore.updatePrices([.mock(assetId: ethereum, price: 100, rate: 0.8)])
        try db.dbQueue.write { db in
            try db.execute(sql: "CREATE TRIGGER reject_price BEFORE INSERT ON prices BEGIN SELECT RAISE(ABORT, 'price failed'); END")
        }

        #expect(throws: DatabaseError.self) {
            try priceStore.saveRatesAndPrices([rate], conversion: rate, prices: [.mock(assetId: bitcoin, price: 200, rate: 0.9)])
        }
        #expect(try priceStore.getRate(currency: "EUR")?.rate == 0.8)
        #expect(try price(ethereum) == 80)

        try db.dbQueue.write { db in
            try db.execute(sql: "DROP TRIGGER reject_price")
        }
        try priceStore.saveRatesAndPrices([rate], conversion: rate, prices: [.mock(assetId: bitcoin, price: 200, rate: 0.9)])

        #expect(try priceStore.getRate(currency: "EUR")?.rate == 0.9)
        #expect(try price(ethereum) == 90)
        #expect(try price(bitcoin) == 180)
    }

    @Test
    func savingAnotherCurrencyDoesNotReprice() throws {
        let db = DB.mockWithChains([.ethereum])
        let priceStore = PriceStore(db: db)
        let assetId = Chain.ethereum.assetId
        try priceStore.updatePrices([.mock(assetId: assetId, price: 100, rate: 0.8)])

        try priceStore.saveRates([FiatRate(symbol: .gbp, rate: 0.7)])

        #expect(try priceStore.getRate(currency: "GBP")?.rate == 0.7)
        #expect(try priceStore.getPrices(for: [assetId.identifier]).first?.price == 80)
    }

    @Test
    func convertPricesKeepsMarketFiguresInUsd() throws {
        let db = DB.mockWithChains([.ethereum])
        let priceStore = PriceStore(db: db)
        try priceStore.updateMarket(assetId: Chain.ethereum.assetId, market: .mock(marketCap: 1000, circulatingSupply: 10))
        let read = { (column: String) in try db.dbQueue.read { try Double.fetchOne($0, sql: "SELECT \(column) FROM asset_market") } }

        _ = try priceStore.convertPrices(rate: 2)

        #expect(try read("marketCap") == 1000)
        #expect(try read("circulatingSupply") == 10)
    }

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
