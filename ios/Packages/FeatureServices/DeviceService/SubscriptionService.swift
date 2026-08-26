// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSubscriptionServiceProtocol
import Preferences

public struct SubscriptionService: Sendable {
    private let subscriptionProvider: any GemSubscriptionServiceProtocol
    private let preferences: Preferences

    public init(
        subscriptionProvider: any GemSubscriptionServiceProtocol,
        preferences: Preferences = .standard,
    ) {
        self.subscriptionProvider = subscriptionProvider
        self.preferences = preferences
    }

    public func invalidateSubscriptions() {
        preferences.invalidateSubscriptions()
    }

    public func update() async throws {
        _ = try await subscriptionProvider.sync()
        preferences.subscriptionsVersionHasChange = false
    }
}
