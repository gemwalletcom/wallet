// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PriceService
import PriceServiceTestKit
import StreamService
import WebSocketClient
import WebSocketClientTestKit

public extension StreamSubscriptionService {
    static func mock(
        priceService: PriceService = .mock(),
        webSocket: any WebSocketConnectable = WebSocketConnectionMock(),
    ) -> StreamSubscriptionService {
        StreamSubscriptionService(
            priceService: priceService,
            webSocket: webSocket,
        )
    }
}
