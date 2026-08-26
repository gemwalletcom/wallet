// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.InAppNotification
import protocol Gemstone.GemNotificationStore
import GemstonePrimitives
import Preferences
import Primitives
import Store

public final class GemstoneNotificationStore: GemNotificationStore, @unchecked Sendable {
    private let store: InAppNotificationStore

    public init(store: InAppNotificationStore) {
        self.store = store
    }

    public func save(notifications: [Gemstone.InAppNotification]) async throws {
        try store.addNotifications(notifications.map { try Primitives.InAppNotification($0) })
    }

    public func getSyncTimestamp(walletId: String) async throws -> UInt64 {
        try UInt64(WalletPreferences(walletId: WalletId.from(id: walletId)).notificationsTimestamp)
    }

    public func setSyncTimestamp(walletId: String, timestamp: UInt64) async throws {
        try WalletPreferences(walletId: WalletId.from(id: walletId)).notificationsTimestamp = Int(timestamp)
    }
}
