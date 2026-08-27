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

    public func save(notifications: [Gemstone.InAppNotification]) async throws {
        try store.addNotifications(notifications.map { try Primitives.InAppNotification($0) })
    }
}
