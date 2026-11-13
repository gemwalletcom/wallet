// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB

struct LightningLeafRecord: Codable, PersistableRecord, FetchableRecord, TableRecord {
    static let databaseTableName = "lightning_leaves"

    enum Columns {
        static let walletId = Column("walletId")
        static let id = Column("id")
        static let data = Column("data")
    }

    var walletId: String
    var id: String
    var data: String
}

extension LightningLeafRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.column(Columns.walletId.name, .text)
                .notNull()
                .referencesWallet()
            $0.column(Columns.id.name, .text)
                .notNull()
            $0.column(Columns.data.name, .text)
                .notNull()
            $0.primaryKey([Columns.walletId.name, Columns.id.name])
        }
    }
}

extension LightningLeafRecord {
    var leaf: LightningLeaf {
        LightningLeaf(id: id, data: data)
    }
}
