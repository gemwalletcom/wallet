// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct PerpetualRecord: Codable, TableRecord, FetchableRecord, PersistableRecord {
    static let databaseTableName: String = "perpetuals"

    enum Columns {
        static let id = Column("id")
        static let name = Column("name")
        static let provider = Column("provider")
        static let assetId = Column("assetId")
        static let identifier = Column("identifier")
        static let price = Column("price")
        static let pricePercentChange24h = Column("pricePercentChange24h")
        static let openInterest = Column("openInterest")
        static let volume24h = Column("volume24h")
        static let funding = Column("funding")
        static let maxLeverage = Column("maxLeverage")
        static let isIsolatedOnly = Column("isIsolatedOnly")
        static let isPinned = Column("isPinned")
    }

    var id: PerpetualId
    var name: String
    var provider: PerpetualProvider
    var assetId: AssetId
    var identifier: String
    var price: Double
    var pricePercentChange24h: Double
    var openInterest: Double
    var volume24h: Double
    var funding: Double
    var maxLeverage: UInt8
    var isIsolatedOnly: Bool = false
    var isPinned: Bool = false

    static let positions = hasMany(PerpetualPositionRecord.self).forKey("positions")
    static let asset = belongsTo(AssetRecord.self, using: ForeignKey(["assetId"], to: ["id"]))
    static let search = hasOne(SearchRecord.self, using: ForeignKey(["perpetualId"], to: ["id"]))
}

extension PerpetualRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName) {
            $0.column(Columns.id.name, .text).primaryKey().notNull()
            $0.column(Columns.name.name, .text).notNull()
            $0.column(Columns.provider.name, .text).notNull()
            $0.column(Columns.assetId.name, .text).notNull()
                .references(AssetRecord.databaseTableName, column: AssetRecord.Columns.id.name, onDelete: .cascade)
            $0.column(Columns.identifier.name, .text).notNull()
            $0.column(Columns.price.name, .double).notNull()
            $0.column(Columns.pricePercentChange24h.name, .double).notNull()
            $0.column(Columns.openInterest.name, .double).notNull()
            $0.column(Columns.volume24h.name, .double).notNull()
            $0.column(Columns.funding.name, .double).notNull()
            $0.column(Columns.maxLeverage.name, .integer).notNull()
            $0.column(Columns.isIsolatedOnly.name, .boolean).notNull().defaults(to: false)
            $0.column(Columns.isPinned.name, .boolean).notNull().defaults(to: false)
        }
    }
}
