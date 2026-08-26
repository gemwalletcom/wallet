// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNotificationServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives
import Store

public struct InAppNotificationService: Sendable {
    private let apiService: any GemNotificationServiceProtocol
    private let store: InAppNotificationStore

    public init(
        apiService: any GemNotificationServiceProtocol,
        store: InAppNotificationStore,
    ) {
        self.apiService = apiService
        self.store = store
    }

    public func update(walletId: WalletId) async throws {
        let preferences = WalletPreferences(walletId: walletId)
        let newTimestamp = Int(Date.now.timeIntervalSince1970)

        let notifications = try await apiService.getNotifications(
            fromTimestamp: UInt64(preferences.notificationsTimestamp),
        ).map { try InAppNotification($0) }
        try store.addNotifications(notifications)

        preferences.notificationsTimestamp = newTimestamp
    }

    public func markNotificationsRead() async throws {
        try await apiService.markRead()
    }
}
