// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct PerpetualPositionRecord: Codable, TableRecord, FetchableRecord, PersistableRecord {
    static let databaseTableName: String = "perpetuals_positions"

    enum Columns {
        static let id = Column("id")
        static let walletId = Column("walletId")
        static let perpetualId = Column("perpetualId")
        static let assetId = Column("assetId")
        static let size = Column("size")
        static let sizeValue = Column("sizeValue")
        static let leverage = Column("leverage")
        static let entryPrice = Column("entryPrice")
        static let liquidationPrice = Column("liquidationPrice")
        static let marginType = Column("marginType")
        static let direction = Column("direction")
        static let marginAmount = Column("marginAmount")
        static let takeProfit = Column("takeProfit")
        static let stopLoss = Column("stopLoss")
        static let pnl = Column("pnl")
        static let funding = Column("funding")
        static let updatedAt = Column("updatedAt")
    }

    var id: String
    var walletId: String
    var perpetualId: PerpetualId
    var assetId: AssetId
    var size: Double
    var sizeValue: Double
    var leverage: Int
    var entryPrice: Double
    var liquidationPrice: Double?
    var marginType: PerpetualMarginType
    var direction: PerpetualDirection
    var marginAmount: Double
    var takeProfit: PerpetualTriggerOrder?
    var stopLoss: PerpetualTriggerOrder?
    var pnl: Double
    var funding: Float?
    var updatedAt: Date = .init()

    // MARK: - Associations

    static let perpetual = belongsTo(PerpetualRecord.self, using: ForeignKey([Columns.perpetualId]))
}

extension PerpetualPositionRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName) {
            $0.column(Columns.id.name, .text).notNull()
            $0.column(Columns.walletId.name, .text).notNull().indexed()
                .references(WalletRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.perpetualId.name, .text).notNull()
                .references(PerpetualRecord.databaseTableName, onDelete: .cascade)
            $0.column(Columns.assetId.name, .jsonText).notNull()
                .references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.size.name, .double).notNull()
            $0.column(Columns.sizeValue.name, .double).notNull()
            $0.column(Columns.leverage.name, .integer).notNull()
            $0.column(Columns.entryPrice.name, .double).notNull()
            $0.column(Columns.liquidationPrice.name, .double)
            $0.column(Columns.marginType.name, .text).notNull()
            $0.column(Columns.direction.name, .text).notNull()
            $0.column(Columns.marginAmount.name, .double).notNull()
            $0.column(Columns.takeProfit.name, .jsonText)
            $0.column(Columns.stopLoss.name, .jsonText)
            $0.column(Columns.pnl.name, .double).notNull()
            $0.column(Columns.funding.name, .double)
            $0.column(Columns.updatedAt.name, .date).notNull()
            $0.uniqueKey([
                Columns.id.name,
                Columns.walletId.name,
            ])
        }
    }
}

// MARK: - Mapping
