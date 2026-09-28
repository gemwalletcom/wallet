// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNotificationStore
import struct Gemstone.InAppNotification
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneNotificationStore: GemNotificationStore, @unchecked Sendable {
    private let store: InAppNotificationStore

    public init(store: InAppNotificationStore) {
        self.store = store
    }

    public func saveNotifications(notifications: [Gemstone.InAppNotification]) async throws {
        try store.addNotifications(notifications.map { $0.toPrimitives() })
    }

    public func markNotificationsRead(walletId: WalletId, createdBefore: Date) async throws {
        try store.markNotificationsRead(walletId: walletId, createdBefore: createdBefore)
    }

    public func hasUnreadNotifications(walletId: WalletId) async throws -> Bool {
        try store.hasUnreadNotifications(walletId: walletId)
    }
}
