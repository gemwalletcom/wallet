// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct RecentActivityStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    public func add(_ data: RecentActivityData, walletId: WalletId) throws {
        try add(assetId: data.assetId, toAssetId: data.toAssetId, walletId: walletId, type: data.type)
    }

    public func add(
        assetId: AssetId,
        toAssetId: AssetId?,
        walletId: WalletId,
        type: RecentActivityType,
        createdAt: Date = .now,
    ) throws {
        try db.write { db in
            try RecentActivityRecord
                .filter(RecentActivityRecord.Columns.assetId == assetId.identifier)
                .filter(RecentActivityRecord.Columns.walletId == walletId.id)
                .filter(RecentActivityRecord.Columns.type == type.rawValue)
                .deleteAll(db)
            try RecentActivityRecord(
                assetId: assetId,
                toAssetId: toAssetId,
                walletId: walletId.id,
                type: type,
                createdAt: createdAt,
            ).insert(db)
        }
    }

    public func getRecent(walletId: WalletId, types: [RecentActivityType], limit: Int, filters: [AssetsQueryFilter] = []) throws -> [RecentAsset] {
        try db.read { db in
            try RecentActivityQuery(walletId: walletId, limit: limit, types: types, filters: filters).fetch(db)
        }
    }

    public func clear(scope: RecentActivityScope, types: [RecentActivityType]) throws {
        let activities = RecentActivityRecord.filter(types.map(\.rawValue).contains(RecentActivityRecord.Columns.type))
        _ = try db.write { db in
            switch scope {
            case let .wallet(walletId): try activities.filter(RecentActivityRecord.Columns.walletId == walletId.id).deleteAll(db)
            case .allWallets: try activities.deleteAll(db)
            }
        }
    }
}
