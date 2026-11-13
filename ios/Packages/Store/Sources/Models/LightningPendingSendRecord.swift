// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB

struct LightningPendingSendRecord: Codable, PersistableRecord, FetchableRecord, TableRecord {
    static let databaseTableName = "lightning_pending_sends"

    enum Columns {
        static let walletId = Column("walletId")
        static let transferId = Column("transferId")
        static let invoice = Column("invoice")
        static let amountSats = Column("amountSats")
        static let expiresAt = Column("expiresAt")
    }

    var walletId: String
    var transferId: String
    var invoice: String
    var amountSats: Int64?
    var expiresAt: Int64
}

extension LightningPendingSendRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.column(Columns.walletId.name, .text)
                .notNull()
                .referencesWallet()
            $0.column(Columns.transferId.name, .text)
                .notNull()
            $0.column(Columns.invoice.name, .text)
                .notNull()
            $0.column(Columns.amountSats.name, .integer)
            $0.column(Columns.expiresAt.name, .integer)
                .notNull()
            $0.primaryKey([Columns.walletId.name, Columns.transferId.name])
        }
    }
}

extension LightningPendingSendRecord {
    var send: LightningPendingSend {
        LightningPendingSend(
            transferId: transferId,
            invoice: invoice,
            amountSats: amountSats.map(UInt64.init),
            expiresAt: UInt64(expiresAt),
        )
    }
}
