// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct PerpetualStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    public func upsertPerpetuals(_ perpetuals: [Perpetual]) throws {
        try db.write { db in
            for perpetual in perpetuals {
                try perpetual.record.insert(db, onConflict: .ignore)
                try PerpetualRecord
                    .filter(PerpetualRecord.Columns.id == perpetual.id)
                    .updateAll(
                        db,
                        PerpetualRecord.Columns.name.set(to: perpetual.name),
                        PerpetualRecord.Columns.provider.set(to: perpetual.provider.rawValue),
                        PerpetualRecord.Columns.assetId.set(to: perpetual.assetId.identifier),
                        PerpetualRecord.Columns.identifier.set(to: perpetual.identifier),
                        PerpetualRecord.Columns.price.set(to: perpetual.price),
                        PerpetualRecord.Columns.pricePercentChange24h.set(to: perpetual.pricePercentChange24h),
                        PerpetualRecord.Columns.openInterest.set(to: perpetual.openInterest),
                        PerpetualRecord.Columns.volume24h.set(to: perpetual.volume24h),
                        PerpetualRecord.Columns.funding.set(to: perpetual.funding),
                        PerpetualRecord.Columns.maxLeverage.set(to: perpetual.maxLeverage),
                        PerpetualRecord.Columns.isIsolatedOnly.set(to: perpetual.isIsolatedOnly),
                    )
            }
        }
    }

    public func getPerpetuals() throws -> [Perpetual] {
        try db.read { db in
            try PerpetualRecord
                .fetchAll(db)
                .map { $0.mapToPerpetual() }
        }
    }

    public func getPositions(walletId: WalletId) throws -> [PerpetualPosition] {
        try db.read { db in
            try PerpetualPositionRecord
                .filter(PerpetualPositionRecord.Columns.walletId == walletId.id)
                .order(PerpetualPositionRecord.Columns.updatedAt.desc)
                .fetchAll(db)
                .map { $0.mapToPerpetualPosition() }
        }
    }

    public func getPositions(walletId: WalletId, provider: PerpetualProvider) throws -> [PerpetualPosition] {
        try db.read { db in
            try PerpetualPositionRecord
                .joining(required: PerpetualPositionRecord.perpetual
                    .filter(PerpetualRecord.Columns.provider == provider.rawValue))
                .filter(PerpetualPositionRecord.Columns.walletId == walletId.id)
                .order(PerpetualPositionRecord.Columns.updatedAt.desc)
                .fetchAll(db)
                .map { $0.mapToPerpetualPosition() }
        }
    }

    public func diffPositions(deleteIds: [String], positions: [PerpetualPosition], walletId: WalletId) throws {
        if deleteIds.isEmpty, positions.isEmpty {
            return
        }
        try db.write { db in
            try PerpetualPositionRecord
                .filter(deleteIds.contains(PerpetualPositionRecord.Columns.id))
                .deleteAll(db)

            for position in positions {
                try position.record(walletId: walletId.id).upsert(db)
            }
        }
    }

    @discardableResult
    public func setPinned(for perpetualIds: [String], value: Bool) throws -> Int {
        try setColumn(for: perpetualIds, column: PerpetualRecord.Columns.isPinned, value: value)
    }

    private func setColumn(for perpetualIds: [String], column: Column, value: Bool) throws -> Int {
        try db.write { db in
            try PerpetualRecord
                .filter(perpetualIds.contains(PerpetualRecord.Columns.id))
                .updateAll(db, column.set(to: value))
        }
    }

    public func updatePrices(_ prices: [String: Double]) throws {
        guard !prices.isEmpty else { return }
        try db.write { db in
            for (name, price) in prices {
                try PerpetualRecord
                    .filter(PerpetualRecord.Columns.name == name)
                    .updateAll(db, PerpetualRecord.Columns.price.set(to: price))
            }
        }
    }

    @discardableResult
    public func updateMarket(
        coin: String,
        price: Double,
        pricePercentChange24h: Double,
        openInterest: Double,
        volume24h: Double,
        funding: Double,
    ) throws -> Int {
        try db.write { db in
            try PerpetualRecord
                .filter(PerpetualRecord.Columns.name == coin)
                .updateAll(
                    db,
                    PerpetualRecord.Columns.price.set(to: price),
                    PerpetualRecord.Columns.pricePercentChange24h.set(to: pricePercentChange24h),
                    PerpetualRecord.Columns.openInterest.set(to: openInterest),
                    PerpetualRecord.Columns.volume24h.set(to: volume24h),
                    PerpetualRecord.Columns.funding.set(to: funding),
                )
        }
    }

    public func clear() throws {
        try db.write { db in
            try PerpetualPositionRecord.deleteAll(db)
            try PerpetualRecord.deleteAll(db)
        }
    }
}
