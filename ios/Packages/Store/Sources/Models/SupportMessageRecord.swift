// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct SupportMessageRecord: Codable, FetchableRecord, PersistableRecord {
    static let databaseTableName: String = "support_messages"

    enum Columns {
        static let id = Column("id")
        static let content = Column("content")
        static let sender = Column("sender")
        static let status = Column("status")
        static let createdAt = Column("createdAt")
        static let images = Column("images")
    }

    var id: String
    var content: String
    var sender: SupportMessageSender
    var status: SupportMessageStatus
    var createdAt: Date
    var images: [SupportMessageImage]
}

extension SupportMessageRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.primaryKey(Columns.id.name, .text)
                .notNull()
            $0.column(Columns.content.name, .text)
                .notNull()
            $0.column(Columns.sender.name, .jsonText)
                .notNull()
            $0.column(Columns.status.name, .text)
                .notNull()
            $0.column(Columns.createdAt.name, .datetime)
                .notNull()
                .indexed()
            $0.column(Columns.images.name, .jsonText)
                .notNull()
        }
    }
}

extension SupportMessageRecord {
    var message: SupportMessage {
        SupportMessage(
            id: id,
            content: content,
            sender: sender,
            status: status,
            createdAt: createdAt,
            images: images,
        )
    }
}

extension SupportMessage {
    var record: SupportMessageRecord {
        SupportMessageRecord(
            id: id,
            content: content,
            sender: sender,
            status: status,
            createdAt: createdAt,
            images: images,
        )
    }
}
