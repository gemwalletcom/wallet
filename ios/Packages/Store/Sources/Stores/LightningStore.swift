// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct LightningStore: Sendable {
    let db: DatabasePool

    public init(db: DB) {
        self.db = db.dbPool
    }

    public func getLeaves(walletId: WalletId) throws -> [LightningLeaf] {
        try db.read { db in
            try LightningLeafRecord
                .filter(LightningLeafRecord.Columns.walletId == walletId.id)
                .fetchAll(db)
                .map(\.leaf)
        }
    }

    public func updateLeaves(walletId: WalletId, leaves: [LightningLeaf], deleteIds: [String]) throws {
        try db.write { db in
            try LightningLeafRecord
                .filter(LightningLeafRecord.Columns.walletId == walletId.id)
                .filter(deleteIds.contains(LightningLeafRecord.Columns.id))
                .deleteAll(db)
            for leaf in leaves {
                try LightningLeafRecord(walletId: walletId.id, id: leaf.id, data: leaf.data).upsert(db)
            }
        }
    }

    public func getPendingSends(walletId: WalletId) throws -> [LightningPendingSend] {
        try db.read { db in
            try LightningPendingSendRecord
                .filter(LightningPendingSendRecord.Columns.walletId == walletId.id)
                .order(LightningPendingSendRecord.Columns.transferId)
                .fetchAll(db)
                .map(\.send)
        }
    }

    public func addPendingSend(walletId: WalletId, send: LightningPendingSend) throws {
        try db.write { db in
            try LightningPendingSendRecord(
                walletId: walletId.id,
                transferId: send.transferId,
                invoice: send.invoice,
                amountSats: send.amountSats.map(Int64.init),
                expiresAt: Int64(send.expiresAt),
            ).insert(db)
        }
    }

    public func deletePendingSend(walletId: WalletId, transferId: String) throws {
        _ = try db.write { db in
            try LightningPendingSendRecord
                .filter(LightningPendingSendRecord.Columns.walletId == walletId.id)
                .filter(LightningPendingSendRecord.Columns.transferId == transferId)
                .deleteAll(db)
        }
    }
}
