// Copyright (c). Gem Wallet. All rights reserved.

import DeviceService
import protocol Gemstone.GemSubscriptionServiceProtocol
import GemstonePrimitivesTestKit
import Preferences
import Store
import StoreTestKit

public extension SubscriptionService {
    static func mock(
        subscriptionProvider: any GemSubscriptionServiceProtocol = GemSubscriptionServiceMock(),
        walletStore: WalletStore = .mock(),
        preferences: Preferences = .standard,
    ) -> SubscriptionService {
        SubscriptionService(
            subscriptionProvider: subscriptionProvider,
            walletStore: walletStore,
            preferences: preferences,
        )
    }
}
