// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct AssetListRecord: Codable, PersistableRecord, FetchableRecord, TableRecord {
    static let databaseTableName = "asset_lists"

    enum Columns {
        static let id = Column("id")
        static let name = Column("name")
    }

    var id: String
    var name: String
}

extension AssetListRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.primaryKey(Columns.id.name, .text)
            $0.column(Columns.name.name, .text)
                .notNull()
        }
    }
}

extension AssetListRecord {
    static let search = hasOne(SearchRecord.self, using: ForeignKey(["listId"], to: ["id"]))

    var assetList: AssetList {
        AssetList(id: id, name: name)
    }
}

extension AssetList {
    var record: AssetListRecord {
        AssetListRecord(id: id, name: name)
    }
}
