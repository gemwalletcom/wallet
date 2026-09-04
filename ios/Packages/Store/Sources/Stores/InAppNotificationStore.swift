// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct InAppNotificationStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    public func addNotifications(_ notifications: [Primitives.InAppNotification]) throws {
        try db.write { db in
            for notification in notifications {
                try notification.record().upsert(db)
            }
        }
    }

    public func hasUnreadNotifications(walletId: WalletId) throws -> Bool {
        try db.read { db in
            try NotificationRecord
                .filter(NotificationRecord.Columns.walletId == walletId.id)
                .filter(NotificationRecord.Columns.readAt == nil)
                .fetchCount(db) > 0
        }
    }
}
