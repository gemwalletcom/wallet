// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.InAppNotification
import protocol Gemstone.GemNotificationStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneNotificationStore: GemNotificationStore, @unchecked Sendable {
    private let store: InAppNotificationStore

    public init(store: InAppNotificationStore) {
        self.store = store
    }

    public func saveNotifications(notifications: [Gemstone.InAppNotification]) async throws {
        try store.addNotifications(notifications.map { try Primitives.InAppNotification($0) })
    }

    public func hasUnreadNotifications(walletId: String) async throws -> Bool {
        try store.hasUnreadNotifications(walletId: WalletId.from(id: walletId))
    }
}
