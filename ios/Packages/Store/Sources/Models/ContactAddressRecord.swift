// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct ContactAddressRecord: Codable, FetchableRecord, PersistableRecord, Sendable, Equatable {
    static let databaseTableName: String = "contacts_addresses"

    enum Columns {
        static let id = Column("id")
        static let contactId = Column("contactId")
        static let address = Column("address")
        static let chain = Column("chain")
        static let memo = Column("memo")
    }

    var id: String
    var contactId: String
    var address: String
    var chain: Chain
    var memo: String?
}

extension ContactAddressRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.primaryKey(Columns.id.name, .text)
                .notNull()
            $0.column(Columns.contactId.name, .text)
                .notNull()
                .indexed()
                .references(ContactRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.address.name, .text)
                .notNull()
            $0.column(Columns.chain.name, .text)
                .notNull()
            $0.column(Columns.memo.name, .text)
        }
    }
}
