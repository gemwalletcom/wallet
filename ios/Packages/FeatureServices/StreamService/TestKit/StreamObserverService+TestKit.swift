// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemStreamServiceProtocol
import protocol Gemstone.GemStreamSubscriptionServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Preferences
import PreferencesTestKit
import StreamService
import WebSocketClient
import WebSocketClientTestKit

public extension StreamObserverService {
    static func mock(
        subscriptionService: any GemStreamSubscriptionServiceProtocol = GemStreamSubscriptionServiceMock(),
        service: any GemStreamServiceProtocol = GemStreamServiceMock(),
        preferences: Preferences = .mock(),
        webSocket: any WebSocketConnectable = WebSocketConnectionMock(),
    ) -> StreamObserverService {
        StreamObserverService(
            subscriptionService: subscriptionService,
            service: service,
            preferences: preferences,
            webSocket: webSocket,
        )
    }
}
