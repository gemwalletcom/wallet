// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct SupportChatStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    public func addMessages(_ messages: [SupportMessage]) throws {
        try db.write { db in
            for message in messages {
                try message.record.upsert(db)
            }
        }
    }

    public func replace(id: String, with message: SupportMessage) throws {
        try db.write { db in
            _ = try SupportMessageRecord.deleteOne(db, key: id)
            try message.record.upsert(db)
        }
    }
}
