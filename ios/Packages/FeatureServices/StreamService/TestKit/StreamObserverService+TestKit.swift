// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemStreamServiceProtocol
import protocol Gemstone.GemStreamSubscriptionServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import StreamService
import WebSocketClient
import WebSocketClientTestKit

public extension StreamObserverService {
    static func mock(
        subscriptionService: any GemStreamSubscriptionServiceProtocol = GemStreamSubscriptionServiceMock(),
        service: any GemStreamServiceProtocol = GemStreamServiceMock(),
        preferencesService: any GemPreferencesServiceProtocol = GemPreferencesServiceMock(),
        webSocket: any WebSocketConnectable = WebSocketConnectionMock(),
    ) -> StreamObserverService {
        StreamObserverService(
            subscriptionService: subscriptionService,
            service: service,
            preferencesService: preferencesService,
            webSocket: webSocket,
        )
    }
}
