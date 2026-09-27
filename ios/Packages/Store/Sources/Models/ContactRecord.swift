// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct ContactRecord: Codable, FetchableRecord, PersistableRecord, Sendable, Equatable {
    static let databaseTableName: String = "contacts"

    enum Columns {
        static let id = Column("id")
        static let name = Column("name")
        static let description = Column("description")
        static let imageUrl = Column("imageUrl")
        static let createdAt = Column("createdAt")
        static let updatedAt = Column("updatedAt")
    }

    var id: String
    var name: String
    var description: String?
    var imageUrl: String?
    var createdAt: Date
    var updatedAt: Date

    static let addresses = hasMany(ContactAddressRecord.self).forKey("addresses")
}

extension ContactRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.primaryKey(Columns.id.name, .text)
                .notNull()
            $0.column(Columns.name.name, .text)
                .notNull()
            $0.column(Columns.description.name, .text)
            $0.column(Columns.imageUrl.name, .text)
            $0.column(Columns.createdAt.name, .date)
                .notNull()
            $0.column(Columns.updatedAt.name, .date)
                .notNull()
        }
    }
}
