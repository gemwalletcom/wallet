// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct PriceStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    public func getRate(currency: String) throws -> FiatRateRecord? {
        try db.read { db in
            try FiatRateRecord.filter(key: currency).fetchOne(db)
        }
    }

    public func getRates() throws -> [FiatRateRecord] {
        try db.read { db in
            try FiatRateRecord.fetchAll(db)
        }
    }

    public func saveRates(_ rates: [FiatRate], conversion: FiatRate? = nil) throws {
        try db.write { db in
            for rate in rates {
                try rate.record.upsert(db)
            }
            if let conversion {
                _ = try convertPrices(db, rate: conversion.rate)
            }
        }
    }

    public func updatePrices(_ updates: [PriceUpdate]) throws {
        try db.write { db in
            for update in updates {
                _ = try update.record.upsertAndFetch(
                    db,
                    onConflict: [],
                    doUpdate: { _ in [
                        PriceRecord.Columns.price.set(to: update.price),
                        PriceRecord.Columns.priceUsd.set(to: update.priceUsd),
                        PriceRecord.Columns.priceChangePercentage24h.set(to: update.priceChangePercentage24h),
                        PriceRecord.Columns.updatedAt.set(to: update.updatedAt),
                    ] },
                )
            }
        }
    }

    public func updateMarket(assetId: AssetId, market: AssetMarket) throws {
        try db.write { db in
            try AssetMarketRecord(assetId: assetId, market: market).upsert(db)
        }
    }

    public func getPrices(for assetIds: [String]) throws -> [AssetPrice] {
        try db.read { db in
            try PriceRecord
                .filter(assetIds.contains(PriceRecord.Columns.assetId))
                .fetchAll(db)
                .map { $0.mapToAssetPrice() }
        }
    }

    @discardableResult
    public func convertPrices(rate: Double) throws -> Int {
        try db.write { db in
            try convertPrices(db, rate: rate)
        }
    }

    private func convertPrices(_ db: Database, rate: Double) throws -> Int {
        try PriceRecord.updateAll(db, [
            PriceRecord.Columns.price.set(to: PriceRecord.Columns.priceUsd * rate),
        ])
    }

    public func clear() throws -> Int {
        try db.write {
            try PriceRecord
                .deleteAll($0)
        }
    }
}
