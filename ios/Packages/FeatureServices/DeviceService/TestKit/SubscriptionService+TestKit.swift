// Copyright (c). Gem Wallet. All rights reserved.

import DeviceService
import protocol Gemstone.GemSubscriptionServiceProtocol
import GemstonePrimitivesTestKit
import Preferences

public extension SubscriptionService {
    static func mock(
        subscriptionProvider: any GemSubscriptionServiceProtocol = GemSubscriptionServiceMock(),
        preferences: Preferences = .standard,
    ) -> SubscriptionService {
        SubscriptionService(
            subscriptionProvider: subscriptionProvider,
            preferences: preferences,
        )
    }
}
