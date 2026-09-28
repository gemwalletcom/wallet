// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct NFTAssetAssociationRecord: Codable, FetchableRecord, PersistableRecord, Identifiable {
    static let databaseTableName = NFTAssetRecord.databaseTableName + "_associations"

    enum Columns {
        static let id = Column("id")
        static let walletId = Column("walletId")
        static let collectionId = Column("collectionId")
        static let assetId = Column("assetId")
    }

    var id: String
    var walletId: String
    var collectionId: NFTCollectionId
    var assetId: NFTAssetId

    init(walletId: String, collectionId: NFTCollectionId, assetId: NFTAssetId) {
        id = Self.computedId(walletId: walletId, collectionId: collectionId, assetId: assetId)
        self.walletId = walletId
        self.collectionId = collectionId
        self.assetId = assetId
    }
}

extension NFTAssetAssociationRecord {
    static func computedId(walletId: String, collectionId: NFTCollectionId, assetId: NFTAssetId) -> String {
        [walletId, collectionId.identifier, assetId.identifier].joined(separator: "_")
    }
}

extension NFTAssetAssociationRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.column(Columns.id.name, .text)
                .primaryKey()
            $0.column(Columns.walletId.name, .text)
                .notNull()
                .indexed()
                .referencesWallet()
            $0.column(Columns.collectionId.name, .text)
                .notNull()
                .indexed()
                .references(NFTCollectionRecord.databaseTableName, onDelete: .cascade)
            $0.column(Columns.assetId.name, .text)
                .notNull()
                .indexed()
                .references(NFTAssetRecord.databaseTableName, onDelete: .cascade)
        }
    }
}
