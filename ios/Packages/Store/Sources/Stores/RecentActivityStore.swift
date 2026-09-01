// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct RecentActivityStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    public func add(_ data: RecentActivityData, walletId: String) throws {
        try add(assetId: data.assetId, toAssetId: data.toAssetId, walletId: walletId, type: data.type)
    }

    public func add(
        assetId: AssetId,
        toAssetId: AssetId?,
        walletId: String,
        type: RecentActivityType,
        createdAt: Date = .now,
    ) throws {
        try db.write { db in
            try RecentActivityRecord(
                assetId: assetId,
                toAssetId: toAssetId,
                walletId: walletId,
                type: type,
                createdAt: createdAt,
            ).insert(db)
        }
    }

    public func getRecent(walletId: WalletId, types: [RecentActivityType]) throws -> [RecentAsset] {
        try db.read { db in
            try RecentActivityRequest(walletId: walletId, types: types).fetch(db)
        }
    }

    public func clear(walletId: String, types: [RecentActivityType]) throws {
        _ = try db.write { db in
            try RecentActivityRecord
                .filter(RecentActivityRecord.Columns.walletId == walletId)
                .filter(types.map(\.rawValue).contains(RecentActivityRecord.Columns.type))
                .deleteAll(db)
        }
    }
}
